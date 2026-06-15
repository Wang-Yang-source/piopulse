pub mod cube;
pub mod image;

use crate::app::{App, WidgetType};
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
        ("button", "Ratatui TUI Button Panel", WidgetType::Button),
        ("cube", "Ratatui Canvas Orientation Cube", WidgetType::Cube),
        (
            "dashboard",
            "Ratatui TUI System Dashboard",
            WidgetType::Dashboard,
        ),
        ("delay", "Ratatui TUI Delayed Trigger", WidgetType::Delay),
        ("dial", "Ratatui TUI Dial Panel", WidgetType::Dial),
        (
            "example",
            "Example Rust/Ratatui Module Template",
            WidgetType::Example,
        ),
        ("gauge", "Ratatui Gauge Telemetry Meter", WidgetType::Gauge),
        (
            "image",
            "Ratatui Canvas Image/ROI Preview",
            WidgetType::Image,
        ),
        (
            "joystick",
            "Ratatui Canvas Joystick Grid",
            WidgetType::Joystick,
        ),
        ("knob", "Ratatui TUI Precision Knob", WidgetType::Knob),
        ("light", "Ratatui TUI Status Lights", WidgetType::Light),
        ("pad", "Ratatui Canvas Dual-Axis Pad", WidgetType::Pad),
        ("ring", "Ratatui TUI Ring Dial", WidgetType::Ring),
        ("slider", "Ratatui TUI Parameter Slider", WidgetType::Slider),
        ("toggle", "Ratatui TUI Latched Switch", WidgetType::Toggle),
    ]
}

pub fn get_filtered_catalog_items(search: &str) -> Vec<(&'static str, &'static str, WidgetType)> {
    let search = search.to_lowercase();
    let mut items: Vec<_> = get_catalog_items()
        .into_iter()
        .filter(|(name, desc, _)| name.contains(&search) || desc.to_lowercase().contains(&search))
        .collect();
    // Keep the module catalog stable as new modules are added.
    items.sort_by_key(|(name, _, _)| *name);
    items
}

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
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
        let modal_area = center_rect(65, 20, area);
        draw_add_widget_modal(f, app, modal_area);
    }
}

fn draw_welcome_screen(f: &mut Frame, _app: &App, area: Rect) {
    let welcome_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "     No modules are currently loaded in this dashboard.",
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "     Press [A] ",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("to open the Ratatui module catalog and add items."),
        ]),
        Line::from(vec![
            Span::styled(
                "     Press [D] ",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.danger)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("to remove/delete the currently focused pane."),
        ]),
        Line::from(vec![
            Span::styled(
                "     Press [Tab] / Left-Right Arrows ",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("to navigate focused panes."),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "   ────────────────────────────────────────────────────────",
            Style::default().fg(CATPPUCCIN_MOCHA.border),
        )),
        Line::from(Span::styled(
            "     💡 Hint: Panes will auto-split horizontally & vertically!",
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        )),
    ];

    let p = Paragraph::new(welcome_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
                .title(Span::styled(
                    " Tiling Workspace ",
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.text)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel))
        .alignment(Alignment::Left);

    f.render_widget(p, area);
}

fn draw_add_widget_modal(f: &mut Frame, app: &App, area: Rect) {
    let filtered_items = get_filtered_catalog_items(&app.widget_search_input);

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "Select Ratatui Module to Add to Pane",
        Style::default()
            .fg(CATPPUCCIN_MOCHA.accent)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        "───────────────────────────────────────────────────",
        Style::default().fg(CATPPUCCIN_MOCHA.border),
    )));
    lines.push(Line::from(""));

    let search_label = format!(" Search: {}█", app.widget_search_input);
    lines.push(Line::from(Span::styled(
        search_label,
        Style::default().fg(CATPPUCCIN_MOCHA.text),
    )));
    lines.push(Line::from(Span::styled(
        "───────────────────────────────────────────────────",
        Style::default().fg(CATPPUCCIN_MOCHA.border),
    )));
    lines.push(Line::from(""));

    for (idx, (name, desc, _)) in filtered_items.iter().enumerate() {
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
        lines.push(Line::from(vec![
            prefix,
            Span::styled(format!("{:<18} : {}", name, desc), style),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Press [↑/↓] to navigate | [ENTER] to add | [ESC] to close",
        Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
    )));

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(CATPPUCCIN_MOCHA.accent))
                .title(Span::styled(
                    " Ratatui Module Catalog ",
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.text)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    f.render_widget(p, area);
}

