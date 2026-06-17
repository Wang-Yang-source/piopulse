use crate::app::{App, SerialNotice, SerialNoticeKind};
use crate::ui::theme::{CATPPUCCIN_MOCHA, mocha};
use crate::ui::tr;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::canvas::Canvas,
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
};
use unicode_width::UnicodeWidthStr;

const SERIAL_OPTION_ROWS: usize = 4;
const SERIAL_OPTION_COUNT: usize = SERIAL_OPTION_ROWS * 2;

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
    let rx_title = if let Some(notice) = &app.serial_notice {
        format!(
            "{}  {} {}",
            tr("serial_rx_title", lang).replace("{}", &active_port),
            serial_notice_spinner(notice),
            notice.message
        )
    } else {
        tr("serial_rx_title", lang).replace("{}", &active_port)
    };
    let console_border = app
        .serial_notice
        .as_ref()
        .map(serial_notice_color)
        .unwrap_or(CATPPUCCIN_MOCHA.border_focus);
    let console_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(console_border))
        .title(Span::styled(
            rx_title,
            Style::default()
                .fg(console_border)
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

    if active_port == "NONE" {
        let aspect = (inner_area.width as f64 / inner_area.height as f64) * 0.5;
        let x_limit = 20.0 * aspect;

        let canvas = Canvas::default()
            .block(Block::default())
            .x_bounds([-x_limit, x_limit])
            .y_bounds([-10.0, 10.0])
            .paint(|ctx| {
                let t = app.anim_tick as f64;

                // 1. Draw subtle background oscilloscope grid
                // Vertical grid lines
                let mut x_grid = -x_limit;
                while x_grid <= x_limit {
                    ctx.draw(&ratatui::widgets::canvas::Line {
                        x1: x_grid,
                        y1: -10.0,
                        x2: x_grid,
                        y2: 10.0,
                        color: CATPPUCCIN_MOCHA.border,
                    });
                    x_grid += 4.0;
                }
                // Horizontal grid lines
                let mut y_grid = -8.0;
                while y_grid <= 8.0 {
                    ctx.draw(&ratatui::widgets::canvas::Line {
                        x1: -x_limit,
                        y1: y_grid,
                        x2: x_limit,
                        y2: y_grid,
                        color: CATPPUCCIN_MOCHA.border,
                    });
                    y_grid += 4.0;
                }

                // 2. Draw the scrolling ECG pulse waveform
                let ecg_wave = |phase: f64| -> f64 {
                    let mut p = (phase % (2.0 * std::f64::consts::PI)) / (2.0 * std::f64::consts::PI);
                    if p < 0.0 {
                        p += 1.0;
                    }
                    let gauss = |p_val: f64, center: f64, width: f64, amp: f64| -> f64 {
                        amp * (-((p_val - center) / width).powi(2)).exp()
                    };

                    let mut y = 0.0;
                    y += gauss(p, 0.2, 0.03, 1.2);   // P wave (small bump)
                    y += gauss(p, 0.35, 0.015, -1.0); // Q wave (small dip)
                    y += gauss(p, 0.4, 0.012, 7.5);   // R wave (high spike)
                    y += gauss(p, 0.45, 0.018, -2.5); // S wave (deep dip)
                    y += gauss(p, 0.6, 0.06, 1.8);    // T wave (medium bump)
                    y
                };

                let num_segments = 160;
                let mut prev_point: Option<(f64, f64)> = None;
                for s in 0..=num_segments {
                    let alpha = s as f64 / num_segments as f64;
                    let x = -x_limit + alpha * (2.0 * x_limit);
                    
                    // Wave scrolls right-to-left
                    let phase = (x * 0.25) - (t * 0.15);
                    let y = ecg_wave(phase);

                    if let Some((px, py)) = prev_point {
                        ctx.draw(&ratatui::widgets::canvas::Line {
                            x1: px,
                            y1: py,
                            x2: x,
                            y2: y,
                            color: CATPPUCCIN_MOCHA.success, // Pulse Green
                        });
                    }
                    prev_point = Some((x, y));
                }

                // 3. Draw hint message
                let msg = if lang == "zh" {
                    " 请在右侧菜单选择串口以开始监视... "
                } else {
                    " Select a serial port on the right to start monitoring... "
                };
                
                // Draw a beautiful centered box for the message to overlay the oscilloscope grid cleanly
                ctx.print(
                    -x_limit + 1.0,
                    -9.0,
                    Span::styled(
                        msg,
                        Style::default()
                            .fg(CATPPUCCIN_MOCHA.text)
                            .bg(mocha::MANTLE)
                            .add_modifier(Modifier::BOLD),
                    ),
                );
            });

        f.render_widget(canvas, inner_area);
    } else {
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
    }

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
            Constraint::Length(4), // Port Info & Baud
            Constraint::Length(6), // Toggles / Settings
            Constraint::Length(6), // Timeline / Parser Summary
            Constraint::Min(5),    // Quick Command Templates
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
    let port_border = app
        .serial_notice
        .as_ref()
        .map(serial_notice_color)
        .unwrap_or(CATPPUCCIN_MOCHA.border);
    let info_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(port_border))
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

    let status_line = if let Some(notice) = &app.serial_notice {
        let style = serial_notice_style(notice);
        Line::from(vec![
            Span::styled(
                format!("{} ", serial_notice_spinner(notice)),
                Style::default()
                    .fg(serial_notice_color(notice))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                truncate_for_panel(&notice.message, chunks[0].width.saturating_sub(5) as usize),
                style,
            ),
        ])
    } else {
        let pending = app.serial_pending_monitors.len();
        let open = app.serial_tx_senders.len();
        let state = if app.serial_monitor_enabled {
            if pending > 0 {
                if lang == "zh" { "打开中" } else { "opening" }
            } else if open > 0 {
                if lang == "zh" {
                    "监视中"
                } else {
                    "monitoring"
                }
            } else if lang == "zh" {
                "待连接"
            } else {
                "idle"
            }
        } else if lang == "zh" {
            "已暂停"
        } else {
            "paused"
        };
        Line::from(vec![
            Span::styled(
                if lang == "zh" { "状态 " } else { "State " },
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                format!("{}  open:{} pending:{}", state, open, pending),
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            ),
        ])
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
            Span::raw("  "),
            Span::styled(
                tr("serial_baud", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                format!("{} 8-N-1", app.serial_baud_rate),
                Style::default().fg(CATPPUCCIN_MOCHA.success),
            ),
        ])
        .style(info_row_style(0)),
        status_line.style(info_row_style(1)),
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

    let monitor_check = if app.serial_monitor_enabled {
        "[X]"
    } else {
        "[ ]"
    };
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
    let option_cell_style = |idx| {
        if app.hover_serial_option == Some(idx) {
            Style::default()
                .bg(CATPPUCCIN_MOCHA.selection_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        }
    };
    let left_option_width = serial_option_left_width(chunks[1]);

    let toggles_text = vec![
        compact_option_line(
            monitor_check,
            if lang == "zh" { "监视" } else { "MON" },
            "m",
            true,
            option_cell_style(0),
            scroll_check,
            if lang == "zh" { "滚动" } else { "Scroll" },
            "s",
            false,
            option_cell_style(1),
            left_option_width,
        ),
        compact_option_line(
            crlf_check,
            if lang == "zh" { "换行" } else { "NL" },
            "n",
            false,
            option_cell_style(2),
            rx_hex,
            if lang == "zh" { "RX HEX" } else { "RX HEX" },
            "h",
            false,
            option_cell_style(3),
            left_option_width,
        ),
        compact_option_line(
            tx_hex,
            if lang == "zh" { "TX HEX" } else { "TX HEX" },
            "t",
            false,
            option_cell_style(4),
            recording,
            if lang == "zh" { "录制" } else { "REC" },
            "r",
            false,
            option_cell_style(5),
            left_option_width,
        ),
        compact_option_line(
            replaying,
            if lang == "zh" { "回放" } else { "PLAY" },
            "y",
            false,
            option_cell_style(6),
            "[ ]",
            if lang == "zh" { "波特" } else { "Baud" },
            "b",
            false,
            option_cell_style(7),
            left_option_width,
        ),
    ];

    f.render_widget(Paragraph::new(toggles_text).block(toggles_block), chunks[1]);

    draw_analysis_panel(f, app, chunks[2]);

    // 3. Quick Command Templates
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

    let visible_quick_rows = chunks[3].height.saturating_sub(3) as usize;
    let max_quick_scroll = rows.len().saturating_sub(visible_quick_rows);
    app.serial_quick_scroll = app.serial_quick_scroll.min(max_quick_scroll);
    let quick_title = if visible_quick_rows > 0 && rows.len() > visible_quick_rows {
        let first = app.serial_quick_scroll + 1;
        let last = (app.serial_quick_scroll + visible_quick_rows).min(rows.len());
        format!(
            "{} {}-{}/{} ",
            tr("serial_quick_commands", lang),
            first,
            last,
            rows.len()
        )
    } else {
        tr("serial_quick_commands", lang).to_string()
    };

    let quick_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
        .title(Span::styled(
            quick_title,
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    let rows = rows
        .into_iter()
        .enumerate()
        .skip(app.serial_quick_scroll)
        .take(visible_quick_rows)
        .map(|(idx, row)| {
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

fn serial_notice_spinner(notice: &SerialNotice) -> &'static str {
    const FRAMES: [&str; 4] = ["-", "\\", "|", "/"];
    let idx = ((notice.started_at.elapsed().as_millis() / 120) % FRAMES.len() as u128) as usize;
    FRAMES[idx]
}

fn serial_notice_color(notice: &SerialNotice) -> ratatui::style::Color {
    match notice.kind {
        SerialNoticeKind::Info => CATPPUCCIN_MOCHA.info,
        SerialNoticeKind::Success => CATPPUCCIN_MOCHA.success,
        SerialNoticeKind::Warning => CATPPUCCIN_MOCHA.warning,
    }
}

fn serial_notice_style(notice: &SerialNotice) -> Style {
    Style::default()
        .fg(serial_notice_color(notice))
        .add_modifier(Modifier::BOLD)
}

fn compact_option_line<'a>(
    left_check: &'a str,
    left_label: &'a str,
    left_key: &'a str,
    left_special: bool,
    left_style: Style,
    right_check: &'a str,
    right_label: &'a str,
    right_key: &'a str,
    right_special: bool,
    right_style: Style,
    left_cell_width: usize,
) -> Line<'a> {
    let mut spans = option_cell_spans(left_check, left_label, left_key, left_special, left_style);
    let left_width = option_cell_display_width(left_check, left_label, left_key);
    let padding = left_cell_width.saturating_sub(left_width);
    spans.push(Span::styled(" ".repeat(padding), left_style));
    spans.extend(option_cell_spans(
        right_check,
        right_label,
        right_key,
        right_special,
        right_style,
    ));
    Line::from(spans)
}

fn option_cell_spans<'a>(
    check: &'a str,
    label: &'a str,
    key: &'a str,
    special: bool,
    cell_style: Style,
) -> Vec<Span<'a>> {
    let accent = if special {
        CATPPUCCIN_MOCHA.warning
    } else {
        CATPPUCCIN_MOCHA.accent
    };
    let label_color = if special {
        CATPPUCCIN_MOCHA.warning
    } else {
        CATPPUCCIN_MOCHA.text
    };

    vec![
        Span::styled(format!("{} ", check), cell_style.fg(accent)),
        Span::styled(label, cell_style.fg(label_color)),
        Span::styled(
            format!("[{}]", key),
            cell_style.fg(CATPPUCCIN_MOCHA.text_muted),
        ),
    ]
}

fn option_cell_display_width(check: &str, label: &str, key: &str) -> usize {
    UnicodeWidthStr::width(check)
        + 1
        + UnicodeWidthStr::width(label)
        + UnicodeWidthStr::width(format!("[{}]", key).as_str())
}

fn serial_option_left_width(area: Rect) -> usize {
    let inner_width = area.width.saturating_sub(2) as usize;
    inner_width.div_ceil(2)
}

pub fn serial_option_at(area: Rect, col: u16, row: u16) -> Option<usize> {
    let inner_x = area.x.saturating_add(1);
    let inner_y = area.y.saturating_add(1);
    let inner_width = area.width.saturating_sub(2);
    let inner_height = area.height.saturating_sub(2);

    if inner_width == 0
        || inner_height == 0
        || col < inner_x
        || col >= inner_x.saturating_add(inner_width)
        || row < inner_y
        || row >= inner_y.saturating_add(inner_height)
    {
        return None;
    }

    let row_idx = row.saturating_sub(inner_y) as usize;
    if row_idx >= SERIAL_OPTION_ROWS {
        return None;
    }

    let left_width = serial_option_left_width(area) as u16;
    let relative_col = col.saturating_sub(inner_x);
    let col_idx = if relative_col < left_width { 0 } else { 1 };
    let idx = row_idx.saturating_mul(2).saturating_add(col_idx);
    (idx < SERIAL_OPTION_COUNT).then_some(idx)
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
