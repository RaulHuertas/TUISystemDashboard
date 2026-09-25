use std::{io, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

use crate::app::{DashboardState, SharedState};

pub async fn run_tui(state: SharedState) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = ratatui::init();

    let result = run_app(&mut terminal, state).await;

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    ratatui::restore();

    result
}

async fn run_app(terminal: &mut DefaultTerminal, state: SharedState) -> io::Result<()> {
    loop {
        let snapshot = state.read().await.clone();
        terminal.draw(|frame| render(frame, &snapshot))?;

        if event::poll(Duration::from_millis(200))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
            && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
        {
            return Ok(());
        }
    }
}

fn render(frame: &mut Frame, state: &DashboardState) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    let [left, right] = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .areas(body);

    frame.render_widget(
        Paragraph::new(state.title.clone().bold())
            .block(Block::default().borders(Borders::ALL).title("Title"))
            .style(Style::default().fg(Color::Cyan)),
        header,
    );

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(format!("Status: {}", state.status)),
            Line::from(format!("Uptime: {}s", state.uptime_seconds)),
            Line::from(format!("Refreshes: {}", state.refresh_count)),
        ])
        .block(Block::default().borders(Borders::ALL).title("Overview")),
        left,
    );

    frame.render_widget(
        Paragraph::new(vec![
            Line::from("HTTP endpoints:"),
            Line::from(format!("- GET http://{}/", state.server_addr)),
            Line::from(format!("- GET http://{}/health", state.server_addr)),
            Line::from(format!("- GET http://{}/status", state.server_addr)),
        ])
        .block(Block::default().borders(Borders::ALL).title("Axum API")),
        right,
    );

    frame.render_widget(
        Paragraph::new("Press q or Esc to quit")
            .block(Block::default().borders(Borders::ALL).title("Controls")),
        footer,
    );
}
