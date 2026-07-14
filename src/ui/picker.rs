use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::time::Duration;

use crate::error::{RepoHopError, Result};

pub struct PickItem {
    pub label: String,
    pub detail: String,
}

/// Simple ↑↓ Enter Esc list picker.
pub fn pick_list(title: &str, items: &[PickItem], default_index: usize) -> Result<usize> {
    if items.is_empty() {
        return Err(RepoHopError::Config("nothing to pick".into()));
    }
    use std::io::IsTerminal;
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(RepoHopError::NotTty);
    }

    crossterm::terminal::enable_raw_mode().map_err(RepoHopError::Io)?;
    let mut stdout = io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )
    .map_err(RepoHopError::Io)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(RepoHopError::Io)?;

    let mut state = ListState::default();
    let start = default_index.min(items.len() - 1);
    state.select(Some(start));

    let result = run_loop(&mut terminal, title, items, &mut state);

    crossterm::terminal::disable_raw_mode().ok();
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )
    .ok();
    terminal.show_cursor().ok();

    result
}

fn run_loop(
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
