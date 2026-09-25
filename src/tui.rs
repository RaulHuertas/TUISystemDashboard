use std::{io, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Gauge, Paragraph},
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

    let [left, right] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(body);

    frame.render_widget(
        Paragraph::new(state.title.clone().bold())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("www.nutylabs.com"),
            )
            .style(Style::default().fg(Color::Cyan)),
        header,
    );

    let mut overview_lines = vec![
        Line::from(format!("Uptime: {}s", state.uptime_seconds)),
        Line::from(""),
    ];

    if state.network_interfaces.is_empty() {
        overview_lines.push(Line::from("No network interfaces found"));
    } else {
        for interface in &state.network_interfaces {
            let ips = if interface.ip_addresses.is_empty() {
                "-".to_string()
            } else {
                interface.ip_addresses.join(", ")
            };

            let mac = interface.mac_address.as_deref().unwrap_or("-");

            overview_lines.push(Line::from(interface.name.clone().bold()));
            overview_lines.push(Line::from(format!("  IPs: {ips}")));
            overview_lines.push(Line::from(format!("  MAC: {mac}")));
            overview_lines.push(Line::from(format!(
                "  RX: {} bytes / {} packets",
                interface.rx_bytes, interface.rx_packets
            )));
            overview_lines.push(Line::from(format!(
                "  TX: {} bytes / {} packets",
                interface.tx_bytes, interface.tx_packets
            )));
            overview_lines.push(Line::from(""));
        }
    }

    frame.render_widget(
        Paragraph::new(overview_lines)
            .block(Block::default().borders(Borders::ALL).title("Overview")),
        left,
    );

    let system_load_block = Block::default().borders(Borders::ALL).title("System load");
    let system_load_area = system_load_block.inner(right);
    frame.render_widget(system_load_block, right);

    let [memory_area, cpu_area] = Layout::vertical([Constraint::Length(5), Constraint::Min(6)])
        .margin(1)
        .areas(system_load_area);

    let memory_ratio = if state.memory_total_bytes == 0 {
        0.0
    } else {
        state.memory_used_bytes as f64 / state.memory_total_bytes as f64
    };

    frame.render_widget(
        Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Memory"))
            .gauge_style(Style::default().fg(Color::LightMagenta))
            .ratio(memory_ratio.clamp(0.0, 1.0))
            .label(Span::raw(format!(
                "{} / {} ({:.0}%)",
                format_bytes(state.memory_used_bytes),
                format_bytes(state.memory_total_bytes),
                memory_ratio * 100.0
            )))
            .use_unicode(true),
        memory_area,
    );

    let cpu_bars: Vec<Bar> = state
        .cpu_usages
        .iter()
        .enumerate()
        .map(|(index, usage)| {
            Bar::default()
                .value(*usage)
                .label(Line::from(format!("CPU{index}")))
                .text_value(format!("{usage:>3}%"))
                .style(Style::default().fg(Color::Cyan))
                .value_style(Style::default().fg(Color::Black).bg(Color::Cyan).bold())
        })
        .collect();

    frame.render_widget(
        BarChart::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Per-CPU usage"),
            )
            .direction(Direction::Horizontal)
            .data(BarGroup::default().bars(&cpu_bars))
            .max(100)
            .bar_width(1)
            .bar_gap(0)
            .value_style(Style::default().fg(Color::Black).bg(Color::Cyan).bold())
            .label_style(Style::default().fg(Color::White)),
        cpu_area,
    );

    frame.render_widget(
        Paragraph::new("Press q or Esc to quit")
            .block(Block::default().borders(Borders::ALL).title("Controls")),
        footer,
    );
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];

    let mut value = bytes as f64;
    let mut unit_index = 0;

    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    format!("{value:.1} {}", UNITS[unit_index])
}
