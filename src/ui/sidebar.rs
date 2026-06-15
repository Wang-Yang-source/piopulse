use crate::app::App;
use crate::ui::tr;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::ui::theme::{CATPPUCCIN_MOCHA, mocha};

pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(5)])
        .split(area);

    app.layout_zones.monitor_panel = chunks[1];

    draw_stats(f, app, chunks[0]);
    draw_serial_monitor(f, app, chunks[1]);
}

fn draw_stats(f: &mut Frame, app: &App, area: Rect) {
    let lang = &app.tool_config.language;
    let stats_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
        .title(Span::styled(
            tr("sidebar_stats_title", lang),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ));

    let total = app.stats.total_passed + app.stats.total_failed;
    let yield_rate = if total > 0 {
        (app.stats.total_passed as f32 / total as f32) * 100.0
    } else {
        0.0
    };

    let elapsed_str = format!(
        "{:02}:{:02}",
        app.elapsed_time.as_secs() / 60,
        app.elapsed_time.as_secs() % 60
    );

    let stats_text = vec![
        Line::from(vec![
            Span::styled(
                tr("sidebar_total_attempted", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                app.stats.total_attempted.to_string(),
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.text)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                tr("sidebar_passed", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                app.stats.total_passed.to_string(),
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.success)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                tr("sidebar_failed", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                app.stats.total_failed.to_string(),
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.danger)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                tr("sidebar_yield_rate", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                format!("{:.1}%", yield_rate),
                Style::default()
                    .fg(if yield_rate >= 95.0 || total == 0 {
                        CATPPUCCIN_MOCHA.success
                    } else {
                        CATPPUCCIN_MOCHA.danger
                    })
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                tr("sidebar_elapsed_time", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(elapsed_str, Style::default().fg(CATPPUCCIN_MOCHA.text)),
        ]),
    ];

    let inner_area = stats_block.inner(area);
    f.render_widget(stats_block, area);

    let stats_widget = Paragraph::new(stats_text)
        .block(Block::default())
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    f.render_widget(stats_widget, inner_area);
}

fn wrap_log_line(line: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![line.to_string()];
    }
    let mut result = Vec::new();
    let mut current = String::new();

    for word in line.split_whitespace() {
        if current.is_empty() {
            if word.len() <= max_width {
                current.push_str(word);
            } else {
                let chars: Vec<char> = word.chars().collect();
                for chunk in chars.chunks(max_width) {
                    let s: String = chunk.iter().collect();
                    if current.is_empty() {
                        current = s;
                    } else {
                        result.push(current);
                        current = s;
                    }
                }
            }
        } else {
            if current.len() + 1 + word.len() <= max_width {
                current.push(' ');
                current.push_str(word);
            } else {
                result.push(current);
                current = String::new();
                if word.len() <= max_width {
                    current.push_str(word);
                } else {
                    let chars: Vec<char> = word.chars().collect();
                    for chunk in chars.chunks(max_width) {
                        let s: String = chunk.iter().collect();
                        if current.is_empty() {
                            current = s;
                        } else {
                            result.push(current);
                            current = s;
                        }
                    }
                }
            }
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    if result.is_empty() {
        result.push(String::new());
    }
    result
}

fn draw_serial_monitor(f: &mut Frame, app: &App, area: Rect) {
    let lang = &app.tool_config.language;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
        .title(Span::styled(
            tr("sidebar_monitor_title", lang),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let max_width = (inner_area.width as usize).saturating_sub(1);
    let mut wrapped_lines: Vec<Line> = Vec::new();

    for log in &app.logs {
        let fg_color = if log.contains("FAILED") || log.contains("failed") || log.contains("Error")
        {
            CATPPUCCIN_MOCHA.danger
        } else if log.contains("SUCCESS") || log.contains("PASSED") || log.contains("Success") {
            CATPPUCCIN_MOCHA.success
        } else {
            CATPPUCCIN_MOCHA.text_muted
        };

        let sub_lines = wrap_log_line(log, max_width);
        for sub_line in sub_lines {
            wrapped_lines.push(Line::from(Span::styled(
                sub_line,
                Style::default().fg(fg_color),
            )));
        }
    }

    let height = inner_area.height as usize;
    let logs_to_draw = if wrapped_lines.len() > height {
        let start = wrapped_lines.len() - height;
        wrapped_lines[start..].to_vec()
    } else {
        wrapped_lines
    };

    let logs_widget = Paragraph::new(logs_to_draw)
        .block(Block::default())
        .style(Style::default().bg(mocha::MANTLE));

    f.render_widget(logs_widget, inner_area);
}
