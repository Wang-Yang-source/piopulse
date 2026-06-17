pub mod cube;
pub mod image;

use crate::app::{
    App, DashboardEmptyAction, PARAM_SLIDER_LAST_OFFSET, PARAM_SLIDER_TRACK_WIDTH, WidgetType,
};
use crate::ui::theme::{CATPPUCCIN_MOCHA, mocha};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::canvas::{Canvas, Line as CanvasLine},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

pub fn get_catalog_items() -> Vec<(&'static str, &'static str, WidgetType)> {
    vec![
        ("button", "widget_button_desc", WidgetType::Button),
        ("cube", "widget_cube_desc", WidgetType::Cube),
        ("dashboard", "widget_dashboard_desc", WidgetType::Dashboard),
        ("delay", "widget_delay_desc", WidgetType::Delay),
        ("dial", "widget_dial_desc", WidgetType::Dial),
        ("example", "widget_example_desc", WidgetType::Example),
        ("gauge", "widget_gauge_desc", WidgetType::Gauge),
        ("image", "widget_image_desc", WidgetType::Image),
        ("joystick", "widget_joystick_desc", WidgetType::Joystick),
        ("knob", "widget_knob_desc", WidgetType::Knob),
        ("light", "widget_light_desc", WidgetType::Light),
        ("pad", "widget_pad_desc", WidgetType::Pad),
        ("ring", "widget_ring_desc", WidgetType::Ring),
        ("slider", "widget_slider_desc", WidgetType::Slider),
        ("toggle", "widget_toggle_desc", WidgetType::Toggle),
    ]
}

pub fn get_filtered_catalog_items(
    search: &str,
    _lang: &str,
) -> Vec<(&'static str, &'static str, WidgetType)> {
    let search = search.to_lowercase();
    let mut items: Vec<_> = get_catalog_items()
        .into_iter()
        .filter(|(name, desc_key, _)| {
            name.contains(&search)
                || crate::ui::tr(desc_key, "en")
                    .to_lowercase()
                    .contains(&search)
                || crate::ui::tr(desc_key, "zh").contains(&search)
        })
        .collect();
    // Keep the module catalog stable as new modules are added.
    items.sort_by_key(|(name, _, _)| *name);
    items
}

pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    if app.dashboard_widgets.is_empty() {
        draw_welcome_screen(f, app, area);
    } else {
        // Draw split panes based on tiled count
        let count = app.dashboard_widgets.len();
        let pane_layouts = get_pane_layouts(area, count);

        for (idx, &widget_type) in app.dashboard_widgets.iter().enumerate() {
            let pane_area = pane_layouts[idx];
            let is_focused = app.selected_widget_idx == idx;

            match widget_type {
                WidgetType::Cube => cube::draw(f, app, pane_area, is_focused),
                WidgetType::Image => image::draw(f, app, pane_area, is_focused),
                WidgetType::Button => draw_button_widget(f, app, pane_area, is_focused),
                WidgetType::Slider => draw_slider_widget(f, app, pane_area, is_focused),
                WidgetType::Dial => draw_dial_widget(f, app, pane_area, is_focused),
                WidgetType::Joystick => draw_joystick_widget(f, app, pane_area, is_focused),
                WidgetType::Light => draw_light_widget(f, app, pane_area, is_focused),
                WidgetType::Gauge => draw_gauge_widget(f, app, pane_area, is_focused),
                WidgetType::Dashboard => draw_dashboard_widget(f, app, pane_area, is_focused),
                WidgetType::Example => draw_example_widget(f, app, pane_area, is_focused),
                WidgetType::Delay => draw_delay_widget(f, app, pane_area, is_focused),
                WidgetType::Toggle => draw_toggle_widget(f, app, pane_area, is_focused),
                WidgetType::Knob => draw_knob_widget(f, app, pane_area, is_focused),
                WidgetType::Ring => draw_ring_widget(f, app, pane_area, is_focused),
                WidgetType::Pad => draw_pad_widget(f, app, pane_area, is_focused),
            }
        }
    }

    // Centered popup modal to add widget
    if app.is_adding_widget {
        let modal_height = area.height.saturating_sub(2).clamp(8, 20);
        let modal_width_pct = if area.width < 78 { 94 } else { 65 };
        let modal_area = center_rect(modal_width_pct, modal_height, area);
        app.layout_zones.widget_add_modal = modal_area;
        draw_add_widget_modal(f, app, modal_area);
    }
}

