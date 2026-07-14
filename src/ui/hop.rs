//! Multi-level hop UI: Project → Agent → Session in one alternate-screen session.

use std::io::{self, Stdout};
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Utc;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};
use ratatui::Terminal;

use crate::db::{launches, Database};
use crate::error::{RepoHopError, Result};
use crate::paths::display_path;
use crate::project::Project;
use crate::provider::{
    detect_installed, provider_by_id, DetectedAgent, ProviderId, SessionSummary,
};
use crate::ui::timefmt::format_relative_time;

#[derive(Debug, Clone)]
pub struct ProjectRow {
    pub name: String,
    pub path: String,
    pub last_used: String,
}

#[derive(Debug, Clone)]
pub struct AgentRow {
    pub provider: ProviderId,
    pub name: String,
    pub last_used: String,
    pub uses: String,
}

#[derive(Debug, Clone)]
pub struct SessionRow {
    pub title: String,
    pub last_used: String,
    pub preview: String,
    /// None = "New chat" synthetic row.
    pub session: Option<SessionSummary>,
}

/// Final choice after navigating the multi-level UI.
#[derive(Debug, Clone)]
pub enum HopChoice {
    New {
        project: Project,
        provider: ProviderId,
    },
    Resume {
        project: Project,
        provider: ProviderId,
        session: SessionSummary,
    },
}

enum Screen {
    Projects,
    Agents,
    Sessions,
    PathInput,
}

struct HopState {
    screen: Screen,
    projects: Vec<Project>,
    project_rows: Vec<ProjectRow>,
    project_state: TableState,
    selected_project: Option<Project>,
    agents: Vec<DetectedAgent>,
    agent_rows: Vec<AgentRow>,
    agent_state: TableState,
    selected_provider: Option<ProviderId>,
    session_rows: Vec<SessionRow>,
    session_state: TableState,
    path_input: String,
    status: Option<String>,
    update_banner: Option<String>,
}

/// Run the full hop flow inside a single alternate-screen session.
///
/// When `start_at_agents` is true (e.g. `rhop .`), skip the project table and
/// open the tools screen for the first project.
pub fn run_hop_ui(
    db: &Database,
    projects: Vec<Project>,
    update_banner: Option<String>,
    start_at_agents: bool,
) -> Result<HopChoice> {
    ensure_tty()?;
    let agents = detect_installed();
    if agents.is_empty() {
        return Err(RepoHopError::NoAgents);
    }

    let mut terminal = enter_tui()?;
    let now = Utc::now();
    let project_rows = build_project_rows(&projects, now);
    let mut project_state = TableState::default();
    if !project_rows.is_empty() {
        project_state.select(Some(0));
    }

    let mut st = HopState {
        screen: Screen::Projects,
        projects,
        project_rows,
        project_state,
        selected_project: None,
        agents,
        agent_rows: Vec::new(),
        agent_state: TableState::default(),
        selected_provider: None,
        session_rows: Vec::new(),
        session_state: TableState::default(),
        path_input: String::new(),
        status: None,
        update_banner,
    };

    if start_at_agents {
        if let Some(project) = st.projects.first().cloned() {
            enter_agents(db, &mut st, project);
        }
    }

    let result = loop_ui(db, &mut terminal, &mut st);
    leave_tui(&mut terminal);
    result
}

fn build_project_rows(projects: &[Project], now: chrono::DateTime<Utc>) -> Vec<ProjectRow> {
    projects
        .iter()
        .map(|p| {
            let mut name = p.name.clone();
            if p.is_favorite {
                name = format!("★ {name}");
            }
            if !p.exists() {
                name = format!("[?] {name}");
            }
            ProjectRow {
                name,
                path: display_path(&p.path),
                last_used: format_relative_time(now, p.last_launched_at),
            }
        })
        .collect()
}

