pub mod app;
pub mod input;
pub mod widgets;

use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

use crate::cache::sqlite::{LogEntry, SqliteCache, Stats};
use crate::config;
use crate::core::risk::RiskLevel;

use self::app::AppState;
use self::input::{apply_action, map_key, Action};

/// Data loaded from SQLite for display.
struct DashboardData {
    log_entries: Vec<LogEntry>,
    stats: Stats,
}

impl DashboardData {
    fn load(cache: &SqliteCache) -> Self {
        let log_entries = cache.get_log(50).unwrap_or_default();
        let stats = cache.get_stats().unwrap_or(Stats {
            total: 0,
            by_source: Default::default(),
            by_level: Default::default(),
            cache_hit_rate: 0.0,
        });
        Self { log_entries, stats }
    }
}

/// Run the TUI dashboard.
pub fn run() {
    // Open SQLite cache
    let db_path = config::bark_db_path();
    let cache = match SqliteCache::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to open database at {}: {}", db_path.display(), e);
            eprintln!("Run some assessments first to create the database.");
            return;
        }
    };

    // Set up terminal
    if let Err(e) = enable_raw_mode() {
        eprintln!("Failed to enable raw mode: {}", e);
        return;
    }
    let mut stdout = io::stdout();
    if let Err(e) = stdout.execute(EnterAlternateScreen) {
        let _ = disable_raw_mode();
        eprintln!("Failed to enter alternate screen: {}", e);
        return;
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::new(backend) {
        Ok(t) => t,
        Err(e) => {
            let _ = io::stdout().execute(LeaveAlternateScreen);
            let _ = disable_raw_mode();
            eprintln!("Failed to create terminal: {}", e);
            return;
        }
    };

    let result = run_app(&mut terminal, &cache);

    // Restore terminal
    let _ = disable_raw_mode();
    let _ = terminal.backend_mut().execute(LeaveAlternateScreen);

    if let Err(e) = result {
        eprintln!("TUI error: {}", e);
    }
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    cache: &SqliteCache,
) -> io::Result<()> {
    let mut state = AppState::new();
    let mut data = DashboardData::load(cache);
    let mut last_refresh = Instant::now();
    let refresh_interval = Duration::from_secs(2);

    loop {
        // Draw UI
        terminal.draw(|frame| {
            draw_ui(frame, &state, &data);
        })?;

        // Poll for events with a timeout so we can auto-refresh
        let timeout = refresh_interval
            .checked_sub(last_refresh.elapsed())
            .unwrap_or(Duration::ZERO);

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let action = map_key(key);
                    if action == Action::Refresh {
                        data = DashboardData::load(cache);
                        last_refresh = Instant::now();
                    }
                    apply_action(&mut state, action);
                }
            }
        }

        // Auto-refresh every 2 seconds
        if last_refresh.elapsed() >= refresh_interval {
            data = DashboardData::load(cache);
            last_refresh = Instant::now();
        }

        if state.should_quit {
            return Ok(());
        }
    }
}

fn draw_ui(frame: &mut ratatui::Frame, state: &AppState, data: &DashboardData) {
    let size = frame.area();

    // Main layout: header, body, footer
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),   // Body
            Constraint::Length(3), // Footer
        ])
        .split(size);

    // Header
    draw_header(frame, main_chunks[0]);

    // Body: left panel (log) + right panel (stats)
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Left: recent assessments
            Constraint::Percentage(40), // Right: stats
        ])
        .split(main_chunks[1]);

    // Left panel: recent assessments
    draw_log_panel(frame, body_chunks[0], &data.log_entries, state);

    // Right panel: stats
    draw_stats_panel(frame, body_chunks[1], &data.stats, state);

    // Footer: keybindings
    draw_footer(frame, main_chunks[2]);
}

fn draw_header(frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
    let version = env!("CARGO_PKG_VERSION");
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " Bark ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("v{}", version),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("  "),
        Span::styled(
            "AI-Powered Risk Assessment Dashboard",
            Style::default().fg(Color::White),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(header, area);
}

fn draw_log_panel(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    entries: &[LogEntry],
    state: &AppState,
) {
    let is_active = state.active_panel == 0;
    let border_style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let items: Vec<ListItem> = entries
        .iter()
        .skip(state.scroll_offset)
        .map(|entry| {
            let risk_color = match entry.risk_level {
                RiskLevel::Low => Color::Green,
                RiskLevel::Medium => Color::Yellow,
                RiskLevel::High => Color::Red,
            };
            let source_color = match entry.source.as_str() {
                "FAST" => Color::Green,
                "CACHE" => Color::Cyan,
                "RULE" => Color::Magenta,
                "AI" => Color::Yellow,
                _ => Color::White,
            };
            let line = Line::from(vec![
                Span::styled(
                    format!("{} ", entry.risk_level),
                    Style::default().fg(risk_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("[{}] ", entry.source),
                    Style::default().fg(source_color),
                ),
                Span::styled(
                    format!("{} ", entry.tool_name),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{}ms ", entry.duration_ms),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    truncate_str(&entry.reason, 60),
                    Style::default().fg(Color::Gray),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let title = format!(
        " Recent Assessments ({}) ",
        entries.len()
    );
    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style),
    );
    frame.render_widget(list, area);
}

fn draw_stats_panel(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    stats: &Stats,
    state: &AppState,
) {
    let is_active = state.active_panel == 1;
    let border_style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let mut lines = Vec::new();

    // Total
    lines.push(Line::from(vec![
        Span::styled("Total: ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(
            format!("{}", stats.total),
            Style::default().fg(Color::Cyan),
        ),
    ]));
    lines.push(Line::from(""));

    // By source
    lines.push(Line::from(Span::styled(
        "By Source:",
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    )));
    for (source, count) in &stats.by_source {
        let color = match source.as_str() {
            "FAST" => Color::Green,
            "CACHE" => Color::Cyan,
            "RULE" => Color::Magenta,
            "AI" => Color::Yellow,
            _ => Color::White,
        };
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{:<10}", source), Style::default().fg(color)),
            Span::styled(format!("{}", count), Style::default().fg(Color::White)),
        ]));
    }
    lines.push(Line::from(""));

    // By risk level
    lines.push(Line::from(Span::styled(
        "By Risk Level:",
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    )));
    for (level, count) in &stats.by_level {
        let color = match level.as_str() {
            "LOW" => Color::Green,
            "MEDIUM" => Color::Yellow,
            "HIGH" => Color::Red,
            _ => Color::White,
        };
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{:<10}", level), Style::default().fg(color)),
            Span::styled(format!("{}", count), Style::default().fg(Color::White)),
        ]));
    }
    lines.push(Line::from(""));

    // Cache hit rate
    let rate_pct = stats.cache_hit_rate * 100.0;
    let rate_color = if rate_pct > 70.0 {
        Color::Green
    } else if rate_pct > 40.0 {
        Color::Yellow
    } else {
        Color::Red
    };
    lines.push(Line::from(vec![
        Span::styled(
            "Cache Hit Rate: ",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:.1}%", rate_pct),
            Style::default().fg(rate_color),
        ),
    ]));

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(" Stats ")
            .borders(Borders::ALL)
            .border_style(border_style),
    );
    frame.render_widget(paragraph, area);
}

fn draw_footer(frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" q", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" quit  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Tab", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" switch panel  ", Style::default().fg(Color::DarkGray)),
        Span::styled("j/k", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" scroll  ", Style::default().fg(Color::DarkGray)),
        Span::styled("r", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" refresh", Style::default().fg(Color::DarkGray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(footer, area);
}

/// Truncate a string to max_len characters, appending "..." if truncated.
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
