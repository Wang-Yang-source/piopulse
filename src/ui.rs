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
pub use utils::{center_rect, responsive_modal_rect};

use crate::app::{ActiveTab, App};
use crate::ui::theme::{CATPPUCCIN_MOCHA, mocha};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use std::time::Duration;

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
        let area = responsive_modal_rect(48, 11, f.size());
        app.layout_zones.exit_menu_modal = area;
        modal::draw_exit_menu(f, app, area);
    }

    if app.is_entering_password {
        let area = responsive_modal_rect(45, 11, f.size());
        app.layout_zones.password_modal = area;
        modal::draw(f, app, area);
    }

    if app.show_tool_settings {
        let area = responsive_modal_rect(48, 11, f.size());
        app.layout_zones.tool_settings_modal = area;
        modal::draw_tool_settings(f, app, area);
    }

    if app.show_port_menu {
        let port_count = app.channels.len() + 1;
        let height = (port_count + 4).clamp(6, 15) as u16;
        let area = responsive_modal_rect(50, height, f.size());
        app.layout_zones.port_menu_modal = area;
        modal::draw_port_menu(f, app, area);
    }

    if app.show_custom_baud_modal {
        let area = responsive_modal_rect(42, 11, f.size());
        app.layout_zones.custom_baud_modal = area;
        modal::draw_custom_baud(f, app, area);
    }

    if app.show_auto_reply_modal {
        let area = responsive_modal_rect(52, 12, f.size());
        app.layout_zones.auto_reply_modal = area;
        modal::draw_auto_reply(f, app, area);
    }

    if app.show_manifest_delete_confirm {
        let area = responsive_modal_rect(52, 9, f.size());
        app.layout_zones.manifest_delete_modal = area;
        modal::draw_manifest_delete_confirm(f, app, area);
    }

    if app.show_manifest_edit_modal {
        let area = responsive_modal_rect(52, 9, f.size());
        app.layout_zones.manifest_edit_modal = area;
        modal::draw_manifest_edit(f, app, area);
    }

    if app.show_file_picker {
        let area = responsive_modal_rect(70, 18, f.size());
        app.layout_zones.file_picker_modal = area;
        modal::draw_file_picker(f, app, area);
    }
}

pub fn draw_startup_screen(f: &mut Frame, lang: &str, elapsed: Duration) {
    let is_zh = lang == "zh";
    let status = if is_zh {
        "正在准备工作区"
    } else {
        "Preparing workspace"
    };
    let detail = if is_zh {
        format!(
            "正在读取配置、检测 PlatformIO 和串口设备  |  已用 {}s",
            elapsed.as_secs()
        )
    } else {
        format!(
            "Loading config, detecting PlatformIO and scanning serial ports  |  {}s elapsed",
            elapsed.as_secs()
        )
    };
    draw_logo_progress_screen(
        f,
        lang,
        status,
        None,
        Some(&detail),
        if is_zh { "请稍候" } else { "Please wait" },
        ProgressBarMode::Indeterminate(elapsed),
    );
}