fn draw_welcome_screen(f: &mut Frame, app: &App, area: Rect) {
    let lang = &app.tool_config.language;
    let selected_port = app.get_selected_port().unwrap_or_else(|| {
        if lang == "zh" {
            "未选择".into()
        } else {
            "NONE".into()
        }
    });
    let telemetry_points = app
        .get_selected_port()
        .and_then(|port| app.waveform_history.get(&port).map(Vec::len))
        .unwrap_or(0);
    let latest_channels = app
        .get_selected_port()
        .and_then(|port| app.waveform_history.get(&port))
        .and_then(|frames| frames.last())
        .map(Vec::len)
        .unwrap_or(0);

    let title = if lang == "zh" {
        " 仪表盘工作台 - 未预设模块 "
    } else {
        " Dashboard Workspace - No Preset Modules "
    };

    let compact = area.height < 16 || area.width < 82;
    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                if lang == "zh" {
                    "  当前端口  "
                } else {
                    "  Active Port  "
                },
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                selected_port,
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("    "),
            Span::styled(
                if lang == "zh" {
                    "遥测缓存  "
                } else {
                    "Telemetry  "
                },
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                format!("{} frames / {} ch", telemetry_points, latest_channels),
                Style::default().fg(CATPPUCCIN_MOCHA.info),
            ),
            Span::raw("    "),
            Span::styled(
                if lang == "zh" {
                    "模块  "
                } else {
                    "Modules  "
                },
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                format!("{}/6", app.dashboard_widgets.len()),
                Style::default().fg(CATPPUCCIN_MOCHA.success),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            action_chip(
                if lang == "zh" {
                    "添加模块"
                } else {
                    "Add Module"
                },
                CATPPUCCIN_MOCHA.success,
                app.hover_dashboard_empty_action == Some(DashboardEmptyAction::AddCatalog),
            ),
            Span::raw("  "),
            action_chip(
                "Button",
                CATPPUCCIN_MOCHA.primary,
                app.hover_dashboard_empty_action == Some(DashboardEmptyAction::Button),
            ),
            Span::raw("  "),
            action_chip(
                "Slider",
                CATPPUCCIN_MOCHA.accent,
                app.hover_dashboard_empty_action == Some(DashboardEmptyAction::Slider),
            ),
            Span::raw("  "),
            action_chip(
                "Dashboard",
                CATPPUCCIN_MOCHA.info,
                app.hover_dashboard_empty_action == Some(DashboardEmptyAction::Dashboard),
            ),
            Span::raw("  "),
            action_chip(
                "Image",
                CATPPUCCIN_MOCHA.warning,
                app.hover_dashboard_empty_action == Some(DashboardEmptyAction::Image),
            ),
        ]),
    ];

    if !compact {
        lines.extend([
            Line::from(""),
            Line::from(Span::styled(
                if lang == "zh" {
                    "  推荐：点击下面任一行直接添加，也可以点击上方添加模块打开完整目录。"
                } else {
                    "  Suggested: click a row to add it, or click Add Module for the full catalog."
                },
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            )),
            Line::from(""),
            suggestion_line(
                "Button",
                if lang == "zh" {
                    "串口 START / STOP / RESET / PING 控制面板"
                } else {
                    "Serial START / STOP / RESET / PING command panel"
                },
                CATPPUCCIN_MOCHA.primary,
                app.hover_dashboard_empty_action == Some(DashboardEmptyAction::Button),
            ),
            suggestion_line(
                "Slider",
                if lang == "zh" {
                    "可点击调节 Kp / Ki / Kd 参数"
                } else {
                    "Clickable Kp / Ki / Kd tuning controls"
                },
                CATPPUCCIN_MOCHA.accent,
                app.hover_dashboard_empty_action == Some(DashboardEmptyAction::Slider),
            ),
            suggestion_line(
                "Dashboard",
                if lang == "zh" {
                    "电机状态、目标速度和控制输出摘要"
                } else {
                    "Motor state, target speed, and control output summary"
                },
                CATPPUCCIN_MOCHA.info,
                app.hover_dashboard_empty_action == Some(DashboardEmptyAction::Dashboard),
            ),
            suggestion_line(
                "Image",
                if lang == "zh" {
                    "查看 VOFA 图像或 ROI 数据"
                } else {
                    "View VOFA image or ROI payloads"
                },
                CATPPUCCIN_MOCHA.warning,
                app.hover_dashboard_empty_action == Some(DashboardEmptyAction::Image),
            ),
            suggestion_line(
                "Cube",
                if lang == "zh" {
                    "需要 IMU 姿态时再手动添加"
                } else {
                    "Add manually when IMU orientation is needed"
                },
                CATPPUCCIN_MOCHA.success,
                app.hover_dashboard_empty_action == Some(DashboardEmptyAction::Cube),
            ),
            Line::from(""),
            Line::from(Span::styled(
                if lang == "zh" {
                    "  鼠标：点击推荐模块添加；键盘：A 打开目录，D 删除模块，方向键切换模块。"
                } else {
                    "  Mouse: click suggestions to add; Keyboard: A catalog, D delete, arrows switch panes."
                },
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            )),
        ]);
    } else {
        lines.push(Line::from(Span::styled(
            if lang == "zh" {
                "  小屏模式：点击上方按钮，或按 A 打开目录。"
            } else {
                "  Compact: click a button above, or press A for catalog."
            },
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        )));
    }

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
                .title(Span::styled(
                    title,
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.text)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel))
        .alignment(Alignment::Left);

    f.render_widget(p, area);
}

