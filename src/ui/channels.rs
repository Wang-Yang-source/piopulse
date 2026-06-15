use crate::app::App;
use crate::ui::theme::{CATPPUCCIN_MOCHA, mocha};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
};

pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    if app.channels.is_empty() {
        draw_empty_state(f, app, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Summary Bar
            Constraint::Min(5),    // Table
        ])
        .split(area);

    draw_summary_dashboard(f, app, chunks[0]);
    draw_device_table(f, app, chunks[1]);
}

fn draw_empty_state(f: &mut Frame, app: &App, area: Rect) {
    let centered_area = crate::ui::center_rect(65, 12, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
        .title(Span::styled(
            " 🔋 BATCH FLASHER ENGINE ",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    let info_lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "   ⚠️  No active flashing devices detected.",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.warning)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "   Target Chip:  ",
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                format!("[{}]", app.config.chip_type),
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "   Baud Rate:  ",
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                format!("[{} bps]", app.config.baud_rate),
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "   Flash Mode:   ",
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                format!("[{}]", app.config.flash_mode),
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            ),
            Span::styled(
                "   Flash Size: ",
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                format!("[{}]", app.config.flash_size),
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "   🔌 Connect USB devices to auto-scan and begin flashing...",
            Style::default().fg(CATPPUCCIN_MOCHA.text_disabled),
        )),
        Line::from(vec![
            Span::styled(
                "   ⚡ Hint: Press ",
                Style::default().fg(CATPPUCCIN_MOCHA.text_disabled),
            ),
            Span::styled(
                "Spacebar",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " to manually re-scan ports.",
                Style::default().fg(CATPPUCCIN_MOCHA.text_disabled),
            ),
        ]),
    ];

    f.render_widget(Paragraph::new(info_lines).block(block), centered_area);
}

fn draw_summary_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let total = app.channels.len();
    let idle = app.channels.iter().filter(|c| c.status == "Idle").count();
    let flashing = app
        .channels
        .iter()
        .filter(|c| !c.finished && c.status != "Idle")
        .count();
    let passed = app
        .channels
        .iter()
        .filter(|c| c.finished && c.success)
        .count();
    let failed = app
        .channels
        .iter()
        .filter(|c| c.finished && !c.success)
        .count();

    let summary_line = Line::from(vec![
        Span::raw("  ⚡ Devices Count: "),
        Span::styled(
            total.to_string(),
            Style::default()
                .fg(mocha::BLUE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  |  ● Idle: "),
        Span::styled(
            idle.to_string(),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text_muted)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  |  ⟳ Flashing: "),
        Span::styled(
            flashing.to_string(),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  |  ✓ Success: "),
        Span::styled(
            passed.to_string(),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.success)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  |  ✗ Failed: "),
        Span::styled(
            failed.to_string(),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.danger)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
        .title(Span::styled(
            " BATCH FLASHING MONITOR DASHBOARD ",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ));

    let paragraph = Paragraph::new(summary_line)
        .block(block)
        .alignment(ratatui::layout::Alignment::Left)
        .style(Style::default().bg(mocha::MANTLE));

    f.render_widget(paragraph, area);
}

fn draw_device_table(f: &mut Frame, app: &App, area: Rect) {
    let mut rows = Vec::new();
    for (idx, channel) in app.channels.iter().enumerate() {
        let is_selected = idx == app.selected_channel_idx;

        let port_prefix = if is_selected { "> " } else { "  " };
        let port_text = format!("{}{}", port_prefix, channel.port);
        let port_style = if is_selected {
            Style::default()
                .fg(CATPPUCCIN_MOCHA.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(CATPPUCCIN_MOCHA.text)
        };

        let chip_text = channel.chip.as_deref().unwrap_or("Detecting...");
        let mac_text = channel.mac.as_deref().unwrap_or("XX:XX:XX:XX:XX:XX");

        let (status_text, status_style) = if channel.finished {
            if channel.success {
                (
                    "✓ SUCCESS".to_string(),
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.success)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                let err_msg = channel.error.as_deref().unwrap_or("Unknown");
                (
                    format!("✗ FAILED ({})", err_msg),
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.danger)
                        .add_modifier(Modifier::BOLD),
                )
            }
        } else if channel.status == "Idle" {
            (
                "● Idle".to_string(),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            )
        } else {
            (
                format!("⟳ {}", channel.status),
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.warning)
                    .add_modifier(Modifier::BOLD),
            )
        };

        let progress_text = make_progress_bar(channel.progress, 10);
        let progress_style = if channel.finished && channel.success {
            Style::default().fg(CATPPUCCIN_MOCHA.success)
        } else if channel.finished && !channel.success {
            Style::default().fg(CATPPUCCIN_MOCHA.danger)
        } else if channel.status == "Idle" {
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted)
        } else {
            Style::default().fg(CATPPUCCIN_MOCHA.warning)
        };

        let speed_text = if channel.status == "Idle" {
            "---".to_string()
        } else {
            channel.speed.clone()
        };

        let row_style = if is_selected {
            Style::default().bg(mocha::SURFACE0)
        } else if idx % 2 == 0 {
            Style::default().bg(mocha::MANTLE)
        } else {
            Style::default().bg(mocha::BASE)
        };

        rows.push(
            Row::new(vec![
                Cell::from(Span::styled(port_text, port_style)),
                Cell::from(Span::raw(chip_text)),
                Cell::from(Span::raw(mac_text)),
                Cell::from(Span::styled(status_text, status_style)),
                Cell::from(Span::styled(progress_text, progress_style)),
                Cell::from(Span::raw(speed_text)),
            ])
            .style(row_style),
        );
    }

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(25),
            Constraint::Percentage(12),
            Constraint::Percentage(8),
        ],
    )
    .header(
        Row::new(vec![
            "Port",
            "Target Chip",
            "MAC Address",
            "Status",
            "Progress",
            "Speed",
        ])
        .style(
            Style::default()
                .fg(mocha::SUBTEXT1)
                .bg(mocha::SURFACE0)
                .add_modifier(Modifier::BOLD),
        ),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
            .title(Span::styled(
                " Batch Flasher Devices ",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.text)
                    .add_modifier(Modifier::BOLD),
            )),
    );

    f.render_widget(table, area);
}

fn make_progress_bar(pct: u8, width: usize) -> String {
    let filled = (pct as usize * width) / 100;
    let empty = width - filled;
    format!("[{}{}] {:>3}%", "█".repeat(filled), "░".repeat(empty), pct)
}
