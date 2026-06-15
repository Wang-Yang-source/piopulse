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
    let space_hint = match app.active_tab {
        crate::app::ActiveTab::Plotter => "N/A",
        crate::app::ActiveTab::Serial => "Type Space",
        crate::app::ActiveTab::Flasher => "Start Flash",
        crate::app::ActiveTab::Configuration => "Start Flash",
        crate::app::ActiveTab::Widgets => "Focus",
    };

    let mut footer_spans = vec![
        Span::styled(
            " F1",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            ": Toggle Admin | ",
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ),
        Span::styled(
            "F2",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            ": Toggle Sidebar | ",
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ),
        Span::styled(
            "Space",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(": {} | ", space_hint),
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
                ": Add Module | ",
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                "D",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                ": Delete Module | ",
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
            ": Clear Stats | ",
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ),
        Span::styled(
            "1/2/3/4/5",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            ": Tabs | ",
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ),
        Span::styled(
            "Esc",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(": Menu", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
    ]);
    let footer_text = Line::from(footer_spans);
    f.render_widget(
        Paragraph::new(footer_text).style(Style::default().bg(mocha::MANTLE)),
        area,
    );
}
