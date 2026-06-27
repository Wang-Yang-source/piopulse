pub mod cube;
pub mod image;

use crate::app::{App, PARAM_SLIDER_LAST_OFFSET, PARAM_SLIDER_TRACK_WIDTH};
use crate::ui::theme::{CATPPUCCIN_MOCHA, mocha};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::canvas::{Canvas, Line as CanvasLine},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    draw_probe_dashboard(f, app, area);
}

fn draw_probe_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let lang = &app.tool_config.language;
    let probe_channels: Vec<_> = app
        .channels
        .iter()
        .filter(|channel| channel.port.starts_with("probe:"))
        .collect();
    let serial_channels: Vec<_> = app
        .channels
        .iter()
        .filter(|channel| !channel.port.starts_with("probe:"))
        .collect();
    let selected_probe = app
        .channels
        .get(app.selected_channel_idx)
        .filter(|channel| channel.port.starts_with("probe:"))
        .or_else(|| probe_channels.first().copied());
    let selected_serial = app
        .channels
        .get(app.selected_channel_idx)
        .filter(|channel| !channel.port.starts_with("probe:"))
        .or_else(|| serial_channels.first().copied());
    let selected_probe_name = selected_probe
        .map(|channel| short_probe_name(&channel.port))
        .or_else(|| {
            selected_serial.map(|_| {
                if lang == "zh" {
                    "串口已连接".to_string()
                } else {
                    "serial connected".to_string()
                }
            })
        })
        .unwrap_or_else(|| {
            if lang == "zh" {
                "未发现探针"
            } else {
                "no probe"
            }
            .to_string()
        });
    let selected_probe_detail = selected_probe
        .map(|channel| {
            format!(
                "{} {}",
                channel.usb_product.as_deref().unwrap_or("probe-rs"),
                channel.usb_manufacturer.as_deref().unwrap_or("debug probe")
            )
        })
        .or_else(|| {
            selected_serial.map(|channel| {
                if lang == "zh" {
                    format!("{}；非 probe-rs 调试探针", serial_device_label(channel))
                } else {
                    format!(
                        "{}; not a probe-rs debug probe",
                        serial_device_label(channel)
                    )
                }
            })
        })
        .unwrap_or_else(|| {
            if lang == "zh" {
                "未发现串口或调试探针".to_string()
            } else {
                "no serial device or debug probe found".to_string()
            }
        });
    let target_status = selected_probe
        .map(|channel| channel.status.as_str())
        .unwrap_or_else(|| {
            if selected_serial.is_some() {
                if lang == "zh" {
                    "串口连接；调试未 attach"
                } else {
                    "serial only; debug detached"
                }
            } else if lang == "zh" {
                "未连接"
            } else {
                "detached"
            }
        });
    let target_name = selected_probe
        .or(selected_serial)
        .and_then(|channel| channel.chip.as_deref())
        .unwrap_or(app.config.chip_type.as_str());
    let firmware_path = probe_firmware_label(app);
    let firmware_size = std::fs::metadata(&firmware_path)
        .map(|metadata| format_bytes(metadata.len() as usize))
        .unwrap_or_else(|_| if lang == "zh" { "未找到" } else { "missing" }.to_string());

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(12),
            Constraint::Length(5),
        ])
        .split(area);
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(outer[0]);
    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(outer[1]);
    let body_top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(body[0]);
    let body_bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(body[1]);

    draw_metric_panel(
        f,
        top[0],
        if lang == "zh" { "Probe" } else { "Probe" },
        &selected_probe_name,
        &selected_probe_detail,
        CATPPUCCIN_MOCHA.primary,
    );
    draw_metric_panel(
        f,
        top[1],
        if lang == "zh" { "Target" } else { "Target" },
        target_name,
        target_status,
        CATPPUCCIN_MOCHA.info,
    );
    draw_metric_panel(
        f,
        top[2],
        if lang == "zh" { "Cores" } else { "Cores" },
        "Core 0",
        if lang == "zh" {
            "halt/run/step 预留"
        } else {
            "halt/run/step ready"
        },
        CATPPUCCIN_MOCHA.accent,
    );
    draw_metric_panel(
        f,
        top[3],
        if lang == "zh" { "Flash" } else { "Flash" },
        &firmware_size,
        &firmware_path,
        CATPPUCCIN_MOCHA.success,
    );

    draw_list_panel(
        f,
        body_top[0],
        if lang == "zh" {
            " 连接状态 "
        } else {
            " Connection "
        },
        connection_lines(app, lang),
    );
    draw_list_panel(
        f,
        body_top[1],
        if lang == "zh" {
            " Target/Core "
        } else {
            " Target/Core "
        },
        vec![
            metric_line(if lang == "zh" { "目标" } else { "target" }, target_name),
            metric_line(if lang == "zh" { "状态" } else { "state" }, target_status),
            metric_line(
                if lang == "zh" {
                    "当前 Core"
                } else {
                    "active core"
                },
                "0",
            ),
            metric_line("PC", "pending attach"),
            metric_line("SP", "pending attach"),
            metric_line("halt reason", "pending attach"),
        ],
    );
    draw_list_panel(
        f,
        body_top[2],
        if lang == "zh" {
            " Flash 下载 "
        } else {
            " Flash Download "
        },
        vec![
            metric_line(
                if lang == "zh" { "文件" } else { "file" },
                firmware_path.clone(),
            ),
            metric_line(if lang == "zh" { "大小" } else { "size" }, firmware_size),
            metric_line(
                if lang == "zh" { "格式" } else { "format" },
                firmware_format(&firmware_path),
            ),
            metric_line(
                if lang == "zh" { "地址" } else { "base addr" },
                app.config.app_offset.clone(),
            ),
            metric_line(
                if lang == "zh" { "擦除" } else { "erase" },
                app.config.erase_mode.clone(),
            ),
            metric_line(
                if lang == "zh" { "校验" } else { "verify" },
                app.config.verify_method.clone(),
            ),
        ],
    );
    draw_list_panel(
        f,
        body_top[3],
        if lang == "zh" {
            " Registers "
        } else {
            " Registers "
        },
        vec![
            metric_line("R0-R3", "pending attach"),
            metric_line("R4-R7", "pending attach"),
            metric_line("R8-R12", "pending attach"),
            metric_line("LR", "pending attach"),
            metric_line("xPSR", "pending attach"),
            metric_line("MSP/PSP", "pending attach"),
        ],
    );
    draw_list_panel(
        f,
        body_bottom[0],
        if lang == "zh" {
            " Memory Viewer "
        } else {
            " Memory Viewer "
        },
        vec![
            metric_line(if lang == "zh" { "地址" } else { "address" }, "0x00000000"),
            metric_line(if lang == "zh" { "长度" } else { "length" }, "256 bytes"),
            metric_line(
                if lang == "zh" { "格式" } else { "format" },
                "hex / u8 / u16 / u32",
            ),
            metric_line(if lang == "zh" { "RAM" } else { "RAM" }, "map pending"),
            metric_line(if lang == "zh" { "Flash" } else { "Flash" }, "map pending"),
            metric_line(
                if lang == "zh" { "操作" } else { "actions" },
                "read / write / dump",
            ),
        ],
    );
    draw_list_panel(
        f,
        body_bottom[1],
        if lang == "zh" {
            " RTT / Logs "
        } else {
            " RTT / Logs "
        },
        vec![
            metric_line(if lang == "zh" { "状态" } else { "state" }, "detached"),
            metric_line(
                if lang == "zh" {
                    "上行通道"
                } else {
                    "up channels"
                },
                "pending scan",
            ),
            metric_line(
                if lang == "zh" {
                    "下行通道"
                } else {
                    "down channels"
                },
                "pending scan",
            ),
            metric_line(if lang == "zh" { "吞吐" } else { "throughput" }, "0B/s"),
            metric_line(
                if lang == "zh" {
                    "日志缓存"
                } else {
                    "log buffer"
                },
                format!("{} lines", app.logs.len()),
            ),
            metric_line(
                if lang == "zh" { "发送" } else { "send" },
                "down channel ready",
            ),
        ],
    );
    draw_list_panel(
        f,
        body_bottom[2],
        if lang == "zh" {
            " Fault 诊断 "
        } else {
            " Fault Analysis "
        },
        vec![
            metric_line("HardFault", "pending halt"),
            metric_line("CFSR/HFSR", "pending read"),
            metric_line("MMFAR/BFAR", "pending read"),
            metric_line("stack", "pending unwind"),
            metric_line("symbol", "ELF symbols pending"),
            metric_line("reason", "attach target first"),
        ],
    );
    draw_list_panel(
        f,
        body_bottom[3],
        if lang == "zh" {
            " Profiling "
        } else {
            " Profiling "
        },
        vec![
            metric_line(
                if lang == "zh" {
                    "PC 采样"
                } else {
                    "PC samples"
                },
                "0",
            ),
            metric_line(
                if lang == "zh" {
                    "热点地址"
                } else {
                    "hot addresses"
                },
                "pending run",
            ),
            metric_line(if lang == "zh" { "符号" } else { "symbols" }, "ELF pending"),
            metric_line(
                if lang == "zh" {
                    "栈水位"
                } else {
                    "stack watermark"
                },
                "pending pattern",
            ),
            metric_line(
                if lang == "zh" {
                    "读速率"
                } else {
                    "read speed"
                },
                "0B/s",
            ),
            metric_line(
                if lang == "zh" { "目标" } else { "goal" },
                "performance tuning",
            ),
        ],
    );

    let footer = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                if lang == "zh" {
                    "Probe-rs 能力面板  "
                } else {
                    "Probe-rs capability panel  "
                },
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.text)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if lang == "zh" {
                    if selected_probe.is_some() {
                        "调试探针已发现；可继续接入 attach / halt / reset / memory"
                    } else if selected_serial.is_some() {
                        "已发现串口设备；probe-rs attach 需要 ESP32-S3 USB-JTAG 或外置调试器"
                    } else {
                        "未发现设备；请检查 USB、权限或端口扫描"
                    }
                } else {
                    if selected_probe.is_some() {
                        "debug probe found; attach / halt / reset / memory can be wired next"
                    } else if selected_serial.is_some() {
                        "serial device found; probe-rs attach needs ESP32-S3 USB-JTAG or an external probe"
                    } else {
                        "no device found; check USB, permissions, or rescan ports"
                    }
                },
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                if lang == "zh" {
                    "仪表盘  "
                } else {
                    "Dashboard  "
                },
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                if lang == "zh" {
                    "probe-rs 固定视图"
                } else {
                    "probe-rs fixed view"
                },
                Style::default().fg(CATPPUCCIN_MOCHA.success),
            ),
            Span::styled(
                if lang == "zh" {
                    "  寄存器 / 内存 / RTT / 性能分析"
                } else {
                    "  registers / memory / RTT / profiling"
                },
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
            .title(Span::styled(
                if lang == "zh" {
                    " 仪表盘 "
                } else {
                    " Dashboard "
                },
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.text)
                    .add_modifier(Modifier::BOLD),
            )),
    )
    .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));
    f.render_widget(footer, outer[2]);
}

