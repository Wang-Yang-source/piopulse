pub mod channels;
pub mod config;
pub mod footer;
pub mod header;
pub mod modal;
pub mod plotter;
pub mod serial;
pub mod sidebar;
pub mod theme;
pub mod translation;
pub mod utils;
pub mod widgets;

pub use translation::tr;
pub use utils::center_rect;

use crate::app::{ActiveTab, App};
use crate::ui::theme::{CATPPUCCIN_MOCHA, mocha};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::canvas::Canvas,
    widgets::{Block, Borders, Clear, Paragraph},
};

pub fn draw(f: &mut Frame, app: &mut App) {
    if app.splash_ticks_remaining.is_some() {
        draw_splash_screen(f, app);
        return;
    }
    // Main layout: Vertical split for Header, Main Body, Footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Header
            Constraint::Min(10),   // Main Area
            Constraint::Length(1), // Footer
        ])
        .split(f.size());

    app.layout_zones.header = chunks[0];

    header::draw(f, app, chunks[0]);
    draw_main_area(f, app, chunks[1]);
    footer::draw(f, app, chunks[2]);

    if app.show_exit_menu {
        let area = center_rect(48, 11, f.size());
        app.layout_zones.exit_menu_modal = area;
        modal::draw_exit_menu(f, app, area);
    }

    if app.is_entering_password {
        let area = center_rect(45, 11, f.size());
        app.layout_zones.password_modal = area;
        modal::draw(f, app, area);
    }

    if app.show_tool_settings {
        let area = center_rect(48, 11, f.size());
        app.layout_zones.tool_settings_modal = area;
        modal::draw_tool_settings(f, app, area);
    }

    if app.show_port_menu {
        let port_count = app.channels.len() + 1;
        let height = (port_count + 4).clamp(6, 15) as u16;
        let area = center_rect(50, height, f.size());
        app.layout_zones.port_menu_modal = area;
        modal::draw_port_menu(f, app, area);
    }

    if app.flash_success_ticks_remaining.is_some() {
        draw_flash_success_animation(f, app);
    }
}

fn draw_main_area(f: &mut Frame, app: &mut App, area: Rect) {
    let can_show_sidebar = area.width >= 110 && area.height >= 22;
    if app.active_tab == ActiveTab::Flasher && app.show_sidebar && can_show_sidebar {
        // Horizontal split: Left Workspace (70%), Right Panel (30%)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);

        draw_workspace(f, app, chunks[0]);
        sidebar::draw(f, app, chunks[1]);
    } else {
        // Workspace takes full screen width on all other tabs, or when sidebar is hidden
        draw_workspace(f, app, area);
    }
}

fn draw_workspace(f: &mut Frame, app: &mut App, area: Rect) {
    // Vertical split for Tabs Bar and Content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(5)])
        .split(area);

    // Render Tabs
    let lang = &app.tool_config.language;
    let tab_titles = tab_titles_for_width(lang, chunks[0].width);

    let active_index = match app.active_tab {
        ActiveTab::Serial => 0,
        ActiveTab::Plotter => 1,
        ActiveTab::Widgets => 2,
        ActiveTab::Flasher => 3,
        ActiveTab::Configuration => 4,
    };

    let mut tab_spans = Vec::new();
    for (idx, title) in tab_titles.iter().enumerate() {
        let is_active = active_index == idx;
        let is_hovered = app.hover_tab == Some(idx);
        let style = if is_hovered {
            Style::default()
                .fg(if is_active {
                    theme::mocha::CRUST
                } else {
                    theme::CATPPUCCIN_MOCHA.text
                })
                .bg(if is_active {
                    theme::CATPPUCCIN_MOCHA.accent
                } else {
                    theme::CATPPUCCIN_MOCHA.selection_bg
                })
                .add_modifier(Modifier::BOLD)
        } else if is_active {
            Style::default()
                .fg(theme::CATPPUCCIN_MOCHA.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::CATPPUCCIN_MOCHA.text_muted)
        };

        tab_spans.push(Span::styled(*title, style));
        if idx + 1 < tab_titles.len() {
            tab_spans.push(Span::styled(
                " | ",
                Style::default().fg(theme::CATPPUCCIN_MOCHA.border),
            ));
        }
    }

    let tabs = Paragraph::new(Line::from(tab_spans))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(theme::CATPPUCCIN_MOCHA.border)),
        )
        .style(Style::default().bg(theme::mocha::MANTLE));

    f.render_widget(tabs, chunks[0]);
    app.layout_zones.tabs = chunks[0];

    // Render Tab Content
    match app.active_tab {
        ActiveTab::Serial => serial::draw(f, app, chunks[1]),
        ActiveTab::Plotter => plotter::draw(f, app, chunks[1]),
        ActiveTab::Flasher => channels::draw(f, app, chunks[1]),
        ActiveTab::Configuration => {
            app.layout_zones.config_table = chunks[1];
            config::draw(f, app, chunks[1]);
        }
        ActiveTab::Widgets => {
            app.layout_zones.monitor_panel = chunks[1];
            widgets::draw(f, app, chunks[1]);
        }
    }
}