fn build_agent_rows(db: &Database, project: &Project, agents: &[DetectedAgent]) -> Vec<AgentRow> {
    let now = Utc::now();
    let stats = launches::stats_for_project(db.conn(), &project.path).unwrap_or_default();
    let mut rows: Vec<AgentRow> = agents
        .iter()
        .map(|a| {
            let st = stats.iter().find(|s| s.provider == a.provider);
            let (count, last) = match st {
                Some(s) => (s.launch_count, s.last_launched_at),
                None => (0, None),
            };
            // Prefer project-level last_provider recency for ranking display when no launches yet.
            let last = last.or_else(|| {
                if project.last_provider == Some(a.provider) {
                    project.last_launched_at
                } else {
                    None
                }
            });
            AgentRow {
                provider: a.provider,
                name: a.provider.as_str().to_string(),
                last_used: format_relative_time(now, last),
                uses: count.to_string(),
            }
        })
        .collect();
    // Sort: most recent use first, then uses desc, then name.
    rows.sort_by(|a, b| {
        let ai = agents.iter().position(|x| x.provider == a.provider);
        let bi = agents.iter().position(|x| x.provider == b.provider);
        let a_last = stats
            .iter()
            .find(|s| s.provider == a.provider)
            .and_then(|s| s.last_launched_at);
        let b_last = stats
            .iter()
            .find(|s| s.provider == b.provider)
            .and_then(|s| s.last_launched_at);
        b_last
            .cmp(&a_last)
            .then_with(|| {
                let ac: i64 = a.uses.parse().unwrap_or(0);
                let bc: i64 = b.uses.parse().unwrap_or(0);
                bc.cmp(&ac)
            })
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| ai.cmp(&bi))
    });
    rows
}

fn build_session_rows(project: &Project, provider: ProviderId) -> Vec<SessionRow> {
    let now = Utc::now();
    let mut rows = vec![SessionRow {
        title: "＋ New chat".into(),
        last_used: "—".into(),
        preview: "Start a fresh session".into(),
        session: None,
    }];
    let provider_box = provider_by_id(provider);
    match provider_box.list_sessions(&project.path) {
        Ok(sessions) => {
            for s in sessions {
                rows.push(SessionRow {
                    title: s.title.clone(),
                    last_used: format_relative_time(now, s.updated_at.or(s.created_at)),
                    preview: if s.preview.is_empty() {
                        s.id.clone()
                    } else {
                        s.preview.clone()
                    },
                    session: Some(s),
                });
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "list_sessions failed");
        }
    }
    rows
}

fn loop_ui(
    db: &Database,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    st: &mut HopState,
) -> Result<HopChoice> {
    loop {
        terminal.draw(|f| draw(f, st)).map_err(RepoHopError::Io)?;
        // Note: draw takes &mut via interior state fields updated only on keys.

        if !event::poll(Duration::from_millis(200)).map_err(RepoHopError::Io)? {
            continue;
        }
        let Event::Key(key) = event::read().map_err(RepoHopError::Io)? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match st.screen {
            Screen::Projects => {
                if let Some(choice) = handle_projects(db, st, key.code)? {
                    return Ok(choice);
                }
            }
            Screen::Agents => {
                if let Some(choice) = handle_agents(st, key.code)? {
                    return Ok(choice);
                }
            }
            Screen::Sessions => {
                if let Some(choice) = handle_sessions(st, key.code)? {
                    return Ok(choice);
                }
            }
            Screen::PathInput => {
                handle_path_input(db, st, key.code)?;
            }
        }
    }
}

