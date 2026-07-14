use std::io::{self, Stdout};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, TableState,
};
use ratatui::Terminal;

use crate::error::{RepoHopError, Result};

pub struct PickItem {
    pub label: String,
    pub detail: String,
}

/// One row in the project table picker.
#[derive(Debug, Clone)]
pub struct ProjectRow {
    pub name: String,
    pub path: String,
    pub last_used: String,
}

/// Result of the project table picker.
#[derive(Debug, Clone)]
pub enum PickOutcome {
    /// Selected the row at this index in the input `rows` slice.
    Index(usize),
    /// Use the process current working directory.
    Cwd,
    /// User typed a path (may be relative; caller resolves & validates).
    NewPath(PathBuf),
}

enum PathPromptResult {
    Confirmed(String),
    Cancelled,
}

/// Simple ↑↓ Enter Esc list picker (agents, etc.).
pub fn pick_list(title: &str, items: &[PickItem], default_index: usize) -> Result<usize> {
    if items.is_empty() {
        return Err(RepoHopError::Config("nothing to pick".into()));
    }
    ensure_tty()?;

    let mut terminal = enter_tui()?;
    let mut state = ListState::default();
    let start = default_index.min(items.len() - 1);
    state.select(Some(start));

    let result = run_list_loop(&mut terminal, title, items, &mut state);
    leave_tui(&mut terminal);
    result
}

/// Column-aligned project picker with cwd / new-path shortcuts.
pub fn pick_project_table(
    title: &str,
    rows: &[ProjectRow],
    default_index: usize,
) -> Result<PickOutcome> {
    // Empty list is OK — user can still press `.` or `n`.
    ensure_tty()?;

    let mut terminal = enter_tui()?;
    let mut state = TableState::default();
    if !rows.is_empty() {
        let start = default_index.min(rows.len() - 1);
        state.select(Some(start));
    }

    let result = run_table_loop(&mut terminal, title, rows, &mut state);
    leave_tui(&mut terminal);
    result
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

fn run_list_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    title: &str,
    items: &[PickItem],
    state: &mut ListState,
) -> Result<usize> {
    loop {
        terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(3), Constraint::Length(2)])
                    .split(f.area());

                let list_items: Vec<ListItem> = items
                    .iter()
                    .map(|i| {
                        let line = if i.detail.is_empty() {
                            Line::from(i.label.clone())
                        } else {
                            Line::from(vec![
                                Span::raw(format!("{}  ", i.label)),
                                Span::styled(
                                    i.detail.clone(),
                                    Style::default().add_modifier(Modifier::DIM),
                                ),
                            ])
                        };
                        ListItem::new(line)
                    })
                    .collect();

                let list = List::new(list_items)
                    .block(Block::default().borders(Borders::ALL).title(title))
                    .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                    .highlight_symbol("▶ ");
                f.render_stateful_widget(list, chunks[0], state);

                let help = Paragraph::new("↑/↓ move  Enter select  Esc cancel");
                f.render_widget(help, chunks[1]);
            })
            .map_err(RepoHopError::Io)?;

        if event::poll(Duration::from_millis(200)).map_err(RepoHopError::Io)? {
            if let Event::Key(key) = event::read().map_err(RepoHopError::Io)? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Err(RepoHopError::Cancelled),
                    KeyCode::Down | KeyCode::Char('j') => {
                        let i = state.selected().unwrap_or(0);
                        let next = if i + 1 >= items.len() { 0 } else { i + 1 };
                        state.select(Some(next));
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let i = state.selected().unwrap_or(0);
                        let next = if i == 0 { items.len() - 1 } else { i - 1 };
                        state.select(Some(next));
                    }
                    KeyCode::Enter => {
                        return Ok(state.selected().unwrap_or(0));
                    }
                    _ => {}
                }
            }
        }
    }
}