pub fn tab_titles_for_width(lang: &str, width: u16) -> [&'static str; 5] {
    use unicode_width::UnicodeWidthStr;

    let full = [
        tr("tab_serial", lang),
        tr("tab_plot", lang),
        tr("tab_dash", lang),
        tr("tab_flash", lang),
        tr("tab_settings", lang),
    ];
    let compact = [" [1] ", " [2] ", " [3] ", " [4] ", " [5] "];
    let full_width: usize = full
        .iter()
        .map(|title| UnicodeWidthStr::width(*title))
        .sum();
    let separator_width = UnicodeWidthStr::width(" | ") * 4;

    if full_width + separator_width <= width as usize {
        full
    } else {
        compact
    }
}

fn draw_splash_screen(f: &mut Frame, app: &App) {
    let area = f.size();

    // Clear screen with Mantle background
    f.render_widget(Clear, area);
    let bg_block = Block::default().style(Style::default().bg(mocha::MANTLE));
    f.render_widget(bg_block, area);

    // Split area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top margin
            Constraint::Length(6), // ASCII Title (needs 6 lines)
            Constraint::Length(1), // Subtitle
            Constraint::Min(5),    // Galaxy Animation
            Constraint::Length(2), // Skip hint
        ])
        .split(area);

    let logo = vec![
        Line::from(Span::styled(
            " ██████╗ ██╗ ██████╗ ██████╗ ██╗   ██╗██╗     ███████╗",
            Style::default().fg(CATPPUCCIN_MOCHA.accent),
        )),
        Line::from(Span::styled(
            " ██╔══██╗██║██╔═══██╗██╔══██╗██║   ██║██║     ██╔════╝",
            Style::default().fg(CATPPUCCIN_MOCHA.primary),
        )),
        Line::from(Span::styled(
            " ██████╔╝██║██║   ██║██████╔╝██║   ██║██║     █████╗  ",
            Style::default().fg(CATPPUCCIN_MOCHA.info),
        )),
        Line::from(Span::styled(
            " ██╔═══╝ ██║██║   ██║██╔═══╝ ██║   ██║██║     ██╔══╝  ",
            Style::default().fg(CATPPUCCIN_MOCHA.success),
        )),
        Line::from(Span::styled(
            " ██║     ██║╚██████╔╝██║     ╚██████╔╝███████╗███████╗",
            Style::default().fg(CATPPUCCIN_MOCHA.warning),
        )),
        Line::from(Span::styled(
            " ╚═╝     ╚═╝ ╚═════╝ ╚═╝      ╚═════╝ ╚══════╝╚══════╝",
            Style::default().fg(CATPPUCCIN_MOCHA.danger),
        )),
    ];

    let logo_paragraph = Paragraph::new(logo).alignment(ratatui::layout::Alignment::Center);
    f.render_widget(logo_paragraph, chunks[1]);

    let subtitle = Paragraph::new(Line::from(vec![
        Span::styled("▼ ", Style::default().fg(CATPPUCCIN_MOCHA.success)),
        Span::styled(
            "Terminal Embedded Debugger & Flashing Tool v0.2.1",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text_muted)
                .add_modifier(Modifier::ITALIC),
        ),
    ]))
    .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(subtitle, chunks[2]);

    // Render 3D rotating wireframe globe on Canvas
    let anim_area = chunks[3];
    let aspect = (anim_area.width as f64 / anim_area.height as f64) * 0.5;
    let x_limit = 12.0 * aspect;

    let canvas = Canvas::default()
        .x_bounds([-x_limit, x_limit])
        .y_bounds([-12.0, 12.0])
        .paint(|ctx| {
            let t = app.anim_tick as f64;
            let pitch: f64 = 0.45; // tilt
            let yaw: f64 = t * 0.08; // rotation speed

            // Parallels (latitude lines)
            for lat_idx in -4..=4 {
                let lat = (lat_idx as f64) * (std::f64::consts::PI / 10.0);
                let z = lat.sin() * 6.0;
                let r_lat = lat.cos() * 6.0;

                let mut prev_point: Option<(f64, f64)> = None;
                let steps = 30;
                for step in 0..=steps {
                    let lon = (step as f64) * (2.0 * std::f64::consts::PI / steps as f64);
                    let x = r_lat * lon.cos();
                    let y = r_lat * lon.sin();

                    // Rotate Z (yaw)
                    let x1 = x * yaw.cos() - y * yaw.sin();
                    let y1 = x * yaw.sin() + y * yaw.cos();
                    let z1 = z;

                    // Rotate X (pitch)
                    let x_rot = x1;
                    let y_rot = y1 * pitch.cos() - z1 * pitch.sin();
                    let z_rot = y1 * pitch.sin() + z1 * pitch.cos();

                    // Perspective projection
                    let dist = 15.0;
                    let scale = 15.0 / (dist - z_rot);
                    let sx = x_rot * scale;
                    let sy = y_rot * scale;

                    // Color based on whether it is in front or back
                    let color = if z_rot > 0.0 {
                        CATPPUCCIN_MOCHA.primary
                    } else {
                        CATPPUCCIN_MOCHA.border // faded color for back-side
                    };

                    if let Some((px, py)) = prev_point {
                        ctx.draw(&ratatui::widgets::canvas::Line {
                            x1: px,
                            y1: py,
                            x2: sx,
                            y2: sy,
                            color,
                        });
                    }
                    prev_point = Some((sx, sy));
                }
            }

            // Meridians (longitude lines)
            for lon_idx in 0..8 {
                let lon = (lon_idx as f64) * (std::f64::consts::PI / 8.0);
                let mut prev_point: Option<(f64, f64)> = None;
                let steps = 24;
                for step in 0..=steps {
                    let lat = (step as f64) * (2.0 * std::f64::consts::PI / steps as f64)
                        - std::f64::consts::PI;
                    let z = lat.sin() * 6.0;
                    let r_lat = lat.cos() * 6.0;

                    let x = r_lat * lon.cos();
                    let y = r_lat * lon.sin();

                    // Rotate Z (yaw)
                    let x1 = x * yaw.cos() - y * yaw.sin();
                    let y1 = x * yaw.sin() + y * yaw.cos();
                    let z1 = z;

                    // Rotate X (pitch)
                    let x_rot = x1;
                    let y_rot = y1 * pitch.cos() - z1 * pitch.sin();
                    let z_rot = y1 * pitch.sin() + z1 * pitch.cos();

                    // Perspective projection
                    let dist = 15.0;
                    let scale = 15.0 / (dist - z_rot);
                    let sx = x_rot * scale;
                    let sy = y_rot * scale;

                    let color = if z_rot > 0.0 {
                        CATPPUCCIN_MOCHA.accent
                    } else {
                        CATPPUCCIN_MOCHA.border
                    };

                    if let Some((px, py)) = prev_point {
                        ctx.draw(&ratatui::widgets::canvas::Line {
                            x1: px,
                            y1: py,
                            x2: sx,
                            y2: sy,
                            color,
                        });
                    }
                    prev_point = Some((sx, sy));
                }
            }
        });
    f.render_widget(canvas, chunks[3]);

    let lang = &app.tool_config.language;
    let skip_text = if lang == "zh" {
        "按任意键跳过启动动画..."
    } else {
        "Press any key to skip splash screen..."
    };
    let skip_hint = Paragraph::new(Line::from(vec![Span::styled(
        skip_text,
        Style::default().fg(CATPPUCCIN_MOCHA.text_disabled),
    )]))
    .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(skip_hint, chunks[4]);
}