fn handle_projects(db: &Database, st: &mut HopState, code: KeyCode) -> Result<Option<HopChoice>> {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => Err(RepoHopError::Cancelled),
        KeyCode::Char('.') => {
            let cwd = std::env::current_dir().map_err(RepoHopError::Io)?;
            enter_agents_for_path(db, st, &cwd)?;
            Ok(None)
        }
        KeyCode::Char('n') | KeyCode::Char('a') => {
            st.path_input.clear();
            st.screen = Screen::PathInput;
            st.status = None;
            Ok(None)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            move_sel(&mut st.project_state, st.project_rows.len(), 1);
            Ok(None)
        }
        KeyCode::Up | KeyCode::Char('k') => {
            move_sel(&mut st.project_state, st.project_rows.len(), -1);
            Ok(None)
        }
        KeyCode::Enter => {
            if st.projects.is_empty() {
                st.status = Some("No project — use . or n".into());
                return Ok(None);
            }
            let idx = st.project_state.selected().unwrap_or(0);
            let project = st
                .projects
                .get(idx)
                .cloned()
                .ok_or_else(|| RepoHopError::Config("invalid project".into()))?;
            if !project.exists() {
                st.status = Some(format!("Missing: {}", display_path(&project.path)));
                return Ok(None);
            }
            enter_agents(db, st, project);
            Ok(None)
        }
        _ => Ok(None),
    }
}

fn handle_agents(st: &mut HopState, code: KeyCode) -> Result<Option<HopChoice>> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace => {
            st.screen = Screen::Projects;
            st.selected_project = None;
            st.status = None;
            Ok(None)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            move_sel(&mut st.agent_state, st.agent_rows.len(), 1);
            Ok(None)
        }
        KeyCode::Up | KeyCode::Char('k') => {
            move_sel(&mut st.agent_state, st.agent_rows.len(), -1);
            Ok(None)
        }
        KeyCode::Enter => {
            if st.agent_rows.is_empty() {
                return Ok(None);
            }
            let idx = st.agent_state.selected().unwrap_or(0);
            let provider = st.agent_rows[idx].provider;
            enter_sessions(st, provider);
            Ok(None)
        }
        _ => Ok(None),
    }
}

fn handle_sessions(st: &mut HopState, code: KeyCode) -> Result<Option<HopChoice>> {
    match code {
        KeyCode::Esc | KeyCode::Backspace => {
            st.screen = Screen::Agents;
            st.selected_provider = None;
            st.status = None;
            Ok(None)
        }
        KeyCode::Char('q') => Err(RepoHopError::Cancelled),
        KeyCode::Char('n') => {
            // New chat shortcut
            let project = st
                .selected_project
                .clone()
                .ok_or_else(|| RepoHopError::Config("no project".into()))?;
            let provider = st
                .selected_provider
                .ok_or_else(|| RepoHopError::Config("no provider".into()))?;
            Ok(Some(HopChoice::New { project, provider }))
        }
        KeyCode::Down | KeyCode::Char('j') => {
            move_sel(&mut st.session_state, st.session_rows.len(), 1);
            Ok(None)
        }
        KeyCode::Up | KeyCode::Char('k') => {
            move_sel(&mut st.session_state, st.session_rows.len(), -1);
            Ok(None)
        }
        KeyCode::Enter => {
            let project = st
                .selected_project
                .clone()
                .ok_or_else(|| RepoHopError::Config("no project".into()))?;
            let provider = st
                .selected_provider
                .ok_or_else(|| RepoHopError::Config("no provider".into()))?;
            if st.session_rows.is_empty() {
                return Ok(Some(HopChoice::New { project, provider }));
            }
            let idx = st.session_state.selected().unwrap_or(0);
            let row = &st.session_rows[idx];
            match &row.session {
                None => Ok(Some(HopChoice::New { project, provider })),
                Some(session) => Ok(Some(HopChoice::Resume {
                    project,
                    provider,
                    session: session.clone(),
                })),
            }
        }
        _ => Ok(None),
    }
}