fn connection_lines(app: &App, lang: &str) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = app
        .channels
        .iter()
        .filter(|channel| channel.port.starts_with("probe:"))
        .take(6)
        .map(|channel| {
            metric_line(
                short_probe_name(&channel.port),
                format!(
                    "{} {}",
                    channel.usb_product.as_deref().unwrap_or("probe-rs"),
                    channel.status
                ),
            )
        })
        .collect();

    let serial_lines: Vec<Line<'static>> = app
        .channels
        .iter()
        .filter(|channel| !channel.port.starts_with("probe:"))
        .take(4)
        .map(|channel| {
            metric_line(
                if lang == "zh" {
                    "串口设备"
                } else {
                    "serial board"
                },
                format!("{} {}", channel.port, serial_device_label(channel)),
            )
        })
        .collect();

    if lines.is_empty() {
        lines.push(metric_line(
            if lang == "zh" {
                "调试探针"
            } else {
                "debug probe"
            },
            if lang == "zh" {
                "未发现；probe-rs list 为空"
            } else {
                "not found; probe-rs list is empty"
            },
        ));
    }
    lines.extend(serial_lines);

    if lines.len() == 1 && app.channels.is_empty() {
        lines.push(metric_line(
            if lang == "zh" {
                "串口设备"
            } else {
                "serial board"
            },
            if lang == "zh" {
                "未发现；请扫描端口"
            } else {
                "not found; rescan ports"
            },
        ));
    }

    lines
}

