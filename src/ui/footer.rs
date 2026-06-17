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

    let mut footer_spans = vec![
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
    ];

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
    let footer_text = Line::from(footer_spans);
    f.render_widget(
        Paragraph::new(footer_text).style(Style::default().bg(mocha::MANTLE)),
        area,
    );
}