fn handle_path_input(db: &Database, st: &mut HopState, code: KeyCode) -> Result<()> {
    match code {
        KeyCode::Esc => {
            st.screen = Screen::Projects;
            st.path_input.clear();
            st.status = Some("Path entry cancelled".into());
        }
        KeyCode::Enter => {
            let raw = st.path_input.trim().to_string();
            if raw.is_empty() {
                st.status = Some("Empty path".into());
                return Ok(());
            }
            let path = resolve_user_path(Path::new(&raw))?;
            if !path.is_dir() {
                st.status = Some(format!("Not a directory: {}", display_path(&path)));
                st.screen = Screen::Projects;
                return Ok(());
            }
            st.path_input.clear();
            enter_agents_for_path(db, st, &path)?;
        }
        KeyCode::Backspace => {
            st.path_input.pop();
        }
        KeyCode::Char(c) => {
            st.path_input.push(c);
        }
        _ => {}
    }
    Ok(())
}

fn enter_agents_for_path(db: &Database, st: &mut HopState, path: &Path) -> Result<()> {
    let project = crate::project::ensure_cwd_project(db, path)?;
    // Keep project in list for display consistency.
    if !st.projects.iter().any(|p| p.path == project.path) {
        st.projects.insert(0, project.clone());
        st.project_rows = build_project_rows(&st.projects, Utc::now());
    }
    enter_agents(db, st, project);
    Ok(())
}

fn enter_agents(db: &Database, st: &mut HopState, project: Project) {
    st.agent_rows = build_agent_rows(db, &project, &st.agents);
    st.agent_state = TableState::default();
    // Prefer last_provider if present.
    let default = project
        .last_provider
        .and_then(|lp| st.agent_rows.iter().position(|r| r.provider == lp))
        .unwrap_or(0);
    if !st.agent_rows.is_empty() {
        st.agent_state
            .select(Some(default.min(st.agent_rows.len() - 1)));
    }
    st.selected_project = Some(project);
    st.screen = Screen::Agents;
    st.status = None;
}

fn enter_sessions(st: &mut HopState, provider: ProviderId) {
    let project = match &st.selected_project {
        Some(p) => p.clone(),
        None => return,
    };
    st.session_rows = build_session_rows(&project, provider);
    st.session_state = TableState::default();
    st.session_state.select(Some(0)); // New chat on top
    st.selected_provider = Some(provider);
    st.screen = Screen::Sessions;
    st.status = None;
}

fn move_sel(state: &mut TableState, len: usize, delta: i32) {
    if len == 0 {
        return;
    }
    let i = state.selected().unwrap_or(0) as i32;
    let next = (i + delta).rem_euclid(len as i32) as usize;
    state.select(Some(next));
}

fn draw(f: &mut ratatui::Frame<'_>, st: &mut HopState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(if st.update_banner.is_some() { 1 } else { 0 }),
            Constraint::Min(3),
            Constraint::Length(1),
            Constraint::Length(2),
        ])
        .split(f.area());

    if let Some(banner) = &st.update_banner {
        f.render_widget(
            Paragraph::new(banner.as_str()).style(Style::default().add_modifier(Modifier::BOLD)),
            chunks[0],
        );
    }

    match st.screen {
        Screen::Projects => draw_projects(f, chunks[1], st),
        Screen::Agents => draw_agents(f, chunks[1], st),
        Screen::Sessions => draw_sessions(f, chunks[1], st),
        Screen::PathInput => draw_path_input(f, chunks[1], st),
    }

    let status = st.status.as_deref().unwrap_or("");
    f.render_widget(
        Paragraph::new(status).style(Style::default().add_modifier(Modifier::DIM)),
        chunks[2],
    );

    let help = match st.screen {
        Screen::Projects => {
            if st.project_rows.is_empty() {
                "No projects  . = cwd  n/a = add path  Esc quit"
            } else {
                "↑/↓  Enter open tools  . = cwd  n/a = path  Esc quit"
            }
        }
        Screen::Agents => "↑/↓  Enter sessions  Esc back",
        Screen::Sessions => "↑/↓  Enter open  n = new chat  Esc back",
        Screen::PathInput => "Type path  Enter confirm  Esc cancel",
    };
    f.render_widget(Paragraph::new(help), chunks[3]);
}

