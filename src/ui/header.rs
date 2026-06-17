use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::ui::theme::{CATPPUCCIN_MOCHA, mocha};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let style = Style::default().bg(mocha::MANTLE).fg(CATPPUCCIN_MOCHA.text);

    let header_block = Block::default()
        .style(style)
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border));

    let title = Span::styled(
        " ☕ PIOPULSE v0.1.3 ",
        Style::default()
            .fg(CATPPUCCIN_MOCHA.accent)
            .add_modifier(Modifier::BOLD),
    );

    let lang = &app.tool_config.language;
    let mode_span = if app.admin_mode {
        Span::styled(
            crate::ui::tr("admin_mode_header", lang),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.danger)
                .bg(mocha::SURFACE0)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            crate::ui::tr("operator_mode_header", lang),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.success)
                .bg(mocha::SURFACE0)
                .add_modifier(Modifier::BOLD),
        )
    };

    let header_text = Line::from(vec![
        title,
        Span::styled(" | ", Style::default().fg(CATPPUCCIN_MOCHA.border)),
        mode_span,
    ]);

    let header = Paragraph::new(header_text).block(header_block).style(style);

    f.render_widget(header, area);
}