fn run_table_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    title: &str,
    rows: &[ProjectRow],
    state: &mut TableState,
) -> Result<PickOutcome> {
    let mut status: Option<String> = None;

    loop {
        terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(3),
                        Constraint::Length(1),
                        Constraint::Length(2),
                    ])
                    .split(f.area());

                let header = Row::new(vec![
                    Cell::from("Name"),
                    Cell::from("Path"),
                    Cell::from("Last used"),
                ])
                .style(Style::default().add_modifier(Modifier::BOLD));

                let table_rows: Vec<Row> = rows
                    .iter()
                    .map(|r| {
                        Row::new(vec![
                            Cell::from(r.name.clone()),
                            Cell::from(r.path.clone()),
                            Cell::from(r.last_used.clone()),
                        ])
                    })
                    .collect();

                let widths = [
                    Constraint::Percentage(22),
                    Constraint::Percentage(58),
                    Constraint::Percentage(20),
                ];

                let table = Table::new(table_rows, widths)
                    .header(header)
                    .block(Block::default().borders(Borders::ALL).title(title))
                    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                    .highlight_symbol("▶ ");

                f.render_stateful_widget(table, chunks[0], state);

                let status_line = status.as_deref().unwrap_or("");
                f.render_widget(
                    Paragraph::new(status_line).style(Style::default().add_modifier(Modifier::DIM)),
                    chunks[1],
                );

                let help = if rows.is_empty() {
                    "No projects yet  . = cwd  n/a = add path  Esc cancel"
                } else {
                    "↑/↓ j/k  Enter select  . = cwd  n/a = add path  Esc cancel"
                };
                f.render_widget(Paragraph::new(help), chunks[2]);
            })
            .map_err(RepoHopError::Io)?;

        if event::poll(Duration::from_millis(200)).map_err(RepoHopError::Io)? {
            if let Event::Key(key) = event::read().map_err(RepoHopError::Io)? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Err(RepoHopError::Cancelled),
                    KeyCode::Char('.') => return Ok(PickOutcome::Cwd),
                    KeyCode::Char('n') | KeyCode::Char('a') => {
                        match prompt_path(terminal, "Path to project:")? {
                            PathPromptResult::Cancelled => {
                                status = Some("Path entry cancelled".into());
                            }
                            PathPromptResult::Confirmed(s) => {
                                let trimmed = s.trim();
                                if trimmed.is_empty() {
                                    status = Some("Empty path".into());
                                } else {
                                    return Ok(PickOutcome::NewPath(PathBuf::from(trimmed)));
                                }
                            }
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if rows.is_empty() {
                            continue;
                        }
                        let i = state.selected().unwrap_or(0);
                        let next = if i + 1 >= rows.len() { 0 } else { i + 1 };
                        state.select(Some(next));
                        status = None;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if rows.is_empty() {
                            continue;
                        }
                        let i = state.selected().unwrap_or(0);
                        let next = if i == 0 { rows.len() - 1 } else { i - 1 };
                        state.select(Some(next));
                        status = None;
                    }
                    KeyCode::Enter => {
                        if rows.is_empty() {
                            status = Some("No project selected — use . or n".into());
                            continue;
                        }
                        return Ok(PickOutcome::Index(state.selected().unwrap_or(0)));
                    }
                    _ => {}
                }
            }
        }
    }
}

fn prompt_path(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    prompt: &str,
) -> Result<PathPromptResult> {
    let mut input = String::new();
    loop {
        terminal
            .draw(|f| {
                let area = f.area();
                let block = Block::default().borders(Borders::ALL).title(prompt);
                let inner = block.inner(area);
                f.render_widget(block, area);
                let line = format!("{input}█");
                f.render_widget(Paragraph::new(line), inner);
            })
            .map_err(RepoHopError::Io)?;

        if event::poll(Duration::from_millis(200)).map_err(RepoHopError::Io)? {
            if let Event::Key(key) = event::read().map_err(RepoHopError::Io)? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Esc => return Ok(PathPromptResult::Cancelled),
                    KeyCode::Enter => return Ok(PathPromptResult::Confirmed(input)),
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Char(c) => {
                        input.push(c);
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Resolve a user-typed path against cwd; expand a lone `.`.
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
    // canonicalize if possible for cleaner storage; fall back to joined.
    match dunce_canonicalize(&joined) {
        Ok(p) => Ok(p),
        Err(_) => Ok(joined),
    }
}

fn dunce_canonicalize(path: &Path) -> std::io::Result<PathBuf> {
    // Avoid depending on `dunce` crate; std canonicalize is fine on Windows for existing dirs.
    std::fs::canonicalize(path)
}
