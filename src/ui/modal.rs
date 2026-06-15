use crate::app::App;
use crate::ui::theme::{CATPPUCCIN_MOCHA, mocha};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            " Admin Authorization Required ",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.danger))
        .style(Style::default().bg(mocha::MANTLE));

    let inner_area = block.inner(area);

    // Dim background
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let text_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title line
            Constraint::Length(3), // Input box
            Constraint::Length(1), // Help / Cancel line
            Constraint::Length(1), // Error line
            Constraint::Min(0),
        ])
        .split(inner_area);

    let msg = Paragraph::new("Enter system sudo password to authorize:")
        .style(Style::default().fg(CATPPUCCIN_MOCHA.text).bg(mocha::MANTLE));
    f.render_widget(msg, text_chunks[0]);

    // Mask password characters
    let masked_input: String = std::iter::repeat('*')
        .take(app.password_input.len())
        .collect();
    let input_widget = Paragraph::new(masked_input)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border_focus)),
        )
        .style(Style::default().fg(CATPPUCCIN_MOCHA.text).bg(mocha::MANTLE));
    f.render_widget(input_widget, text_chunks[1]);

    let cancel_msg = Paragraph::new("Press Enter to submit | Click outside / Esc to cancel").style(
        Style::default()
            .fg(CATPPUCCIN_MOCHA.text_muted)
            .bg(mocha::MANTLE),
    );
    f.render_widget(cancel_msg, text_chunks[2]);

    if app.password_incorrect {
        let err_msg = Paragraph::new("Incorrect password. Press Enter to retry.").style(
            Style::default()
                .fg(CATPPUCCIN_MOCHA.danger)
                .bg(mocha::MANTLE),
        );
        f.render_widget(err_msg, text_chunks[3]);
    }
}

pub fn draw_exit_menu(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border_focus))
        .style(Style::default().bg(mocha::MANTLE));

    let inner_area = block.inner(area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    // Layout inside modal: Title (1), Spacer/Text (2), Cards (3), Spacer/Hint (2)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Length(2), // Text question
            Constraint::Length(3), // Two side-by-side cards
            Constraint::Min(0),    // Hint
        ])
        .split(inner_area);

    // 1. Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled("[!] ", Style::default()),
        Span::styled(
            "EXIT CONFIRMATION",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.accent)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center)
    .style(Style::default().bg(mocha::MANTLE));
    f.render_widget(title, chunks[0]);

    // 2. Text question
    let question = Paragraph::new("Choose an action below to proceed:")
        .alignment(Alignment::Center)
        .style(Style::default().fg(CATPPUCCIN_MOCHA.text).bg(mocha::MANTLE));
    f.render_widget(question, chunks[1]);

    // 3. Option Cards
    let card_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    // Card 0: Settings
    let is_settings_selected = app.exit_menu_selected == 0;
    let settings_border_style = if is_settings_selected {
        Style::default().fg(CATPPUCCIN_MOCHA.border_focus)
    } else {
        Style::default().fg(CATPPUCCIN_MOCHA.border)
    };
    let settings_block = Block::default()
        .borders(Borders::ALL)
        .border_type(if is_settings_selected {
            BorderType::Double
        } else {
            BorderType::Rounded
        })
        .border_style(settings_border_style)
        .style(if is_settings_selected {
            Style::default().bg(CATPPUCCIN_MOCHA.selection_bg)
        } else {
            Style::default().bg(mocha::MANTLE)
        });
    let settings_text = Paragraph::new("Settings")
        .alignment(Alignment::Center)
        .block(settings_block)
        .style(if is_settings_selected {
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted)
        });
    f.render_widget(settings_text, card_chunks[0]);

    // Card 1: Quit
    let is_quit_selected = app.exit_menu_selected == 1;
    let quit_border_style = if is_quit_selected {
        Style::default().fg(CATPPUCCIN_MOCHA.danger)
    } else {
        Style::default().fg(CATPPUCCIN_MOCHA.border)
    };
    let quit_block = Block::default()
        .borders(Borders::ALL)
        .border_type(if is_quit_selected {
            BorderType::Double
        } else {
            BorderType::Rounded
        })
        .border_style(quit_border_style)
        .style(if is_quit_selected {
            Style::default().bg(CATPPUCCIN_MOCHA.selection_bg)
        } else {
            Style::default().bg(mocha::MANTLE)
        });
    let quit_text = Paragraph::new("Quit App")
        .alignment(Alignment::Center)
        .block(quit_block)
        .style(if is_quit_selected {
            Style::default()
                .fg(CATPPUCCIN_MOCHA.danger)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted)
        });
    f.render_widget(quit_text, card_chunks[1]);

    // 4. Hint
    let hint = Paragraph::new("Esc/Click outside: Cancel | Tab/Arrows: Move | Enter: Confirm")
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text_muted)
                .bg(mocha::MANTLE),
        );
    f.render_widget(hint, chunks[3]);
}
