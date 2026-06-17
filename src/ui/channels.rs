use crate::app::{App, Channel};
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
            Constraint::Length(4), // Production Summary
            Constraint::Min(5),    // Table
        ])
        .split(area);

    app.layout_zones.flash_summary = chunks[0];
    app.layout_zones.flash_device_table = chunks[1];

    draw_summary_dashboard(f, app, chunks[0]);
    draw_device_table(f, app, chunks[1]);
}

fn draw_empty_state(f: &mut Frame, app: &mut App, area: Rect) {
    let centered_area = if area.width < 76 || area.height < 14 {
        area
    } else {
        crate::ui::center_rect(65, 12, area)
    };
    app.layout_zones.flash_empty_state = centered_area;
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

    let compact = area.height < 14 || area.width < 76;
    let mut info_lines = vec![
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
    ];

    if !compact {
        info_lines.extend([
            Line::from(vec![
                Span::styled(
                    if lang == "zh" {
                        "   校验/空片: "
                    } else {
                        "   Verify/Blank: "
                    },
                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                ),
                Span::styled(
                    format!(
                        "[{} / {}]",
                        app.config.verify_method,
                        enabled_label(app.config.blank_check, lang)
                    ),
                    Style::default().fg(CATPPUCCIN_MOCHA.success),
                ),
                Span::styled(
                    if lang == "zh" {
                        "   SN/批次: "
                    } else {
                        "   SN/Lot: "
                    },
                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                ),
                Span::styled(
                    format!("[{} / {}]", app.config.sn_prefix, app.config.lot_code),
                    Style::default().fg(CATPPUCCIN_MOCHA.primary),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    if lang == "zh" {
                        "   自动感应烧录: "
                    } else {
                        "   Auto-Flash: "
                    },
                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                ),
                Span::styled(
                    if app.auto_flash {
                        if lang == "zh" {
                            "启用 [按A键禁用]"
                        } else {
                            "ENABLED [Press A to disable]"
                        }
                    } else {
                        if lang == "zh" {
                            "禁用 [按A键启用]"
                        } else {
                            "DISABLED [Press A to enable]"
                        }
                    },
                    Style::default()
                        .fg(if app.auto_flash {
                            CATPPUCCIN_MOCHA.success
                        } else {
                            CATPPUCCIN_MOCHA.text_disabled
                        })
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
        ]);
    }

    info_lines.extend([
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
    ]);

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

    let mut summary_lines = vec![Line::from(vec![
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
        Span::raw(" | "),
        Span::raw(if lang == "zh" {
            "自动感应: "
        } else {
            "Auto-Flash: "
        }),
        Span::styled(
            if app.auto_flash { "ON" } else { "OFF" },
            Style::default()
                .fg(if app.auto_flash {
                    CATPPUCCIN_MOCHA.success
                } else {
                    CATPPUCCIN_MOCHA.text_muted
                })
                .add_modifier(Modifier::BOLD),
        ),
    ])];

    summary_lines.push(Line::from(vec![
        Span::raw("  "),
        flash_action_span(0, app.hover_flash_action, lang),
        Span::raw("  "),
        flash_action_span(1, app.hover_flash_action, lang),
        Span::raw("  "),
        flash_action_span(2, app.hover_flash_action, lang),
        Span::raw("  "),
        flash_action_span(3, app.hover_flash_action, lang),
    ]));

    summary_lines.push(Line::from(vec![
        Span::styled(
            if lang == "zh" {
                "  策略 "
            } else {
                "  Policy "
            },
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ),
        Span::styled(
            format!(
                "verify:{} blank:{} erase:{} lock:{} lot:{}",
                app.config.verify_method,
                enabled_label(app.config.blank_check, lang),
                app.config.erase_mode,
                enabled_label(app.config.lock_after_flash, lang),
                app.config.lot_code
            ),
            Style::default().fg(CATPPUCCIN_MOCHA.text),
        ),
    ]));

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

    let paragraph = Paragraph::new(summary_lines)
        .block(block)
        .alignment(ratatui::layout::Alignment::Left)
        .style(Style::default().bg(mocha::MANTLE));

    f.render_widget(paragraph, area);
}

fn draw_device_table(f: &mut Frame, app: &mut App, area: Rect) {
    let lang = &app.tool_config.language;
    let compact = area.width < 120;
    let mut rows = Vec::new();
    let visible_rows = area.height.saturating_sub(3) as usize;
    let max_scroll = app.channels.len().saturating_sub(visible_rows);
    app.flash_table_scroll = app.flash_table_scroll.min(max_scroll);

    for (idx, channel) in app
        .channels
        .iter()
        .enumerate()
        .skip(app.flash_table_scroll)
        .take(visible_rows)
    {
        let is_selected = idx == app.selected_channel_idx;
        let is_hovered = app.hover_flash_row == Some(idx);

        let port_prefix = if is_selected { "> " } else { "  " };
        let port_text = format!("{}{}", port_prefix, channel.port);
        let port_style = if is_selected {
            Style::default()
                .fg(CATPPUCCIN_MOCHA.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(CATPPUCCIN_MOCHA.text)
        };

        let chip_text = channel
            .chip
            .as_deref()
            .unwrap_or_else(|| tr("flash_detecting", lang));
        let (usable_text, usable_style) = flash_usable_status(channel, lang);
        let mac_text = channel.mac.as_deref().unwrap_or("XX:XX:XX:XX:XX:XX");
        let sn_text = channel.serial_number.as_deref().unwrap_or("-");
        let trace_text = channel.trace_id.as_deref().unwrap_or("-");

        let (status_text, status_style) = if channel.finished {
            if channel.success {
                (
                    tr("flash_status_success", lang).to_string(),
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.success)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                let err_msg = channel
                    .error
                    .as_deref()
                    .unwrap_or_else(|| if lang == "zh" { "未知" } else { "Unknown" });
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
                    "Blank Check" => "空片检查",
                    "Erase Plan" => "擦除规划",
                    "Functional Test" => "功能测试",
                    s => s,
                }
            } else {
                &channel.status
            };
            (
                status_disp.to_string(),
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

        let row_style = if is_selected {
            Style::default().bg(mocha::SURFACE0)
        } else if is_hovered {
            Style::default().bg(CATPPUCCIN_MOCHA.selection_bg)
        } else if idx % 2 == 0 {
            Style::default().bg(mocha::MANTLE)
        } else {
            Style::default().bg(mocha::BASE)
        };

        let cells = if compact {
            vec![
                Cell::from(Span::styled(port_text, port_style)),
                Cell::from(Span::styled(usable_text, usable_style)),
                Cell::from(Span::raw(chip_text)),
                Cell::from(Span::styled(status_text, status_style)),
                Cell::from(Span::styled(progress_text, progress_style)),
                Cell::from(Span::styled(
                    channel.qa_result.clone(),
                    qa_style(&channel.qa_result),
                )),
            ]
        } else {
            vec![
                Cell::from(Span::styled(port_text, port_style)),
                Cell::from(Span::styled(usable_text, usable_style)),
                Cell::from(Span::raw(chip_text)),
                Cell::from(Span::raw(sn_text)),
                Cell::from(Span::raw(mac_text)),
                Cell::from(Span::styled(status_text, status_style)),
                Cell::from(Span::styled(progress_text, progress_style)),
                Cell::from(Span::raw(format_bytes(channel.bytes_written))),
                Cell::from(Span::styled(
                    channel.qa_result.clone(),
                    qa_style(&channel.qa_result),
                )),
                Cell::from(Span::raw(channel.security_state.clone())),
                Cell::from(Span::raw(trace_text)),
            ]
        };

        rows.push(Row::new(cells).style(row_style));
    }

    let headers = if compact && lang == "zh" {
        vec!["端口", "可用", "芯片", "流程", "进度", "QA"]
    } else if compact {
        vec!["Port", "Use", "Chip", "Flow", "Progress", "QA"]
    } else if lang == "zh" {
        vec![
            "端口",
            "可用性",
            "目标芯片",
            "SN",
            "MAC 地址",
            "流程",
            "进度",
            "写入",
            "QA",
            "安全",
            "追溯",
        ]
    } else {
        vec![
            "Port",
            "Usable",
            "Target Chip",
            "SN",
            "MAC Address",
            "Flow",
            "Progress",
            "Bytes",
            "QA",
            "Security",
            "Trace",
        ]
    };

    let widths: Vec<Constraint> = if compact {
        vec![
            Constraint::Length(14),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Min(16),
            Constraint::Length(16),
            Constraint::Length(10),
        ]
    } else {
        vec![
            Constraint::Length(14),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(24),
            Constraint::Length(17),
            Constraint::Length(20),
            Constraint::Length(16),
            Constraint::Length(9),
            Constraint::Length(14),
            Constraint::Length(18),
            Constraint::Min(16),
        ]
    };

    let table = Table::new(rows, widths)
        .header(
            Row::new(headers.into_iter().map(Cell::from).collect::<Vec<_>>()).style(
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
                    flash_table_title(
                        lang,
                        app.flash_table_scroll,
                        visible_rows,
                        app.channels.len(),
                    ),
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
    format!("[{}{}] {:>3}%", "=".repeat(filled), " ".repeat(empty), pct)
}

pub fn flash_action_label(idx: usize, lang: &str) -> &'static str {
    match (idx, lang == "zh") {
        (0, true) => "烧录选中",
        (0, false) => "Flash Sel",
        (1, true) => "批量",
        (1, false) => "Batch",
        (2, true) => "扫描",
        (2, false) => "Scan",
        (3, true) => "配置",
        (3, false) => "Config",
        _ => "",
    }
}

fn flash_action_span(idx: usize, hover: Option<usize>, lang: &str) -> Span<'static> {
    let style = if hover == Some(idx) {
        Style::default()
            .fg(CATPPUCCIN_MOCHA.text)
            .bg(CATPPUCCIN_MOCHA.selection_bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(CATPPUCCIN_MOCHA.accent)
            .bg(mocha::SURFACE0)
            .add_modifier(Modifier::BOLD)
    };
    Span::styled(format!(" {} ", flash_action_label(idx, lang)), style)
}

fn flash_table_title(lang: &str, scroll: usize, visible_rows: usize, total: usize) -> String {
    let base = tr("flash_devices_title", lang);
    if total > visible_rows && visible_rows > 0 {
        let first = scroll + 1;
        let last = (scroll + visible_rows).min(total);
        format!("{} {}-{}/{} ", base, first, last, total)
    } else {
        base.to_string()
    }
}

fn enabled_label(enabled: bool, lang: &str) -> &'static str {
    if enabled {
        if lang == "zh" { "启用" } else { "ON" }
    } else if lang == "zh" {
        "关闭"
    } else {
        "OFF"
    }
}

fn flash_usable_status(channel: &Channel, lang: &str) -> (&'static str, Style) {
    let description = format!(
        "{} {}",
        channel.usb_product.as_deref().unwrap_or_default(),
        channel.usb_manufacturer.as_deref().unwrap_or_default()
    )
    .to_ascii_lowercase();

    if channel.vid == Some(0x303a) || description.contains("espressif") {
        return (
            if lang == "zh" {
                "ESP 原生"
            } else {
                "ESP USB"
            },
            Style::default()
                .fg(CATPPUCCIN_MOCHA.success)
                .add_modifier(Modifier::BOLD),
        );
    }

    let known_usb_uart_vid = matches!(
        channel.vid,
        Some(0x10c4 | 0x1a86 | 0x0403 | 0x067b | 0x1b4f | 0x2341)
    );
    let known_usb_uart_text = [
        "cp210",
        "ch340",
        "ch341",
        "wch",
        "ftdi",
        "prolific",
        "usb serial",
        "usb-serial",
        "uart",
    ]
    .iter()
    .any(|needle| description.contains(needle));

    if known_usb_uart_vid || known_usb_uart_text {
        return (
            if lang == "zh" { "USB-UART" } else { "USB-UART" },
            Style::default()
                .fg(CATPPUCCIN_MOCHA.success)
                .add_modifier(Modifier::BOLD),
        );
    }

    let likely_not_target = [
        "bluetooth",
        "pico",
        "cmsis",
        "daplink",
        "stlink",
        "debug probe",
    ]
    .iter()
    .any(|needle| description.contains(needle));

    if likely_not_target {
        return (
            if lang == "zh" {
                "可能不可用"
            } else {
                "Maybe no"
            },
            Style::default().fg(CATPPUCCIN_MOCHA.danger),
        );
    }

    (
        if lang == "zh" { "未知" } else { "Unknown" },
        Style::default().fg(CATPPUCCIN_MOCHA.warning),
    )
}

fn format_bytes(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1}MB", bytes as f64 / 1024.0 / 1024.0)
    } else if bytes >= 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else if bytes == 0 {
        "-".to_string()
    } else {
        format!("{}B", bytes)
    }
}

fn qa_style(value: &str) -> Style {
    let upper = value.to_ascii_uppercase();
    if upper.starts_with("PASS") {
        Style::default()
            .fg(CATPPUCCIN_MOCHA.success)
            .add_modifier(Modifier::BOLD)
    } else if upper.starts_with("FAIL") {
        Style::default()
            .fg(CATPPUCCIN_MOCHA.danger)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(CATPPUCCIN_MOCHA.text_muted)
    }
}
