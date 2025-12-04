//! TUI Module - Interactive Terminal UI
//!
//! Provides a beautiful terminal interface using ratatui
//! for exploring dead code scan results.

use crate::types::{DeadCodeItem, DeadCodeKind, ScanOutput};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;

/// Run the interactive TUI
pub fn run_tui(scan_output: &ScanOutput) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(scan_output);

    // Run event loop
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}

struct App<'a> {
    scan_output: &'a ScanOutput,
    list_state: ListState,
    selected_index: usize,
    scroll_offset: u16,
}

impl<'a> App<'a> {
    fn new(scan_output: &'a ScanOutput) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            scan_output,
            list_state,
            selected_index: 0,
            scroll_offset: 0,
        }
    }

    fn next(&mut self) {
        if self.scan_output.dead_code.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.scan_output.dead_code.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.selected_index = i;
        self.scroll_offset = 0;
    }

    fn previous(&mut self) {
        if self.scan_output.dead_code.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.scan_output.dead_code.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.selected_index = i;
        self.scroll_offset = 0;
    }

    fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    fn selected_item(&self) -> Option<&DeadCodeItem> {
        self.scan_output.dead_code.get(self.selected_index)
    }
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous(),
                    KeyCode::Char('J') | KeyCode::PageDown => app.scroll_down(),
                    KeyCode::Char('K') | KeyCode::PageUp => app.scroll_up(),
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    // Header
    render_header(f, chunks[0], app);

    // Main content
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    // Left: List
    render_list(f, main_chunks[0], app);

    // Right: Details
    render_details(f, main_chunks[1], app);

    // Footer
    render_footer(f, chunks[2]);
}

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let summary = &app.scan_output.summary;

    let text = vec![Line::from(vec![
        Span::styled("🧹 clrd ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("| "),
        Span::styled(
            format!("{} files", app.scan_output.total_files_scanned),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} issues", summary.total_issues),
            Style::default().fg(if summary.total_issues > 0 {
                Color::Yellow
            } else {
                Color::Green
            }),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{} high confidence", summary.high_confidence_issues),
            Style::default().fg(Color::Red),
        ),
    ])];

    let header = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Dead Code Report"),
    );

    f.render_widget(header, area);
}

fn render_list(f: &mut Frame, area: Rect, app: &mut App) {
    let items: Vec<ListItem> = app
        .scan_output
        .dead_code
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let icon = kind_to_icon(&item.kind);
            let confidence_color = if item.confidence >= 0.8 {
                Color::Red
            } else if item.confidence >= 0.5 {
                Color::Yellow
            } else {
                Color::Green
            };

            let content = Line::from(vec![
                Span::raw(format!("{:>3}. ", i + 1)),
                Span::raw(format!("{} ", icon)),
                Span::styled(&item.name, Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(
                    format!("{:.0}%", item.confidence * 100.0),
                    Style::default().fg(confidence_color),
                ),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Issues"))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_details(f: &mut Frame, area: Rect, app: &App) {
    let content = if let Some(item) = app.selected_item() {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&item.name),
            ]),
            Line::from(vec![
                Span::styled("Kind: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{}", item.kind)),
            ]),
            Line::from(vec![
                Span::styled("File: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(&item.relative_path, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Line: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{}", item.span.start)),
            ]),
            Line::from(vec![
                Span::styled(
                    "Confidence: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:.0}%", item.confidence * 100.0),
                    Style::default().fg(if item.confidence >= 0.8 {
                        Color::Red
                    } else {
                        Color::Yellow
                    }),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Reason:",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from(item.reason.as_str()),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Code:",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
        ];

        // Add code snippet
        for line in item.code_snippet.lines() {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(Color::DarkGray),
            )));
        }

        Text::from(lines)
    } else {
        Text::from("No item selected")
    };

    let details = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Details"))
        .wrap(Wrap { trim: false })
        .scroll((app.scroll_offset, 0));

    f.render_widget(details, area);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let help = Line::from(vec![
        Span::styled("↑/k", Style::default().fg(Color::Yellow)),
        Span::raw(" Up  "),
        Span::styled("↓/j", Style::default().fg(Color::Yellow)),
        Span::raw(" Down  "),
        Span::styled("K/PageUp", Style::default().fg(Color::Yellow)),
        Span::raw(" Scroll Up  "),
        Span::styled("J/PageDown", Style::default().fg(Color::Yellow)),
        Span::raw(" Scroll Down  "),
        Span::styled("q/Esc", Style::default().fg(Color::Yellow)),
        Span::raw(" Quit"),
    ]);

    let footer = Paragraph::new(help)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .style(Style::default().fg(Color::DarkGray));

    f.render_widget(footer, area);
}

fn kind_to_icon(kind: &DeadCodeKind) -> &'static str {
    use DeadCodeKind::*;
    match kind {
        UnusedExport => "📤",
        UnreachableFunction => "🔒",
        UnusedVariable => "📦",
        UnusedImport => "📥",
        ZombieFile => "🧟",
        UnusedType => "📝",
        UnusedClass => "🏛️",
        UnusedEnum => "🔢",
        DeadBranch => "🌿",
    }
}