fn action_chip(label: &str, color: ratatui::style::Color, hovered: bool) -> Span<'_> {
    Span::styled(
        format!("[ {} ]", label),
        Style::default().fg(mocha::CRUST).bg(color).add_modifier(
            Modifier::BOLD
                | if hovered {
                    Modifier::REVERSED
                } else {
                    Modifier::empty()
                },
        ),
    )
}

fn suggestion_line<'a>(
    label: &'a str,
    description: &'a str,
    color: ratatui::style::Color,
    hovered: bool,
) -> Line<'a> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(
            format!("{:<10}", label),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(description, Style::default().fg(CATPPUCCIN_MOCHA.text)),
    ])
    .style(if hovered {
        Style::default()
            .bg(CATPPUCCIN_MOCHA.selection_bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    })
}

fn draw_add_widget_modal(f: &mut Frame, app: &App, area: Rect) {
    let lang = &app.tool_config.language;
    let filtered_items = get_filtered_catalog_items(&app.widget_search_input, lang);
    let max_items = area.height.saturating_sub(8) as usize;

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        crate::ui::tr("dash_select_module_title", lang),
        Style::default()
            .fg(CATPPUCCIN_MOCHA.accent)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        "───────────────────────────────────────────────────",
        Style::default().fg(CATPPUCCIN_MOCHA.border),
    )));
    lines.push(Line::from(""));

    let search_label = format!(
        "{}{}█",
        crate::ui::tr("dash_search", lang),
        app.widget_search_input
    );
    lines.push(Line::from(Span::styled(
        search_label,
        Style::default().fg(CATPPUCCIN_MOCHA.text),
    )));
    lines.push(Line::from(Span::styled(
        "───────────────────────────────────────────────────",
        Style::default().fg(CATPPUCCIN_MOCHA.border),
    )));
    lines.push(Line::from(""));

    for (idx, (name, desc_key, _)) in filtered_items.iter().take(max_items).enumerate() {
        let is_selected = app.add_menu_selected == idx;
        let prefix = if is_selected {
            Span::styled(
                " ▶ ",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw("   ")
        };
        let style = if is_selected {
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted)
        };
        let translated_desc = crate::ui::tr(desc_key, lang);
        lines.push(Line::from(vec![
            prefix,
            Span::styled(format!("{:<18} : {}", name, translated_desc), style),
        ]));
    }
    if filtered_items.len() > max_items {
        lines.push(Line::from(Span::styled(
            format!("   ... {} more", filtered_items.len() - max_items),
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        crate::ui::tr("dash_modal_hint", lang),
        Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
    )));

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(CATPPUCCIN_MOCHA.accent))
                .title(Span::styled(
                    crate::ui::tr("dash_catalog_title", lang),
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.text)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    f.render_widget(p, area);
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