fn serial_device_label(channel: &crate::app::Channel) -> String {
    let product = channel
        .usb_product
        .as_deref()
        .or(channel.usb_manufacturer.as_deref())
        .unwrap_or("USB serial");
    match (channel.vid, channel.pid) {
        (Some(vid), Some(pid)) => format!("{} {:04x}:{:04x}", product, vid, pid),
        _ => product.to_string(),
    }
}

fn short_probe_name(port: &str) -> String {
    let parts: Vec<&str> = port.split(':').collect();
    if parts.len() >= 4 {
        format!("{}:{} {}", parts[1], parts[2], parts[3])
    } else {
        port.to_string()
    }
}

fn probe_firmware_label(app: &App) -> String {
    app.config
        .images
        .iter()
        .find(|img| !img.path.trim().is_empty())
        .map(|img| img.path.clone())
        .filter(|path| !path.trim().is_empty())
        .unwrap_or_else(|| app.config.app_path.clone())
}

fn firmware_format(path: &str) -> String {
    std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_uppercase())
        .unwrap_or_else(|| "BIN".to_string())
}

fn draw_metric_panel(
    f: &mut Frame,
    area: Rect,
    title: &str,
    value: &str,
    detail: &str,
    color: ratatui::style::Color,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ))
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let p = Paragraph::new(vec![
        Line::from(Span::styled(
            value.to_string(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            detail.to_string(),
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        )),
    ])
    .alignment(Alignment::Center)
    .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));
    f.render_widget(p, inner);
}

fn metric_line(label: impl Into<String>, value: impl Into<String>) -> Line<'static> {
    let label = label.into();
    let value = value.into();
    Line::from(vec![
        Span::styled(
            format!("  {:<14}", label),
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ),
        Span::styled(value, Style::default().fg(CATPPUCCIN_MOCHA.text)),
    ])
}

fn draw_list_panel(f: &mut Frame, area: Rect, title: &str, lines: Vec<Line<'static>>) {
    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
                .title(Span::styled(
                    title.to_string(),
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.text)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn format_bytes(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1}MB", bytes as f64 / 1024.0 / 1024.0)
    } else if bytes >= 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}

