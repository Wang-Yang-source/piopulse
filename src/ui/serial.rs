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
    // Horizontal Split: Console Terminal (75%) and Control Sidebar (25%)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(area);

    draw_console_panel(f, app, chunks[0]);
    draw_settings_panel(f, app, chunks[1]);
}

fn draw_console_panel(f: &mut Frame, app: &App, area: Rect) {
    // Vertical Split: Console Logs (Min 5) and Input Area (Length 3)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(area);

    let active_port = app
        .get_selected_port()
        .unwrap_or_else(|| "NONE".to_string());

    let lang = &app.tool_config.language;
    // 1. Receive Console block
    let rx_title = tr("serial_rx_title", lang).replace("{}", &active_port);
    let console_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border_focus))
        .title(Span::styled(
            rx_title,
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ));

    let inner_area = console_block.inner(chunks[0]);
    f.render_widget(console_block, chunks[0]);

    // Filter and wrap logs
    let max_width = inner_area.width.saturating_sub(2) as usize;
    let mut wrapped_lines: Vec<Line> = Vec::new();

    // Filter logs that belong to the active port or system messages
    for log in &app.logs {
        let belongs_to_port = log.contains(&format!("[{}]", active_port));
        let is_system =
            log.contains("System") || log.contains("PlatformIO") || log.starts_with("---");

        if belongs_to_port || is_system {
            let fg_color = if log.contains("[TX]") {
                mocha::SAPPHIRE
            } else if log.contains("FAILED") || log.contains("failed") || log.contains("Error") {
                CATPPUCCIN_MOCHA.danger
            } else if log.contains("SUCCESS") || log.contains("PASSED") || log.contains("Success") {
                CATPPUCCIN_MOCHA.success
            } else {
                CATPPUCCIN_MOCHA.text
            };

            // Remove timestamps and port prefixes for cleaner debugger view
            let mut clean_log = log.clone();
            if let Some(pos) = clean_log.find(']') {
                // If it is like [21:05:00] [/dev/ttyUSB0] message
                if clean_log.starts_with('[') {
                    // Try to find the second bracket
                    if let Some(pos2) = clean_log[pos + 1..].find(']') {
                        clean_log = clean_log[pos + 1 + pos2 + 1..].trim().to_string();
                    } else {
                        clean_log = clean_log[pos + 1..].trim().to_string();
                    }
                }
            }

            let sub_lines = wrap_log_line(&clean_log, max_width);
            for sub_line in sub_lines {
                wrapped_lines.push(Line::from(Span::styled(
                    sub_line,
                    Style::default().fg(fg_color),
                )));
            }
        }
    }

    let height = inner_area.height as usize;
    let logs_to_draw = if wrapped_lines.len() > height {
        let start = wrapped_lines.len() - height;
        wrapped_lines[start..].to_vec()
    } else {
        wrapped_lines
    };

    let paragraph = Paragraph::new(logs_to_draw)
        .block(Block::default())
        .style(Style::default().bg(mocha::MANTLE));
    f.render_widget(paragraph, inner_area);

    // 2. Input Sending block
    let lang = &app.tool_config.language;
    let (input_border, input_title, input_title_style) = if app.serial_is_typing {
        (
            CATPPUCCIN_MOCHA.success,
            tr("serial_typing_mode", lang),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.success)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (
            CATPPUCCIN_MOCHA.border,
            tr("serial_send_console", lang),
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        )
    };

    let send_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(input_border))
        .title(Span::styled(input_title, input_title_style));

    let display_text = if app.serial_is_typing {
        format!("{}_", app.serial_send_buffer)
    } else if app.serial_send_buffer.is_empty() {
        tr("serial_placeholder", lang).to_string()
    } else {
        app.serial_send_buffer.clone()
    };

    let text_style = if app.serial_send_buffer.is_empty() && !app.serial_is_typing {
        Style::default()
            .fg(CATPPUCCIN_MOCHA.text_muted)
            .add_modifier(Modifier::ITALIC)
    } else {
        Style::default().fg(CATPPUCCIN_MOCHA.text)
    };

    let input_p = Paragraph::new(Span::styled(display_text, text_style))
        .block(send_block)
        .style(Style::default().bg(mocha::MANTLE));
    f.render_widget(input_p, chunks[1]);
}

