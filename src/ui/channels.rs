use crate::app::{App, Channel};
use crate::ui::theme::{CATPPUCCIN_MOCHA, mocha};
use crate::ui::tr;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, Wrap},
};
use std::path::Path;

pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    // Split area: 3 lines for dashboard, remainder for device list/empty state
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    draw_summary_dashboard(f, app, layout[0]);
    app.layout_zones.flash_device_table = layout[1];

    if app.channels.is_empty() {
        draw_empty_state(f, app, layout[1]);
    } else {
        // Set auto toggle to the right side of the dashboard
        app.layout_zones.flash_auto_toggle = Rect::new(
            layout[0]
                .x
                .saturating_add(layout[0].width.saturating_sub(30)),
            layout[0].y,
            30,
            layout[0].height,
        );
        draw_device_table(f, app, layout[1]);
    }
}

fn draw_empty_state(f: &mut Frame, app: &mut App, area: Rect) {
    let centered_area = if area.width < 76 || area.height < 18 {
        area
    } else {
        crate::ui::center_rect(65, 18, area)
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

    let compact = area.height < 16 || area.width < 76;
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
            ]),
            Line::from(vec![
                Span::styled(
                    if lang == "zh" {
                        "   SN/批次:   "
                    } else {
                        "   SN/Lot:      "
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
                        "   Auto-Flash:   "
                    },
                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                ),
                Span::styled(
                    if app.auto_flash {
                        if lang == "zh" {
                            "启用 [点击切换]"
                        } else {
                            "ENABLED [Click to toggle]"
                        }
                    } else {
                        if lang == "zh" {
                            "禁用 [点击切换]"
                        } else {
                            "DISABLED [Click to toggle]"
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

        // The Auto-Flash line is the 7th line in the block (0-indexed).
        // Adding 1 for the top border, the Y offset from centered_area.y is 8.
        app.layout_zones.flash_auto_toggle = Rect::new(
            centered_area.x + 2,
            centered_area.y + 8,
            centered_area.width.saturating_sub(4),
            1,
        );
    } else {
        app.layout_zones.flash_auto_toggle = Rect::default();
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
            if app.auto_flash {
                if lang == "zh" {
                    "ON [点击切换]"
                } else {
                    "ON [Click to toggle]"
                }
            } else {
                if lang == "zh" {
                    "OFF [点击切换]"
                } else {
                    "OFF [Click to toggle]"
                }
            },
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
    let visible_rows = area.height.saturating_sub(2) as usize;
    let max_scroll = app.channels.len().saturating_sub(visible_rows);
    app.flash_table_scroll = app.flash_table_scroll.min(max_scroll);

    let title_text = flash_table_title(
        lang,
        app.flash_table_scroll,
        visible_rows,
        app.channels.len(),
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
        .style(Style::default().bg(mocha::MANTLE))
        .title(Span::styled(
            title_text,
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 || app.channels.is_empty() {
        return;
    }

    let row_constraints = vec![Constraint::Length(1); visible_rows];
    let list_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(inner);

    let mut ch_idx = app.flash_table_scroll;

    for y in 0..visible_rows {
        if ch_idx >= app.channels.len() {
            break;
        }

        let channel = &app.channels[ch_idx];
        let is_selected = ch_idx == app.selected_channel_idx;
        let is_hovered = app.hover_flash_row == Some(ch_idx);

        let bg_color = if is_selected {
            mocha::SURFACE1
        } else if is_hovered {
            mocha::SURFACE0
        } else if ch_idx % 2 == 0 {
            mocha::MANTLE
        } else {
            mocha::BASE
        };

        let port_text = format!("{:<8}", channel.port);
        let port_style = if is_selected {
            Style::default()
                .fg(CATPPUCCIN_MOCHA.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD)
        };

        let (status_text, status_color) = if channel.finished {
            if channel.success {
                (
                    if lang == "zh" { "成功" } else { "PASS" },
                    CATPPUCCIN_MOCHA.success,
                )
            } else {
                (
                    if lang == "zh" { "失败" } else { "FAIL" },
                    CATPPUCCIN_MOCHA.danger,
                )
            }
        } else if channel.status == "Idle" {
            (
                if lang == "zh" { "待机" } else { "IDLE" },
                CATPPUCCIN_MOCHA.text_muted,
            )
        } else {
            (
                if lang == "zh" { "活动" } else { "WORK" },
                CATPPUCCIN_MOCHA.warning,
            )
        };

        let pct = channel.progress;
        let bar_width = 16;
        let filled = (pct as usize * bar_width) / 100;
        let empty = bar_width.saturating_sub(filled);
        let bar_text = format!("{}{} {:>3}%", "█".repeat(filled), "░".repeat(empty), pct);

        let chip_text = channel.chip.as_deref().unwrap_or("Detecting...");
        let msg = if !channel.qa_result.is_empty() {
            format!("QA: {}", channel.qa_result)
        } else if channel.finished && !channel.success {
            channel
                .error
                .as_deref()
                .unwrap_or("Unknown Error")
                .to_string()
        } else if channel.status != "Idle" {
            channel.status.clone()
        } else {
            format!("{}", chip_text)
        };

        let line = Line::from(vec![
            Span::styled(
                if is_selected { " > " } else { "   " },
                Style::default().fg(CATPPUCCIN_MOCHA.accent).bg(bg_color),
            ),
            Span::styled(port_text, port_style.bg(bg_color)),
            Span::styled(
                format!(" │ {:<4} │ ", status_text),
                Style::default()
                    .fg(status_color)
                    .bg(bg_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(bar_text, Style::default().fg(status_color).bg(bg_color)),
            Span::styled(
                format!(" │ {} ", msg),
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.text_muted)
                    .bg(bg_color),
            ),
        ]);

        f.render_widget(
            Paragraph::new(line).style(Style::default().bg(bg_color)),
            list_rows[y],
        );

        ch_idx += 1;
    }
}

fn make_progress_bar(pct: u8, width: usize) -> String {
    let filled = (pct as usize * width) / 100;
    let empty = width - filled;
    format!("[{}{}] {:>3}%", "=".repeat(filled), " ".repeat(empty), pct)
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
    if channel.usb_manufacturer.as_deref() == Some("probe-rs") {
        return (
            if lang == "zh" {
                "调试器"
            } else {
                "Debug Probe"
            },
            Style::default()
                .fg(CATPPUCCIN_MOCHA.success)
                .add_modifier(Modifier::BOLD),
        );
    }

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

#[allow(dead_code)]
fn draw_empty_state_left(f: &mut Frame, app: &App, area: Rect) {
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
        Line::from(vec![Span::styled(
            tr("flash_connect_usb", lang),
            Style::default().fg(CATPPUCCIN_MOCHA.text_disabled),
        )]),
        Line::from(""),
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

    f.render_widget(Paragraph::new(info_lines).block(block), area);
}

pub fn draw_guided_burning_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let lang = &app.tool_config.language;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
        .title(Span::styled(
            if lang == "zh" {
                " 引导烧录清单与校验 "
            } else {
                " Guided Burning Manifest "
            },
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Let's divide inner area vertically:
    // Chunks[0]: Header info (Mode, DoNotChgBin status) -> Height 3
    // Chunks[1]: Table showing images manifest -> Min 5
    // Chunks[2]: Validation errors or Success status -> Height 4
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Mode selector and DoNotChgBin
            Constraint::Min(5),    // Images Table
            Constraint::Length(6), // Validation status
        ])
        .split(inner_area);

    // Mode Selector & DoNotChgBin Status
    let has_segmented = app.config.images.iter().any(|img| {
        img.label != "merged" && img.label != "factory_merged" && !img.path.ends_with("merged.bin")
    });
    let has_merged = app.config.images.iter().any(|img| {
        img.label == "merged" || img.label == "factory_merged" || img.path.ends_with("merged.bin")
    });
    let both_available = has_segmented && has_merged;

    let mode_str = if app.use_merged_flash {
        if lang == "zh" {
            "合并包烧录 (Merged Bin)"
        } else {
            "Merged-Bin Flashing"
        }
    } else {
        if lang == "zh" {
            "分段烧录 (Segmented)"
        } else {
            "Segmented Flashing"
        }
    };

    let mode_line = Line::from(vec![
        Span::styled(
            if lang == "zh" {
                " 烧录模式: "
            } else {
                " Mode: "
            },
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ),
        Span::styled(
            format!("◀ {} ▶", mode_str),
            Style::default()
                .fg(if both_available {
                    CATPPUCCIN_MOCHA.accent
                } else {
                    CATPPUCCIN_MOCHA.primary
                })
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            if lang == "zh" {
                " [点击切换]"
            } else {
                " [Click to toggle]"
            },
            Style::default().fg(CATPPUCCIN_MOCHA.text_disabled),
        ),
    ]);

    app.layout_zones.flash_mode_toggle = Rect::new(chunks[0].x, chunks[0].y, chunks[0].width, 1);
    app.layout_zones.flash_donotchg_toggle = Rect::new(
        chunks[0].x,
        chunks[0].y.saturating_add(1),
        chunks[0].width,
        1,
    );

    let donotchg_str = if app.config.do_not_chg_bin {
        "TRUE"
    } else {
        "FALSE"
    };
    let donotchg_color = if app.config.do_not_chg_bin {
        CATPPUCCIN_MOCHA.success
    } else {
        CATPPUCCIN_MOCHA.text_muted
    };
    let donotchg_line = Line::from(vec![
        Span::styled(
            " DoNotChgBin: ",
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ),
        Span::styled(
            donotchg_str,
            Style::default()
                .fg(donotchg_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            if app.config.do_not_chg_bin {
                if lang == "zh" {
                    " (不修改 Bin 头，点击切换)"
                } else {
                    " (No header mutation, click to toggle)"
                }
            } else {
                if lang == "zh" {
                    " (根据配置修改 Bin 头，点击切换)"
                } else {
                    " (Headers will be modified, click to toggle)"
                }
            },
            Style::default().fg(CATPPUCCIN_MOCHA.text_disabled),
        ),
    ]);

    f.render_widget(Paragraph::new(vec![mode_line, donotchg_line]), chunks[0]);

    app.layout_zones.flash_manifest_table = chunks[1];

    // Validation
    let (_, validation_errors) = app.config.validate_manifest();
    app.layout_zones.flash_manifest_status = chunks[2];

    // Manifest Table
    let mut rows = Vec::new();
    let headers = if lang == "zh" {
        vec!["偏移地址", "文件名称", "文件大小", "SHA256 校验", "状态"]
    } else {
        vec!["Offset", "File Name", "Size", "SHA256", "Status"]
    };

    let filtered_results = app.config.manifest_results_for_mode(app.use_merged_flash);

    for res in &filtered_results {
        let filename = if res.path.trim().is_empty() {
            ""
        } else {
            Path::new(&res.path)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or(&res.path)
        };

        let size_str = match res.size_bytes {
            Some(sz) => format_bytes(sz as usize),
            None => "-".to_string(),
        };

        let sha_span = match res.sha256_match {
            Some(true) => Span::styled(
                if lang == "zh" { "匹配" } else { "MATCH" },
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Some(false) => Span::styled(
                if lang == "zh" {
                    "不匹配"
                } else {
                    "MISMATCH"
                },
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.danger)
                    .add_modifier(Modifier::BOLD),
            ),
            None => Span::styled("N/A", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
        };

        let status_span = if res.path.trim().is_empty() {
            Span::styled(
                if lang == "zh" { "空" } else { "EMPTY" },
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            )
        } else if res.exists {
            Span::styled(
                if lang == "zh" { "正常" } else { "OK" },
                Style::default().fg(CATPPUCCIN_MOCHA.success),
            )
        } else {
            let is_req = app
                .config
                .images
                .iter()
                .find(|i| i.label == res.label)
                .map(|i| i.required)
                .unwrap_or(true);
            if is_req {
                Span::styled(
                    if lang == "zh" {
                        "缺失 (必选)"
                    } else {
                        "MISSING"
                    },
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.danger)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    if lang == "zh" {
                        "未找到 (可选)"
                    } else {
                        "OPTIONAL"
                    },
                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                )
            }
        };

        rows.push(
            Row::new(vec![
                Cell::from(Span::raw(res.offset.clone())),
                Cell::from(Span::raw(filename.to_string())),
                Cell::from(Span::raw(size_str)),
                Cell::from(sha_span),
                Cell::from(status_span),
            ])
            .style(Style::default().bg(mocha::BASE)),
        );
    }

    let widths = [
        Constraint::Length(10),
        Constraint::Min(15),
        Constraint::Length(10),
        Constraint::Length(12),
        Constraint::Length(14),
    ];

    let table = Table::new(rows, widths)
        .header(
            Row::new(headers.into_iter().map(Cell::from).collect::<Vec<_>>()).style(
                Style::default()
                    .fg(mocha::SUBTEXT1)
                    .bg(mocha::SURFACE0)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(table, chunks[1]);

    // Validation Status at bottom
    // Filter validation errors to only show relevant ones based on current mode
    // (e.g. if we are in merged mode, missing segmented files shouldn't block)
    let filtered_errors: Vec<String> = validation_errors
        .into_iter()
        .filter(|err| {
            if app.use_merged_flash {
                // Merged mode: ignore errors about bootloader/partitions/firmware/otadata segmented files missing
                !err.contains("bootloader")
                    && !err.contains("partitions")
                    && !err.contains("boot_app0")
                    && !err.contains("firmware")
                    && !err.contains("Required image")
            } else {
                // Segmented mode: ignore errors about merged files missing
                !err.contains("merged")
            }
        })
        .collect();

    let status_widget = if filtered_errors.is_empty() {
        let text = if app.manifest_locked {
            if lang == "zh" {
                " 清单已锁定: 点击此状态条解锁 "
            } else {
                " MANIFEST LOCKED: Click this status bar to unlock "
            }
        } else if lang == "zh" {
            " 就绪: 校验通过 | 点击此状态条锁定清单 "
        } else {
            " READY: Verified | Click this status bar to lock manifest "
        };
        let border_color = if app.manifest_locked {
            CATPPUCCIN_MOCHA.warning
        } else {
            CATPPUCCIN_MOCHA.success
        };
        Paragraph::new(text)
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(border_color))
                    .style(Style::default().bg(mocha::SURFACE0)),
            )
            .style(
                Style::default()
                    .fg(border_color)
                    .add_modifier(Modifier::BOLD),
            )
    } else {
        let lock_prefix = if app.manifest_locked {
            if lang == "zh" {
                "清单已锁定 | "
            } else {
                "LOCKED | "
            }
        } else {
            ""
        };
        let lock_suffix = if app.manifest_locked {
            if lang == "zh" {
                " | 点击解锁"
            } else {
                " | Click to unlock"
            }
        } else if lang == "zh" {
            " | 点击锁定"
        } else {
            " | Click to lock"
        };
        let err_text = format!(
            "{}{}{}",
            lock_prefix,
            filtered_errors.join(" | "),
            lock_suffix
        );
        Paragraph::new(err_text)
            .alignment(ratatui::layout::Alignment::Left)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(CATPPUCCIN_MOCHA.danger))
                    .style(Style::default().bg(mocha::SURFACE0)),
            )
            .style(
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.danger)
                    .add_modifier(Modifier::BOLD),
            )
    };

    f.render_widget(status_widget, chunks[2]);
}