// Custom widget drawing functions
fn draw_button_widget(f: &mut Frame, app: &App, area: Rect, is_focused: bool) {
    let border_color = if is_focused {
        CATPPUCCIN_MOCHA.border_focus
    } else {
        CATPPUCCIN_MOCHA.border
    };
    let border_type = if is_focused {
        BorderType::Double
    } else {
        BorderType::Rounded
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            format!(
                " {} ",
                crate::ui::tr("widget_button_title", &app.tool_config.language)
            ),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("   "),
        Span::styled(
            " [ START ] ",
            widget_button_style(
                CATPPUCCIN_MOCHA.success,
                is_focused && app.hover_widget_control == Some(0),
            ),
        ),
        Span::raw("   "),
        Span::styled(
            " [ STOP ] ",
            widget_button_style(
                CATPPUCCIN_MOCHA.danger,
                is_focused && app.hover_widget_control == Some(1),
            ),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("   "),
        Span::styled(
            " [ RESET ] ",
            widget_button_style(
                CATPPUCCIN_MOCHA.warning,
                is_focused && app.hover_widget_control == Some(2),
            ),
        ),
        Span::raw("   "),
        Span::styled(
            " [ PING  ] ",
            widget_button_style(
                CATPPUCCIN_MOCHA.primary,
                is_focused && app.hover_widget_control == Some(3),
            ),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "   Status: TUI command panel ready",
        Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
    )));

    let p = Paragraph::new(lines).wrap(Wrap { trim: true });
    f.render_widget(p, inner);
}

fn widget_button_style(color: ratatui::style::Color, hovered: bool) -> Style {
    Style::default().fg(mocha::CRUST).bg(color).add_modifier(
        Modifier::BOLD
            | if hovered {
                Modifier::REVERSED
            } else {
                Modifier::empty()
            },
    )
}

