use crate::app::App;
use crate::ui::theme::{CATPPUCCIN_MOCHA, mocha};
use crate::ui::tr;
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
    let lang = &app.tool_config.language;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
        .title(Span::styled(
            tr("flash_engine_title", lang),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    let info_lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            tr("flash_no_devices", lang),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.warning)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                tr("flash_target_chip", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                format!("[{}]", app.config.chip_type),
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                tr("flash_baud_rate", lang),
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
                tr("flash_mode", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                format!("[{}]", app.config.flash_mode),
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            ),
            Span::styled(
                tr("flash_size", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                format!("[{}]", app.config.flash_size),
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            tr("flash_connect_usb", lang),
            Style::default().fg(CATPPUCCIN_MOCHA.text_disabled),
        )),
        Line::from(vec![
            Span::styled(
                tr("flash_hint", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_disabled),
            ),
            Span::styled(
                tr("flash_spacebar", lang),
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                tr("flash_rescan", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_disabled),
            ),
        ]),
    ];

    f.render_widget(Paragraph::new(info_lines).block(block), centered_area);
}

fn draw_summary_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let lang = &app.tool_config.language;
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
        Span::raw(tr("flash_devices_count", lang)),
        Span::styled(
            total.to_string(),
            Style::default()
                .fg(mocha::BLUE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(tr("flash_idle_count", lang)),
        Span::styled(
            idle.to_string(),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text_muted)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(tr("flash_flashing_count", lang)),
        Span::styled(
            flashing.to_string(),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(tr("flash_success_count", lang)),
        Span::styled(
            passed.to_string(),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.success)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(tr("flash_failed_count", lang)),
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
            tr("flash_dashboard_title", lang),
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
    let lang = &app.tool_config.language;
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

        let chip_text = channel.chip.as_deref().unwrap_or_else(|| tr("flash_detecting", lang));
        let mac_text = channel.mac.as_deref().unwrap_or("XX:XX:XX:XX:XX:XX");

        let (status_text, status_style) = if channel.finished {
            if channel.success {
                (
                    tr("flash_status_success", lang).to_string(),
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.success)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                let err_msg = channel.error.as_deref().unwrap_or_else(|| {
                    if lang == "zh" { "未知" } else { "Unknown" }
                });
                (
                    tr("flash_status_failed", lang).replace("{}", err_msg),
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.danger)
                        .add_modifier(Modifier::BOLD),
                )
            }
        } else if channel.status == "Idle" {
            (
                tr("flash_status_idle", lang).to_string(),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            )
        } else {
            let status_disp = if lang == "zh" {
                match channel.status.as_str() {
                    "Flashing" => "烧录中",
                    "Erasing" => "擦除中",
                    "Verifying" => "校验中",
                    "Connecting" => "连接中",
                    s => s,
                }
            } else {
                &channel.status
            };
            (
                format!("⟳ {}", status_disp),
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

    let headers = if lang == "zh" {
        vec!["端口", "目标芯片", "MAC 地址", "状态", "进度", "速度"]
    } else {
        vec![
            "Port",
            "Target Chip",
            "MAC Address",
            "Status",
            "Progress",
            "Speed",
        ]
    };

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
        Row::new(headers.into_iter().map(Cell::from).collect::<Vec<_>>())
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
                tr("flash_devices_title", lang),
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