fn draw_flash_success_animation(f: &mut Frame, app: &App) {
    let area = f.size();
    
    // We want a centered modal dialog
    let modal_area = center_rect(50, 15, area);
    
    // Clear the background of the modal
    f.render_widget(Clear, modal_area);
    
    let border_style = Style::default().fg(CATPPUCCIN_MOCHA.success);
    let success_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Double)
        .border_style(border_style)
        .title(Span::styled(" FLASH SUCCESS 🎉 ", Style::default().fg(CATPPUCCIN_MOCHA.success).add_modifier(Modifier::BOLD)));
        
    f.render_widget(success_block.clone(), modal_area);
    
    // Inner area for canvas animation
    let inner_area = success_block.inner(modal_area);
    
    // Let's paint a beautiful radiating checkmark and pulse waves!
    let aspect = (inner_area.width as f64 / inner_area.height as f64) * 0.5;
    let x_limit = 10.0 * aspect;
    
    let canvas = Canvas::default()
        .x_bounds([-x_limit, x_limit])
        .y_bounds([-5.0, 5.0])
        .paint(move |ctx| {
            let ticks = app.flash_success_ticks_remaining.unwrap_or(0) as f64;
            // progress of animation goes from 0.0 (end) to 1.0 (start)
            let progress = (30.0 - ticks) / 30.0;
            
            // Draw a big green checkmark in the center
            // Line 1: from (-2, -0.5) to (0, -2.5)
            // Line 2: from (0, -2.5) to (3.5, 2.0)
            ctx.draw(&ratatui::widgets::canvas::Line {
                x1: -2.0,
                y1: -0.5,
                x2: 0.0,
                y2: -2.5,
                color: CATPPUCCIN_MOCHA.success,
            });
            ctx.draw(&ratatui::widgets::canvas::Line {
                x1: 0.0,
                y1: -2.5,
                x2: 3.5,
                y2: 2.0,
                color: CATPPUCCIN_MOCHA.success,
            });
            
            // Draw radiating pulse rings
            let num_rings = 3;
            for r in 0..num_rings {
                let ring_progress = progress + (r as f64 * 0.3);
                let radius = (ring_progress % 1.0) * 8.0;
                
                // Draw a circle on canvas
                let steps = 32;
                let mut prev_point: Option<(f64, f64)> = None;
                for step in 0..=steps {
                    let theta = (step as f64) * (2.0 * std::f64::consts::PI / steps as f64);
                    let rx = theta.cos() * radius;
                    let ry = theta.sin() * radius * 0.5; // aspect compression
                    
                    let color = if ring_progress % 1.0 < 0.8 {
                        CATPPUCCIN_MOCHA.success
                    } else {
                        CATPPUCCIN_MOCHA.border
                    };
                    
                    if let Some((px, py)) = prev_point {
                        ctx.draw(&ratatui::widgets::canvas::Line {
                            x1: px,
                            y1: py,
                            x2: rx,
                            y2: ry,
                            color,
                        });
                    }
                    prev_point = Some((rx, ry));
                }
            }
            
            // Draw text
            let is_zh = app.tool_config.language == "zh";
            let msg = if is_zh { "烧录成功!" } else { "FLASH COMPLETED!" };
            ctx.print(-3.5, 3.5, Span::styled(msg, Style::default().fg(CATPPUCCIN_MOCHA.success).add_modifier(Modifier::BOLD)));
        });
        
    f.render_widget(canvas, inner_area);
}
