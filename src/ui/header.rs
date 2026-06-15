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
        " ☕ PIOPULSE FLASHER v0.1.2 ",
        Style::default()
            .fg(CATPPUCCIN_MOCHA.accent)
            .add_modifier(Modifier::BOLD),
    );

    let mode_span = if app.admin_mode {
        Span::styled(
            " [ADMIN MODE] ",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.danger)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            " [OPERATOR MODE] ",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.success)
                .add_modifier(Modifier::BOLD),
        )
    };

    let header_text = Line::from(vec![title, Span::raw(" | "), mode_span]);

    let header = Paragraph::new(header_text).block(header_block).style(style);

    f.render_widget(header, area);
}