fn draw_main_area(f: &mut Frame, app: &mut App, area: Rect) {
    let can_show_sidebar = area.width >= 120 && area.height >= 22;
    if app.active_tab == ActiveTab::Flasher && app.show_sidebar && can_show_sidebar {
        // Horizontal split: Left Workspace (flexible), Right Panel (fixed 56 cols for guided manifest)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(80), Constraint::Length(56)])
            .split(area);

        draw_workspace(f, app, chunks[0]);
        sidebar::draw(f, app, chunks[1]);
    } else {
        // Workspace takes full screen width on all other tabs, or when sidebar is hidden
        app.layout_zones.flash_summary = Rect::default();
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
    let tiny = ["1", "2", "3", "4", "5"];
    let full_width: usize = full
        .iter()
        .map(|title| UnicodeWidthStr::width(*title))
        .sum();
    let separator_width = UnicodeWidthStr::width(" | ") * 4;
    let compact_width: usize = compact
        .iter()
        .map(|title| UnicodeWidthStr::width(*title))
        .sum::<usize>()
        + separator_width;

    if full_width + separator_width <= width as usize {
        full
    } else if compact_width <= width as usize {
        compact
    } else {
        tiny
    }
}

enum ProgressBarMode {
    Determinate { progress: usize, total: usize },
    Indeterminate(Duration),
}

fn draw_splash_screen(f: &mut Frame, app: &App) {
    let ticks_left = app.splash_ticks_remaining.unwrap_or(0);
    let total_ticks = crate::app::SPLASH_TICKS;
    let progress = total_ticks.saturating_sub(ticks_left).min(total_ticks);
    let lang = &app.tool_config.language;
    let is_zh = lang == "zh";
    draw_logo_progress_screen(
        f,
        lang,
        if is_zh {
            "正在初始化工作区"
        } else {
            "Initializing workspace"
        },
        Some(format!("{:>3}%", progress * 100 / total_ticks.max(1))),
        Some(if is_zh {
            "串口  |  波形  |  烧录  |  配置"
        } else {
            "Serial  |  Plotter  |  Flash  |  Config"
        }),
        if is_zh {
            "按任意键跳过"
        } else {
            "Press any key to skip"
        },
        ProgressBarMode::Determinate {
            progress,
            total: total_ticks,
        },
    );
}

fn draw_logo_progress_screen(
    f: &mut Frame,
    lang: &str,
    status: &str,
    progress_label: Option<String>,
    detail: Option<&str>,
    hint: &str,
    progress_mode: ProgressBarMode,
) {
    let area = f.size();
    let splash_bg = mocha::BASE;
    let splash_panel = mocha::MANTLE;

    // Paint every splash cell with an explicit color so transparent terminals do not show through.
    f.render_widget(Clear, area);
    let bg_block = Block::default().style(Style::default().bg(splash_bg));
    f.render_widget(bg_block, area);

    let panel_area = if area.width < 76 || area.height < 16 {
        area
    } else {
        center_rect(72, 15, area)
    };
    f.render_widget(Clear, panel_area);
    let panel = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border_focus))
        .style(Style::default().bg(splash_panel));
    let inner = panel.inner(panel_area);
    f.render_widget(panel, panel_area);

    let progress_width = inner.width.saturating_sub(10).clamp(12, 36) as usize;
    let progress_bar = match progress_mode {
        ProgressBarMode::Determinate { progress, total } => {
            let filled = progress_width * progress / total.max(1);
            format!(
                "[{}{}]",
                "=".repeat(filled),
                " ".repeat(progress_width.saturating_sub(filled))
            )
        }
        ProgressBarMode::Indeterminate(elapsed) => {
            let marker_width = progress_width.clamp(6, 12);
            let travel = progress_width + marker_width;
            let offset = ((elapsed.as_millis() / 90) as usize) % travel;
            let mut cells = vec![' '; progress_width];
            for idx in 0..marker_width {
                if let Some(cell) = offset.checked_add(idx).and_then(|pos| {
                    let bar_pos = pos as isize - marker_width as isize;
                    (bar_pos >= 0 && (bar_pos as usize) < progress_width)
                        .then_some(bar_pos as usize)
                }) {
                    cells[cell] = '=';
                }
            }
            format!("[{}]", cells.into_iter().collect::<String>())
        }
    };

    let is_zh = lang == "zh";
    let subtitle = if is_zh {
        "嵌入式串口监视与烧录工具"
    } else {
        "Embedded serial monitor and flashing tool"
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(inner);

    for chunk in chunks.iter() {
        f.render_widget(
            Block::default().style(Style::default().bg(splash_panel)),
            *chunk,
        );
    }

    let logo = vec![
        Line::from(Span::styled(
            " ██████╗ ██╗ ██████╗ ██████╗ ██╗   ██╗██╗     ███████╗",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.accent)
                .bg(splash_panel),
        )),
        Line::from(Span::styled(
            " ██╔══██╗██║██╔═══██╗██╔══██╗██║   ██║██║     ██╔════╝",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.primary)
                .bg(splash_panel),
        )),
        Line::from(Span::styled(
            " ██████╔╝██║██║   ██║██████╔╝██║   ██║██║     █████╗  ",
            Style::default().fg(CATPPUCCIN_MOCHA.info).bg(splash_panel),
        )),
        Line::from(Span::styled(
            " ██╔═══╝ ██║██║   ██║██╔═══╝ ██║   ██║██║     ██╔══╝  ",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.success)
                .bg(splash_panel),
        )),
        Line::from(Span::styled(
            " ██║     ██║╚██████╔╝██║     ╚██████╔╝███████╗███████╗",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.warning)
                .bg(splash_panel),
        )),
        Line::from(Span::styled(
            " ╚═╝     ╚═╝ ╚═════╝ ╚═╝      ╚═════╝ ╚══════╝╚══════╝",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.danger)
                .bg(splash_panel),
        )),
    ];

    let title = Paragraph::new(logo)
        .alignment(ratatui::layout::Alignment::Center)
        .style(Style::default().bg(splash_panel));
    f.render_widget(title, chunks[1]);

    let subtitle = Paragraph::new(Span::styled(
        subtitle,
        Style::default()
            .fg(CATPPUCCIN_MOCHA.text_muted)
            .bg(splash_panel),
    ))
    .alignment(ratatui::layout::Alignment::Center)
    .style(Style::default().bg(splash_panel));
    f.render_widget(subtitle, chunks[2]);

    let status_line = Paragraph::new(Line::from(vec![
        Span::styled(
            status,
            Style::default().fg(CATPPUCCIN_MOCHA.text).bg(splash_panel),
        ),
        Span::styled(
            "  ",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text_muted)
                .bg(splash_panel),
        ),
        Span::styled(
            progress_label.unwrap_or_default(),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.success)
                .bg(splash_panel)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(ratatui::layout::Alignment::Center)
    .style(Style::default().bg(splash_panel));
    f.render_widget(status_line, chunks[3]);

    let bar = Paragraph::new(Span::styled(
        progress_bar,
        Style::default()
            .fg(CATPPUCCIN_MOCHA.success)
            .bg(splash_panel),
    ))
    .alignment(ratatui::layout::Alignment::Center)
    .style(Style::default().bg(splash_panel));
    f.render_widget(bar, chunks[4]);

    let detail = Paragraph::new(Span::styled(
        detail.unwrap_or(""),
        Style::default()
            .fg(CATPPUCCIN_MOCHA.text_disabled)
            .bg(splash_panel),
    ))
    .alignment(ratatui::layout::Alignment::Center)
    .style(Style::default().bg(splash_panel));
    f.render_widget(detail, chunks[5]);

    let skip_hint = Paragraph::new(Span::styled(
        hint,
        Style::default()
            .fg(CATPPUCCIN_MOCHA.text_disabled)
            .bg(splash_panel),
    ))
    .alignment(ratatui::layout::Alignment::Center)
    .style(Style::default().bg(splash_panel));
    f.render_widget(skip_hint, chunks[6]);
}