fn widget_control_line_style(hovered: bool) -> Style {
    if hovered {
        Style::default()
            .bg(CATPPUCCIN_MOCHA.selection_bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    }
}

fn draw_slider_widget(f: &mut Frame, app: &App, area: Rect, is_focused: bool) {
    let border_color = if is_focused {
        CATPPUCCIN_MOCHA.border_focus
    } else {
        CATPPUCCIN_MOCHA.border
    };
    let border_type = if is_focused {
        BorderType::Double
    } else {
        BorderType::Rounded
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            format!(
                " {} ",
                crate::ui::tr("widget_slider_title", &app.tool_config.language)
            ),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let kp_pct = (app.param_kp / 3.0).clamp(0.0, 1.0);
    let ki_pct = app.param_ki.clamp(0.0, 1.0);
    let kd_pct = app.param_kd.clamp(0.0, 1.0);

    let make_track = |pct: f64, color: ratatui::style::Color| {
        let thumb = (pct.clamp(0.0, 1.0) * PARAM_SLIDER_LAST_OFFSET as f64).round() as usize;
        let mut bar = String::with_capacity(PARAM_SLIDER_TRACK_WIDTH as usize);
        for idx in 0..PARAM_SLIDER_TRACK_WIDTH as usize {
            let ch = if idx == thumb {
                '|'
            } else if idx < thumb {
                '='
            } else {
                '-'
            };
            bar.push(ch);
        }
        Span::styled(bar, Style::default().fg(color))
    };

    let mut lines = Vec::new();
    lines.push(Line::from(""));
    lines.push(
        Line::from(vec![
            Span::styled("  Kp: ", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
            make_track(kp_pct, CATPPUCCIN_MOCHA.accent),
            Span::styled(
                format!(" {:.2}", app.param_kp),
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.text)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
        .style(widget_control_line_style(
            is_focused && app.hover_widget_control == Some(0),
        )),
    );
    lines.push(Line::from(""));
    lines.push(
        Line::from(vec![
            Span::styled("  Ki: ", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
            make_track(ki_pct, CATPPUCCIN_MOCHA.primary),
            Span::styled(
                format!(" {:.2}", app.param_ki),
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.text)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
        .style(widget_control_line_style(
            is_focused && app.hover_widget_control == Some(1),
        )),
    );
    lines.push(Line::from(""));
    lines.push(
        Line::from(vec![
            Span::styled("  Kd: ", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
            make_track(kd_pct, CATPPUCCIN_MOCHA.info),
            Span::styled(
                format!(" {:.2}", app.param_kd),
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.text)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
        .style(widget_control_line_style(
            is_focused && app.hover_widget_control == Some(2),
        )),
    );

    let p = Paragraph::new(lines).wrap(Wrap { trim: true });
    f.render_widget(p, inner);
}

fn draw_dial_widget(f: &mut Frame, app: &App, area: Rect, is_focused: bool) {
    let border_color = if is_focused {
        CATPPUCCIN_MOCHA.border_focus
    } else {
        CATPPUCCIN_MOCHA.border
    };
    let border_type = if is_focused {
        BorderType::Double
    } else {
        BorderType::Rounded
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            format!(
                " {} ",
                crate::ui::tr("widget_dial_title", &app.tool_config.language)
            ),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let min_rpm = 0.0;
    let max_rpm = 5000.0;
    let target = app.param_target_speed.clamp(min_rpm, max_rpm);
    let actual = app.sim_motor_speed.abs().clamp(min_rpm, max_rpm);
    let target_pct = ((target - min_rpm) / (max_rpm - min_rpm)).clamp(0.0, 1.0);
    let actual_pct = ((actual - min_rpm) / (max_rpm - min_rpm)).clamp(0.0, 1.0);
    let error = target - actual;
    let status = if error.abs() < 75.0 {
        ("LOCKED", CATPPUCCIN_MOCHA.success)
    } else if actual < target {
        ("RAMP UP", CATPPUCCIN_MOCHA.warning)
    } else {
        ("OVERSPEED", CATPPUCCIN_MOCHA.danger)
    };

    let target_marker = (target_pct * 20.0).round() as usize;
    let actual_marker = (actual_pct * 20.0).round() as usize;
    let mut scale = String::new();
    for i in 0..=20 {
        let ch = if i == target_marker && i == actual_marker {
            "◆"
        } else if i == target_marker {
            "T"
        } else if i == actual_marker {
            "●"
        } else if i % 5 == 0 {
            "|"
        } else {
            "-"
        };
        scale.push_str(ch);
    }

    let filled = (actual_pct * 18.0).round() as usize;
    let arc = format!(
        "{}{}",
        "█".repeat(filled.min(18)),
        "░".repeat(18usize.saturating_sub(filled.min(18)))
    );

    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("  MODE ", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
        Span::styled(
            "Speed Setpoint",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(
            status.0,
            Style::default().fg(status.1).add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "          0      2.5k      5k",
        Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
    )));
    lines.push(
        Line::from(vec![
            Span::raw("      "),
            Span::styled(scale, Style::default().fg(CATPPUCCIN_MOCHA.border_focus)),
        ])
        .style(widget_control_line_style(
            is_focused && app.hover_widget_control == Some(0),
        )),
    );
    lines.push(
        Line::from(vec![
            Span::raw("      "),
            Span::styled(
                format!("[{}]", arc),
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
        .style(widget_control_line_style(
            is_focused && app.hover_widget_control == Some(0),
        )),
    );
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            "  Target ",
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ),
        Span::styled(
            format!("{:>7.1} RPM", target),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled("Actual ", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
        Span::styled(
            format!("{:>7.1} RPM", actual),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.primary)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "  Error  ",
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ),
        Span::styled(
            format!("{:+7.1} RPM", error),
            Style::default().fg(if error.abs() < 75.0 {
                CATPPUCCIN_MOCHA.success
            } else {
                CATPPUCCIN_MOCHA.warning
            }),
        ),
        Span::raw("   "),
        Span::styled(
            "Range 0-5000 RPM",
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            "  Legend ",
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ),
        Span::styled(
            "T",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" target  "),
        Span::styled(
            "●",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" actual  "),
        Span::styled(
            "◆",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.success)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" aligned"),
    ]));

    let p = Paragraph::new(lines).wrap(Wrap { trim: true });
    f.render_widget(p, inner);
}

fn draw_joystick_widget(f: &mut Frame, _app: &App, area: Rect, is_focused: bool) {
    let border_color = if is_focused {
        CATPPUCCIN_MOCHA.border_focus
    } else {
        CATPPUCCIN_MOCHA.border
    };
    let border_type = if is_focused {
        BorderType::Double
    } else {
        BorderType::Rounded
    };

    let aspect = (area.width as f64 / area.height as f64) * 0.5;
    let x_limit = 1.0 * aspect;

    let canvas = Canvas::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(border_type)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(
                    format!(
                        " {} ",
                        crate::ui::tr("widget_joystick_title", &_app.tool_config.language)
                    ),
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.text)
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(CATPPUCCIN_MOCHA.panel)),
        )
        .x_bounds([-x_limit, x_limit])
        .y_bounds([-1.0, 1.0])
        .paint(move |ctx| {
            ctx.draw(&CanvasLine {
                x1: -x_limit,
                y1: 0.0,
                x2: x_limit,
                y2: 0.0,
                color: mocha::SURFACE0,
            });
            ctx.draw(&CanvasLine {
                x1: 0.0,
                y1: -1.0,
                x2: 0.0,
                y2: 1.0,
                color: mocha::SURFACE0,
            });

            let r = 0.8;
            for i in 0..36 {
                let a1 = (i as f64) * 10.0f64.to_radians();
                let a2 = ((i + 1) as f64) * 10.0f64.to_radians();
                ctx.draw(&CanvasLine {
                    x1: a1.cos() * r,
                    y1: a1.sin() * r,
                    x2: a2.cos() * r,
                    y2: a2.sin() * r,
                    color: CATPPUCCIN_MOCHA.border,
                });
            }

            let jx = 0.35;
            let jy = 0.25;
            ctx.draw(&CanvasLine {
                x1: 0.0,
                y1: 0.0,
                x2: jx,
                y2: jy,
                color: CATPPUCCIN_MOCHA.accent,
            });
            ctx.print(jx - 0.1, jy, "●");

            ctx.print(-x_limit + 0.1, -0.8, format!("X:{:+.2} Y:{:+.2}", jx, jy));
        });

    f.render_widget(canvas, area);
}

fn draw_light_widget(f: &mut Frame, _app: &App, area: Rect, is_focused: bool) {
    let border_color = if is_focused {
        CATPPUCCIN_MOCHA.border_focus
    } else {
        CATPPUCCIN_MOCHA.border
    };
    let border_type = if is_focused {
        BorderType::Double
    } else {
        BorderType::Rounded
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            format!(
                " {} ",
                crate::ui::tr("widget_light_title", &_app.tool_config.language)
            ),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  ● ", Style::default().fg(CATPPUCCIN_MOCHA.success)),
        Span::styled("System Power  ", Style::default().fg(CATPPUCCIN_MOCHA.text)),
        Span::styled("  ● ", Style::default().fg(CATPPUCCIN_MOCHA.success)),
        Span::styled("Wi-Fi Comm", Style::default().fg(CATPPUCCIN_MOCHA.text)),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  ● ", Style::default().fg(CATPPUCCIN_MOCHA.danger)),
        Span::styled("Error Lockout ", Style::default().fg(CATPPUCCIN_MOCHA.text)),
        Span::styled("  ● ", Style::default().fg(CATPPUCCIN_MOCHA.warning)),
        Span::styled("Calib Mode", Style::default().fg(CATPPUCCIN_MOCHA.text)),
    ]));

    let p = Paragraph::new(lines).wrap(Wrap { trim: true });
    f.render_widget(p, inner);
}

fn draw_gauge_widget(f: &mut Frame, app: &App, area: Rect, is_focused: bool) {
    let border_color = if is_focused {
        CATPPUCCIN_MOCHA.border_focus
    } else {
        CATPPUCCIN_MOCHA.border
    };
    let border_type = if is_focused {
        BorderType::Double
    } else {
        BorderType::Rounded
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            format!(
                " {} ",
                crate::ui::tr("widget_gauge_title", &app.tool_config.language)
            ),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let gauge_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(inner);

    let voltage_str = format!("Voltage: {:.2}V (Cell: 3.82V)", app.sim_battery_voltage);
    f.render_widget(
        Paragraph::new(Span::styled(
            voltage_str,
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        )),
        gauge_chunks[0],
    );

    let percent = ((app.sim_battery_voltage - 10.0) / 2.6 * 100.0).clamp(0.0, 100.0) as u16;
    let gauge = ratatui::widgets::Gauge::default()
        .block(Block::default())
        .gauge_style(
            Style::default()
                .fg(CATPPUCCIN_MOCHA.success)
                .bg(mocha::SURFACE0),
        )
        .percent(percent);

    f.render_widget(gauge, gauge_chunks[1]);
}

fn draw_dashboard_widget(f: &mut Frame, app: &App, area: Rect, is_focused: bool) {
    let border_color = if is_focused {
        CATPPUCCIN_MOCHA.border_focus
    } else {
        CATPPUCCIN_MOCHA.border
    };
    let border_type = if is_focused {
        BorderType::Double
    } else {
        BorderType::Rounded
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            format!(
                " {} ",
                crate::ui::tr("widget_dashboard_title", &app.tool_config.language)
            ),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            "  Motor Status:     ",
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ),
        if app.motor_enabled {
            Span::styled(
                "ENABLED",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.success)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(
                "DISABLED",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.danger)
                    .add_modifier(Modifier::BOLD),
            )
        },
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "  Feedback Speed:   ",
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ),
        Span::styled(
            format!("{:.1} RPM", app.sim_motor_speed),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.primary)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "  Target Speed:     ",
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ),
        Span::styled(
            format!("{:.1} RPM", app.param_target_speed),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.accent)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            "  Control Output:   ",
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ),
        Span::styled(
            format!("{:.3}", app.sim_pid_out),
            Style::default().fg(CATPPUCCIN_MOCHA.info),
        ),
    ]));

    let p = Paragraph::new(lines).wrap(Wrap { trim: true });
    f.render_widget(p, inner);
}

fn draw_example_widget(f: &mut Frame, _app: &App, area: Rect, is_focused: bool) {
    let border_color = if is_focused {
        CATPPUCCIN_MOCHA.border_focus
    } else {
        CATPPUCCIN_MOCHA.border
    };
    let border_type = if is_focused {
        BorderType::Double
    } else {
        BorderType::Rounded
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            format!(
                " {} ",
                crate::ui::tr("widget_example_title", &_app.tool_config.language)
            ),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  fn draw_custom_module(f: &mut Frame, app: &App, area: Rect) {",
        Style::default().fg(CATPPUCCIN_MOCHA.primary),
    )));
    lines.push(Line::from(Span::styled(
        "      let block = Block::default().title(\" Custom Module \");",
        Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
    )));
    lines.push(Line::from(Span::styled(
        "      f.render_widget(block, area);",
        Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
    )));
    lines.push(Line::from(Span::styled(
        "  }",
        Style::default().fg(CATPPUCCIN_MOCHA.primary),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("  Status: "),
        Span::styled(
            "RATATUI TEMPLATE ACTIVE",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.success)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    let p = Paragraph::new(lines).wrap(Wrap { trim: true });
    f.render_widget(p, inner);
}

fn draw_delay_widget(f: &mut Frame, app: &App, area: Rect, is_focused: bool) {
    let border_color = if is_focused {
        CATPPUCCIN_MOCHA.border_focus
    } else {
        CATPPUCCIN_MOCHA.border
    };
    let border_type = if is_focused {
        BorderType::Double
    } else {
        BorderType::Rounded
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            format!(
                " {} ",
                crate::ui::tr("widget_delay_title", &app.tool_config.language)
            ),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines = Vec::new();
    lines.push(Line::from(""));
    lines.push(
        Line::from(vec![
            Span::raw("  [ "),
            Span::styled(
                "HOLD TO TRIGGER (1.5s)",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.text)
                    .bg(CATPPUCCIN_MOCHA.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" ]"),
        ])
        .style(widget_control_line_style(
            is_focused && app.hover_widget_control == Some(0),
        )),
    );
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("  Progress: "),
        Span::styled(
            "[██████░░░░] 60%",
            Style::default().fg(CATPPUCCIN_MOCHA.accent),
        ),
    ]));
    lines.push(Line::from(Span::styled(
        "  State: Armed & Calibration Checked",
        Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
    )));

    let p = Paragraph::new(lines).wrap(Wrap { trim: true });
    f.render_widget(p, inner);
}

fn draw_toggle_widget(f: &mut Frame, app: &App, area: Rect, is_focused: bool) {
    let border_color = if is_focused {
        CATPPUCCIN_MOCHA.border_focus
    } else {
        CATPPUCCIN_MOCHA.border
    };
    let border_type = if is_focused {
        BorderType::Double
    } else {
        BorderType::Rounded
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            format!(
                " {} ",
                crate::ui::tr("widget_toggle_title", &app.tool_config.language)
            ),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines = Vec::new();
    lines.push(Line::from(""));
    lines.push(
        Line::from(vec![
            Span::raw("  Latching Switch: "),
            Span::styled(
                "  [ ON ]  ",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.text)
                    .bg(CATPPUCCIN_MOCHA.success)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
        .style(widget_control_line_style(
            is_focused && app.hover_widget_control == Some(0),
        )),
    );
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("  Output Signal:   "),
        Span::styled("GPIO 14 - HIGH", Style::default().fg(CATPPUCCIN_MOCHA.info)),
    ]));
    lines.push(Line::from(Span::styled(
        "  Mode: Self-locking Toggle (Press to switch)",
        Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
    )));

    let p = Paragraph::new(lines).wrap(Wrap { trim: true });
    f.render_widget(p, inner);
}

fn draw_knob_widget(f: &mut Frame, app: &App, area: Rect, is_focused: bool) {
    let border_color = if is_focused {
        CATPPUCCIN_MOCHA.border_focus
    } else {
        CATPPUCCIN_MOCHA.border
    };
    let border_type = if is_focused {
        BorderType::Double
    } else {
        BorderType::Rounded
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            format!(
                " {} ",
                crate::ui::tr("widget_knob_title", &app.tool_config.language)
            ),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let angle = (app.param_knob * 360.0).round() as usize;
    let value = app.param_knob * 100.0;

    let pct = app.param_knob.clamp(0.0, 1.0);
    let total_width = 11;
    let dot_pos = (pct * (total_width - 1) as f64).round() as usize;
    let mut bar_chars = vec!['-'; total_width];
    bar_chars[0] = '├';
    bar_chars[total_width - 1] = '┤';
    bar_chars[dot_pos] = '●';
    let bar: String = bar_chars.into_iter().collect();

    let mut lines = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("  Angle Position:  "),
        Span::styled(
            format!("{}° / 360°", angle),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.accent)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  Scale Value:     "),
        Span::styled(
            format!("{:.2} units", value),
            Style::default().fg(CATPPUCCIN_MOCHA.primary),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(
        Line::from(vec![
            Span::raw("  [Min] "),
            Span::styled(bar, Style::default().fg(CATPPUCCIN_MOCHA.accent)),
            Span::raw(" [Max]"),
        ])
        .style(widget_control_line_style(
            is_focused && app.hover_widget_control == Some(0),
        )),
    );

    let p = Paragraph::new(lines).wrap(Wrap { trim: true });
    f.render_widget(p, inner);
}

fn draw_ring_widget(f: &mut Frame, _app: &App, area: Rect, is_focused: bool) {
    let border_color = if is_focused {
        CATPPUCCIN_MOCHA.border_focus
    } else {
        CATPPUCCIN_MOCHA.border
    };
    let border_type = if is_focused {
        BorderType::Double
    } else {
        BorderType::Rounded
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            format!(
                " {} ",
                crate::ui::tr("widget_ring_title", &_app.tool_config.language)
            ),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("  Ring Level:      "),
        Span::styled(
            "64%",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.success)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "      ╭━━━━╮ ",
        Style::default().fg(CATPPUCCIN_MOCHA.success),
    )));
    lines.push(Line::from(vec![
        Span::styled("      ┃  ", Style::default().fg(CATPPUCCIN_MOCHA.success)),
        Span::styled("●", Style::default().fg(CATPPUCCIN_MOCHA.accent)),
        Span::styled(
            " ┃  (Hover Active)",
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ),
    ]));
    lines.push(Line::from(Span::styled(
        "      ╰━━━━╯ ",
        Style::default().fg(CATPPUCCIN_MOCHA.success),
    )));

    let p = Paragraph::new(lines).wrap(Wrap { trim: true });
    f.render_widget(p, inner);
}

fn draw_pad_widget(f: &mut Frame, _app: &App, area: Rect, is_focused: bool) {
    let border_color = if is_focused {
        CATPPUCCIN_MOCHA.border_focus
    } else {
        CATPPUCCIN_MOCHA.border
    };
    let border_type = if is_focused {
        BorderType::Double
    } else {
        BorderType::Rounded
    };

    let aspect = (area.width as f64 / area.height as f64) * 0.5;
    let x_limit = 1.0 * aspect;

    let canvas = Canvas::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(border_type)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(
                    format!(
                        " {} ",
                        crate::ui::tr("widget_pad_title", &_app.tool_config.language)
                    ),
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.text)
                        .add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(CATPPUCCIN_MOCHA.panel)),
        )
        .x_bounds([-x_limit, x_limit])
        .y_bounds([-1.0, 1.0])
        .paint(move |ctx| {
            // Draw the terminal canvas boundary and crosshairs.
            ctx.draw(&CanvasLine {
                x1: -x_limit,
                y1: 0.0,
                x2: x_limit,
                y2: 0.0,
                color: mocha::SURFACE2,
            });
            ctx.draw(&CanvasLine {
                x1: 0.0,
                y1: -1.0,
                x2: 0.0,
                y2: 1.0,
                color: mocha::SURFACE2,
            });

            // Draw concentric rings representing deadzones
            for &r in &[0.3, 0.6, 0.9] {
                for i in 0..18 {
                    let a1 = (i as f64) * 20.0f64.to_radians();
                    let a2 = ((i + 1) as f64) * 20.0f64.to_radians();
                    ctx.draw(&CanvasLine {
                        x1: a1.cos() * r,
                        y1: a1.sin() * r,
                        x2: a2.cos() * r,
                        y2: a2.sin() * r,
                        color: mocha::SURFACE0,
                    });
                }
            }

            // Draw current feedback pointer
            let mx = -0.45;
            let my = 0.55;
            ctx.draw(&CanvasLine {
                x1: 0.0,
                y1: 0.0,
                x2: mx,
                y2: my,
                color: CATPPUCCIN_MOCHA.success,
            });
            ctx.print(mx - 0.1, my, "⬤");

            ctx.print(
                -x_limit + 0.1,
                -0.8,
                format!("Axis X/Y: {:.2}, {:.2}", mx, my),
            );
        });

    f.render_widget(canvas, area);
}

pub fn get_pane_layouts(area: Rect, count: usize) -> Vec<Rect> {
    if count == 0 {
        return Vec::new();
    }
    if count == 1 {
        return vec![area];
    }
    if count == 2 {
        return Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area)
            .to_vec();
    }
    if count == 3 {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        let right_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(cols[1]);
        return vec![cols[0], right_rows[0], right_rows[1]];
    }
    if count == 4 {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        let top_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[0]);
        let bottom_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[1]);
        return vec![top_cols[0], top_cols[1], bottom_cols[0], bottom_cols[1]];
    }
    if count == 5 {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        let top_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[0]);
        let bottom_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(rows[1]);
        return vec![
            top_cols[0],
            top_cols[1],
            bottom_cols[0],
            bottom_cols[1],
            bottom_cols[2],
        ];
    }
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(rows[0]);
    let bottom_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(rows[1]);
    vec![
        top_cols[0],
        top_cols[1],
        top_cols[2],
        bottom_cols[0],
        bottom_cols[1],
        bottom_cols[2],
    ]
}

fn center_rect(percent_x: u16, height_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height.saturating_sub(height_y)) / 2),
            Constraint::Length(height_y),
            Constraint::Min(0),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
