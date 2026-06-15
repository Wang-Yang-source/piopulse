use crate::app::App;
use crate::ui::theme::{mocha, CATPPUCCIN_MOCHA};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let logs_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
        .title(" System Event Log ");

    let list_items: Vec<ListItem> = app
        .logs
        .iter()
        .rev() // Show latest logs at the top
        .skip(app.logs_scroll_offset)
        .map(|log| {
            // Give specific coloring to certain messages
            let style = if log.contains("FAILED") || log.contains("Error") {
                Style::default().fg(CATPPUCCIN_MOCHA.danger)
            } else if log.contains("PASSED") || log.contains("Success") {
                Style::default().fg(CATPPUCCIN_MOCHA.success)
            } else if log.contains("Start Batch") {
                Style::default().fg(CATPPUCCIN_MOCHA.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(CATPPUCCIN_MOCHA.text)
            };
            ListItem::new(log.as_str()).style(style)
        })
        .collect();

    let list = List::new(list_items)
        .block(logs_block)
        .style(Style::default().bg(mocha::MANTLE));
        
    f.render_widget(list, area);
}