// Custom widget drawing functions
fn draw_button_widget(f: &mut Frame, _app: &App, area: Rect, is_focused: bool) {
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
            " button: Ratatui Button Panel ",
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
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .bg(CATPPUCCIN_MOCHA.success)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(
            " [ STOP ] ",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .bg(CATPPUCCIN_MOCHA.danger)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("   "),
        Span::styled(
            " [ RESET ] ",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .bg(CATPPUCCIN_MOCHA.warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(
            " [ PING  ] ",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .bg(CATPPUCCIN_MOCHA.primary)
                .add_modifier(Modifier::BOLD),
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
            " slider: Ratatui PID Parameter Panel ",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let kp_pct = (app.param_kp / 3.0 * 100.0).min(100.0) as usize;
    let ki_pct = (app.param_ki / 1.0 * 100.0).min(100.0) as usize;
    let kd_pct = (app.param_kd / 1.0 * 100.0).min(100.0) as usize;

    let make_track = |pct: usize, color: ratatui::style::Color| {
        let filled = (pct / 10).min(10);
        let empty = 10 - filled;
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
        Span::styled(bar, Style::default().fg(color))
    };

    let mut lines = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Kp: ", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
        make_track(kp_pct, CATPPUCCIN_MOCHA.accent),
        Span::styled(
            format!(" {:.2}", app.param_kp),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Ki: ", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
        make_track(ki_pct, CATPPUCCIN_MOCHA.primary),
        Span::styled(
            format!(" {:.2}", app.param_ki),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Kd: ", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
        make_track(kd_pct, CATPPUCCIN_MOCHA.info),
        Span::styled(
            format!(" {:.2}", app.param_kd),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

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
            " dial: Ratatui Speed Dial ",
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
    lines.push(Line::from(vec![
        Span::raw("      "),
        Span::styled(scale, Style::default().fg(CATPPUCCIN_MOCHA.border_focus)),
    ]));
    lines.push(Line::from(vec![
        Span::raw("      "),
        Span::styled(
            format!("[{}]", arc),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.accent)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
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
                    " joystick: Ratatui Canvas Grid ",
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
            " light: Ratatui Status Indicators ",
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
            " gauge: Ratatui Battery Gauge ",
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
            " dashboard: Ratatui Motor Diagnostics ",
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
            " example: Rust/Ratatui Module Template ",
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

fn draw_delay_widget(f: &mut Frame, _app: &App, area: Rect, is_focused: bool) {
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
            " delay: Ratatui Delayed Trigger ",
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
        Span::raw("  [ "),
        Span::styled(
            "HOLD TO TRIGGER (1.5s)",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .bg(CATPPUCCIN_MOCHA.warning)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" ]"),
    ]));
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

fn draw_toggle_widget(f: &mut Frame, _app: &App, area: Rect, is_focused: bool) {
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
            " toggle: Ratatui Latched Switch ",
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
        Span::raw("  Latching Switch: "),
        Span::styled(
            "  [ ON ]  ",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .bg(CATPPUCCIN_MOCHA.success)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
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
            " knob: Ratatui Fine Dial ",
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
        Span::styled(format!("{:.2} units", value), Style::default().fg(CATPPUCCIN_MOCHA.primary)),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("  [Min] "),
        Span::styled(bar, Style::default().fg(CATPPUCCIN_MOCHA.accent)),
        Span::raw(" [Max]"),
    ]));

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
            " ring: Ratatui Ring Dial ",
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
                    " pad: Ratatui Canvas Input ",
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