fn draw_projects(f: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect, st: &mut HopState) {
    let header = Row::new(vec![
        Cell::from("Name"),
        Cell::from("Path"),
        Cell::from("Last used"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));
    let rows: Vec<Row> = st
        .project_rows
        .iter()
        .map(|r| {
            Row::new(vec![
                Cell::from(r.name.clone()),
                Cell::from(r.path.clone()),
                Cell::from(r.last_used.clone()),
            ])
        })
        .collect();
    let table = Table::new(
        rows,
        [
            Constraint::Percentage(22),
            Constraint::Percentage(58),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("1/3  Select project"),
    )
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol("▶ ");
    f.render_stateful_widget(table, area, &mut st.project_state);
}

fn draw_agents(f: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect, st: &mut HopState) {
    let proj = st
        .selected_project
        .as_ref()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "?".into());
    let header = Row::new(vec![
        Cell::from("Tool"),
        Cell::from("Last used"),
        Cell::from("Uses"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));
    let rows: Vec<Row> = st
        .agent_rows
        .iter()
        .map(|r| {
            Row::new(vec![
                Cell::from(r.name.clone()),
                Cell::from(r.last_used.clone()),
                Cell::from(r.uses.clone()),
            ])
        })
        .collect();
    let table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Percentage(35),
            Constraint::Percentage(25),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("2/3  Tools — {proj}")),
    )
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol("▶ ");
    f.render_stateful_widget(table, area, &mut st.agent_state);
}

fn draw_sessions(f: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect, st: &mut HopState) {
    let proj = st
        .selected_project
        .as_ref()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "?".into());
    let tool = st
        .selected_provider
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| "?".into());
    let header = Row::new(vec![
        Cell::from("Chat"),
        Cell::from("Last used"),
        Cell::from("Preview"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));
    let rows: Vec<Row> = st
        .session_rows
        .iter()
        .map(|r| {
            Row::new(vec![
                Cell::from(r.title.clone()),
                Cell::from(r.last_used.clone()),
                Cell::from(r.preview.clone()),
            ])
        })
        .collect();
    let table = Table::new(
        rows,
        [
            Constraint::Percentage(35),
            Constraint::Percentage(18),
            Constraint::Percentage(47),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("3/3  {tool} chats — {proj}  (n = new)")),
    )
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol("▶ ");
    f.render_stateful_widget(table, area, &mut st.session_state);
}

fn draw_path_input(f: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect, st: &HopState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Path to project");
    let inner = block.inner(area);
    f.render_widget(block, area);
    f.render_widget(Paragraph::new(format!("{}█", st.path_input)), inner);
}

fn ensure_tty() -> Result<()> {
    use std::io::IsTerminal;
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(RepoHopError::NotTty);
    }
    Ok(())
}

fn enter_tui() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    crossterm::terminal::enable_raw_mode().map_err(RepoHopError::Io)?;
    let mut stdout = io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )
    .map_err(RepoHopError::Io)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).map_err(RepoHopError::Io)
}

fn leave_tui(terminal: &mut Terminal<CrosstermBackend<Stdout>>) {
    crossterm::terminal::disable_raw_mode().ok();
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )
    .ok();
    terminal.show_cursor().ok();
}

/// Resolve a user-typed path against cwd.
pub fn resolve_user_path(input: &Path) -> Result<PathBuf> {
    let cwd = std::env::current_dir().map_err(RepoHopError::Io)?;
    if input.as_os_str().is_empty() {
        return Err(RepoHopError::Config("empty path".into()));
    }
    let joined = if input.is_absolute() {
        input.to_path_buf()
    } else {
        cwd.join(input)
    };
    match std::fs::canonicalize(&joined) {
        Ok(p) => Ok(crate::paths::normalize_path(&p)),
        Err(_) => Ok(joined),
    }
}
