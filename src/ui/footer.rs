use crate::app::App;
use crate::ui::theme::{CATPPUCCIN_MOCHA, mocha};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let lang = &app.tool_config.language;
    let space_hint = match app.active_tab {
        crate::app::ActiveTab::Plotter => "N/A",
        crate::app::ActiveTab::Serial => crate::ui::tr("space_hint_serial", lang),
        crate::app::ActiveTab::Flasher => crate::ui::tr("space_hint_flash", lang),
        crate::app::ActiveTab::Configuration => crate::ui::tr("space_hint_flash", lang),
        crate::app::ActiveTab::Widgets => crate::ui::tr("space_hint_focus", lang),
    };

    let width = area.width;
    let mut footer_spans = Vec::new();

    if width < 60 {
        // Ultra-compact mode for tiny screens
        footer_spans.extend([
            Span::styled(
                " 1-5",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if lang == "zh" {
                    "：切页面 | "
                } else {
                    " Tabs | "
                },
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if lang == "zh" { "：菜单" } else { " Menu" },
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
        ]);
    } else if width < 98 {
        // Compact mode: abbreviated descriptions
        let space_label = if lang == "zh" {
            "：执行 | "
        } else {
            " Action | "
        };
        footer_spans.extend([
            Span::styled(
                " F1",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if lang == "zh" {
                    "：管理员 | "
                } else {
                    " Admin | "
                },
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                "Space",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                space_label,
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
        ]);

        if app.active_tab == crate::app::ActiveTab::Widgets {
            footer_spans.extend([
                Span::styled(
                    "A",
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    if lang == "zh" { "：加 | " } else { " Add | " },
                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                ),
                Span::styled(
                    "D",
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    if lang == "zh" { "：删 | " } else { " Del | " },
                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                ),
            ]);
        } else if app.active_tab == crate::app::ActiveTab::Serial {
            footer_spans.extend([
                Span::styled(
                    "p",
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    if lang == "zh" {
                        "：端口 | "
                    } else {
                        " Port | "
                    },
                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                ),
            ]);
        } else if app.active_tab == crate::app::ActiveTab::Flasher {
            footer_spans.extend([
                Span::styled(
                    "b",
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    if lang == "zh" {
                        "：批量 | "
                    } else {
                        " Batch | "
                    },
                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                ),
            ]);
        }

        footer_spans.extend([
            Span::styled(
                "1-5",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if lang == "zh" {
                    "：页面 | "
                } else {
                    " Tabs | "
                },
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if lang == "zh" { "：菜单" } else { " Menu" },
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
        ]);
    } else {
        // Full mode: original complete hints
        footer_spans.extend([
            Span::styled(
                " F1",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                crate::ui::tr("f1_toggle_admin", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                "F2",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                crate::ui::tr("f2_toggle_sidebar", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                "Space",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                crate::ui::tr("space_hint", lang).replace("{}", space_hint),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
        ]);

        if app.active_tab == crate::app::ActiveTab::Widgets {
            footer_spans.extend([
                Span::styled(
                    "A",
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    crate::ui::tr("add_module", lang),
                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                ),
                Span::styled(
                    "D",
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    crate::ui::tr("delete_module", lang),
                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                ),
            ]);
        }

        if app.active_tab == crate::app::ActiveTab::Serial {
            footer_spans.extend([
                Span::styled(
                    "p",
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    if lang == "zh" {
                        "切换端口 "
                    } else {
                        "Switch Port "
                    },
                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                ),
                Span::styled(
                    "f",
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    if lang == "zh" {
                        "切换格式 "
                    } else {
                        "Switch Format "
                    },
                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                ),
            ]);
        }

        if app.active_tab == crate::app::ActiveTab::Flasher {
            footer_spans.extend([
                Span::styled(
                    "b",
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    if lang == "zh" {
                        "批量烧录 "
                    } else {
                        "Batch Flash "
                    },
                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                ),
                Span::styled(
                    "a",
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    if lang == "zh" {
                        "自动感应 "
                    } else {
                        "Auto-Flash "
                    },
                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                ),
            ]);
        }

        footer_spans.extend([
            Span::styled(
                "c",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                crate::ui::tr("clear_stats", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                "1/2/3/4/5",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                crate::ui::tr("tabs_nav", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                crate::ui::tr("menu_hint", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
        ]);
    }

    let footer_text = Line::from(footer_spans);
    f.render_widget(
        Paragraph::new(footer_text).style(Style::default().bg(mocha::MANTLE)),
        area,
    );
}