fn draw_settings_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Port Info & Baud
            Constraint::Length(10), // Toggles / Settings
            Constraint::Length(8),  // Timeline / Parser Summary
            Constraint::Min(5),     // Quick Command Templates
        ])
        .split(area);

    app.layout_zones.serial_port_info = chunks[0];
    app.layout_zones.serial_options = chunks[1];
    app.layout_zones.serial_quick_commands = chunks[3];

    let lang = &app.tool_config.language;
    // 1. Connection Config info
    let active_port = app
        .get_selected_port()
        .unwrap_or_else(|| "NONE".to_string());
    let info_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
        .title(Span::styled(
            tr("serial_port_info", lang),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    let info_row_style = |idx| {
        if app.hover_serial_port_info == Some(idx) {
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .bg(CATPPUCCIN_MOCHA.selection_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        }
    };

    let info_text = vec![
        Line::from(vec![
            Span::styled(
                tr("serial_port", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                active_port,
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.text)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
        .style(info_row_style(0)),
        Line::from(vec![
            Span::styled(
                tr("serial_baud", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                format!("{} bps", app.serial_baud_rate),
                Style::default().fg(CATPPUCCIN_MOCHA.success),
            ),
        ])
        .style(info_row_style(1)),
        Line::from(vec![
            Span::styled(
                tr("serial_bits", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled("8-N-1", Style::default().fg(CATPPUCCIN_MOCHA.text)),
        ]),
    ];

    f.render_widget(Paragraph::new(info_text).block(info_block), chunks[0]);

    // 2. Toggles
    let toggles_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
        .title(Span::styled(
            tr("serial_options_title", lang),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    let scroll_check = if app.serial_auto_scroll { "[X]" } else { "[ ]" };
    let crlf_check = if app.serial_add_newline { "[X]" } else { "[ ]" };
    let rx_hex = if app.serial_hex_mode_rx { "[X]" } else { "[ ]" };
    let tx_hex = if app.serial_hex_mode_tx { "[X]" } else { "[ ]" };
    let recording = if app.serial_recording { "[X]" } else { "[ ]" };
    let replaying = if app.serial_playback_active {
        "[X]"
    } else {
        "[ ]"
    };
    let option_row_style = |idx| {
        if app.hover_serial_option == Some(idx) {
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .bg(CATPPUCCIN_MOCHA.selection_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        }
    };

    let toggles_text = vec![
        Line::from(vec![
            Span::styled(
                format!("{} ", scroll_check),
                Style::default().fg(CATPPUCCIN_MOCHA.accent),
            ),
            Span::styled(
                tr("serial_auto_scroll", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            ),
            Span::styled("[s]", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
        ])
        .style(option_row_style(0)),
        Line::from(vec![
            Span::styled(
                format!("{} ", crlf_check),
                Style::default().fg(CATPPUCCIN_MOCHA.accent),
            ),
            Span::styled(
                tr("serial_send_newline", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            ),
            Span::styled("[n]", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
        ])
        .style(option_row_style(1)),
        Line::from(vec![
            Span::styled(
                format!("{} ", rx_hex),
                Style::default().fg(CATPPUCCIN_MOCHA.accent),
            ),
            Span::styled(
                tr("serial_hex_display", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            ),
            Span::styled("[h]", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
        ])
        .style(option_row_style(2)),
        Line::from(vec![
            Span::styled(
                format!("{} ", tx_hex),
                Style::default().fg(CATPPUCCIN_MOCHA.accent),
            ),
            Span::styled(
                tr("serial_hex_sending", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            ),
            Span::styled("[t]", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
        ])
        .style(option_row_style(3)),
        Line::from(vec![
            Span::styled(
                format!("{} ", recording),
                Style::default().fg(CATPPUCCIN_MOCHA.accent),
            ),
            Span::styled(
                if lang == "zh" {
                    "时间线录制 "
                } else {
                    "Timeline Record "
                },
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            ),
            Span::styled("[r]", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
        ])
        .style(option_row_style(4)),
        Line::from(vec![
            Span::styled(
                format!("{} ", replaying),
                Style::default().fg(CATPPUCCIN_MOCHA.accent),
            ),
            Span::styled(
                if lang == "zh" {
                    "回放并解析 "
                } else {
                    "Replay Parse "
                },
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            ),
            Span::styled("[y]", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
        ])
        .style(option_row_style(5)),
    ];

    f.render_widget(Paragraph::new(toggles_text).block(toggles_block), chunks[1]);

    draw_analysis_panel(f, app, chunks[2]);

    // 3. Quick Command Templates
    let quick_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
        .title(Span::styled(
            tr("serial_quick_commands", lang),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    let rows = vec![
        // Basic AT Commands (Blue)
        Row::new(vec![
            Cell::from(Span::styled(
                "AT",
                Style::default()
                    .fg(mocha::BLUE)
                    .add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                tr("serial_ping_desc", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            )),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                "AT+GMR",
                Style::default()
                    .fg(mocha::BLUE)
                    .add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                tr("serial_version_desc", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            )),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                "ATI",
                Style::default()
                    .fg(mocha::BLUE)
                    .add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                tr("serial_product_desc", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            )),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                "ATE1",
                Style::default()
                    .fg(mocha::BLUE)
                    .add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                tr("serial_echo_on_desc", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            )),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                "ATE0",
                Style::default()
                    .fg(mocha::BLUE)
                    .add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                tr("serial_echo_off_desc", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            )),
        ]),
        // Network / Signal (Teal)
        Row::new(vec![
            Cell::from(Span::styled(
                "AT+CSQ",
                Style::default()
                    .fg(mocha::TEAL)
                    .add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                tr("serial_signal_desc", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            )),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                "AT+CIFSR",
                Style::default()
                    .fg(mocha::TEAL)
                    .add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                tr("serial_ip_desc", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            )),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                "AT+CWMODE?",
                Style::default()
                    .fg(mocha::TEAL)
                    .add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                tr("serial_wifi_mode_desc", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            )),
        ]),
        // System / Config (Peach / Maroon / Red)
        Row::new(vec![
            Cell::from(Span::styled(
                "AT+UART_CUR?",
                Style::default()
                    .fg(mocha::PEACH)
                    .add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                tr("serial_uart_desc", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            )),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                "ATZ",
                Style::default()
                    .fg(mocha::MAROON)
                    .add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                tr("serial_reset_defaults_desc", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            )),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                "RESET",
                Style::default().fg(mocha::RED).add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                tr("serial_reboot_desc", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            )),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                "help",
                Style::default()
                    .fg(mocha::GREEN)
                    .add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                tr("serial_list_desc", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            )),
        ]),
    ];

    let rows = rows.into_iter().enumerate().map(|(idx, row)| {
        if app.hover_serial_quick_command == Some(idx) {
            row.style(
                Style::default()
                    .bg(CATPPUCCIN_MOCHA.selection_bg)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            row
        }
    });

    let table = Table::new(
        rows,
        [Constraint::Percentage(40), Constraint::Percentage(60)],
    )
    .block(quick_block)
    .header(
        Row::new(vec![tr("serial_cmd", lang), tr("serial_desc", lang)]).style(
            Style::default()
                .bg(mocha::SURFACE0)
                .add_modifier(Modifier::BOLD),
        ),
    );

    f.render_widget(table, chunks[3]);
}

fn draw_analysis_panel(f: &mut Frame, app: &App, area: Rect) {
    let lang = &app.tool_config.language;
    let summary = &app.serial_parse_summary;
    let last_hex = if summary.last_hex.is_empty() {
        "-"
    } else {
        summary.last_hex.as_str()
    };
    let last_text = if summary.last_text.is_empty() {
        "-"
    } else {
        summary.last_text.as_str()
    };
    let playback = if app.serial_playback_active {
        format!(
            "{}/{}",
            app.serial_playback_cursor,
            app.serial_timeline.len()
        )
    } else {
        "-".to_string()
    };
    let recorded_ms = app
        .serial_timeline
        .last()
        .map(|entry| entry.offset_ms)
        .unwrap_or_default();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
        .title(Span::styled(
            if lang == "zh" {
                " 时间线 / 解析 "
            } else {
                " Timeline / Parser "
            },
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    let lines = vec![
        Line::from(vec![
            Span::styled("RX ", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
            Span::styled(
                format!("{}f/{}B", summary.rx_frames, summary.rx_bytes),
                Style::default().fg(CATPPUCCIN_MOCHA.success),
            ),
            Span::raw("  "),
            Span::styled("TX ", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
            Span::styled(
                format!("{}f/{}B", summary.tx_frames, summary.tx_bytes),
                Style::default().fg(CATPPUCCIN_MOCHA.info),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                if lang == "zh" { "文本 " } else { "Text " },
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                summary.text_lines.to_string(),
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            ),
            Span::raw("  "),
            Span::styled(
                if lang == "zh" {
                    "数值帧 "
                } else {
                    "Numeric "
                },
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                summary.numeric_frames.to_string(),
                Style::default().fg(CATPPUCCIN_MOCHA.accent),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                if lang == "zh" { "录制 " } else { "Recorded " },
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                format!("{} frames", app.serial_timeline.len()),
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            ),
            Span::raw("  "),
            Span::styled("T ", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
            Span::styled(
                format!("{}ms", recorded_ms),
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            ),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                if lang == "zh" { "回放 " } else { "Replay " },
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(playback, Style::default().fg(CATPPUCCIN_MOCHA.warning)),
        ]),
        Line::from(vec![
            Span::styled("HEX ", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
            Span::styled(
                truncate_for_panel(last_hex, area.width.saturating_sub(7) as usize),
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                if lang == "zh" { "文本 " } else { "Line " },
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                truncate_for_panel(last_text, area.width.saturating_sub(8) as usize),
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            ),
        ]),
    ];

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn truncate_for_panel(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        value.to_string()
    } else {
        let mut truncated = value
            .chars()
            .take(max_chars.saturating_sub(1))
            .collect::<String>();
        truncated.push('…');
        truncated
    }
}

fn wrap_log_line(line: &str, max_width: usize) -> Vec<String> {
    if line.len() <= max_width {
        return vec![line.to_string()];
    }

    let mut result = Vec::new();
    let mut current = String::new();

    for word in line.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= max_width {
            current.push(' ');
            current.push_str(word);
        } else {
            result.push(current);
            current = word.to_string();
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    if result.is_empty() {
        result.push(line.to_string());
    }

    result
}
