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
        .constraints([Constraint::Length(8), Constraint::Min(5)])
        .split(area);

    app.layout_zones.monitor_panel = chunks[1];
    app.layout_zones.flash_summary = chunks[0];

    draw_stats(f, app, chunks[0]);
    crate::ui::channels::draw_guided_burning_panel(f, app, chunks[1]);
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
    let inner_area = stats_block.inner(area);

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

    let total_str = app.stats.total_attempted.to_string();
    let yield_str = format!("{:.1}%", yield_rate);
    let passed_str = app.stats.total_passed.to_string();
    let failed_str = app.stats.total_failed.to_string();

    let attempt_label = tr("sidebar_total_attempted", lang);
    let yield_label = tr("sidebar_yield_rate", lang);
    let passed_label = tr("sidebar_passed", lang);
    let failed_label = tr("sidebar_failed", lang);
    let elapsed_label_str = tr("sidebar_elapsed_time", lang);

    // Dynamic padding calculation for dual-column alignment
    let col1_left = format!("{}{}", attempt_label, total_str);
    let col1_right = format!("{}{}", yield_label, yield_str);
    let pad1 = (inner_area.width as usize).saturating_sub(col1_left.len() + col1_right.len());

    let col2_left = format!("{}{}", passed_label, passed_str);
    let col2_right = format!("{}{}", failed_label, failed_str);
    let pad2 = (inner_area.width as usize).saturating_sub(col2_left.len() + col2_right.len());

    let mut stats_text = vec![
        Line::from(vec![
            Span::styled(
                attempt_label,
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                total_str,
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.text)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" ".repeat(pad1.max(2))),
            Span::styled(
                yield_label,
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                yield_str,
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
                passed_label,
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                passed_str,
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" ".repeat(pad2.max(2))),
            Span::styled(
                failed_label,
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                failed_str,
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.danger)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                elapsed_label_str,
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(elapsed_str, Style::default().fg(CATPPUCCIN_MOCHA.text)),
        ]),
    ];

    // Separator line
    let sep_char = "─";
    stats_text.push(Line::from(Span::styled(
        sep_char.repeat(inner_area.width as usize),
        Style::default().fg(CATPPUCCIN_MOCHA.border),
    )));

    // Render the outer block
    f.render_widget(stats_block, area);

    // Split inner_area vertically
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Stats text + separator line
            Constraint::Length(2), // Buttons rows
        ])
        .split(inner_area);

    // Render stats text
    let stats_widget = Paragraph::new(stats_text)
        .block(Block::default())
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));
    f.render_widget(stats_widget, vertical_chunks[0]);

    // Get the button rects from app
    let rects = app.get_flash_summary_button_rects();

    // Set up button labels
    let b0_label = if lang == "zh" {
        "开始烧录"
    } else {
        "Start Flash"
    };
    let b1_label = if app.flash_batch_mode {
        if lang == "zh" {
            "模式: 批量"
        } else {
            "Mode: Batch"
        }
    } else {
        if lang == "zh" {
            "模式: 单台"
        } else {
            "Mode: Single"
        }
    };
    let b2_label = if lang == "zh" {
        "自动感应"
    } else {
        "Auto Flash"
    };
    let b3_label = if lang == "zh" {
        "清空累计"
    } else {
        "Clear Stats"
    };

    // Button colors
    let b0_color = CATPPUCCIN_MOCHA.primary;
    let b1_color = CATPPUCCIN_MOCHA.accent;
    let b2_color = if app.auto_flash {
        CATPPUCCIN_MOCHA.success
    } else {
        CATPPUCCIN_MOCHA.text_muted
    };
    let b3_color = CATPPUCCIN_MOCHA.danger;

    // Helper closure to render buttons
    let render_btn =
        |f: &mut Frame, rect: Rect, label: &str, is_hovered: bool, color: ratatui::style::Color| {
            let text = if is_hovered {
                let pad_w = (rect.width as usize)
                    .saturating_sub(unicode_width::UnicodeWidthStr::width(label));
                let pad_left = pad_w / 2;
                let pad_right = pad_w - pad_left;
                let padded_label =
                    format!("{}{}{}", " ".repeat(pad_left), label, " ".repeat(pad_right));
                Line::from(vec![Span::styled(
                    padded_label,
                    Style::default()
                        .fg(mocha::BASE)
                        .bg(color)
                        .add_modifier(Modifier::BOLD),
                )])
            } else {
                let content_w = (rect.width as usize).saturating_sub(4);
                let label_w = unicode_width::UnicodeWidthStr::width(label);
                let pad_w = content_w.saturating_sub(label_w);
                let pad_left = pad_w / 2;
                let pad_right = pad_w - pad_left;
                let padded_label =
                    format!("{}{}{}", " ".repeat(pad_left), label, " ".repeat(pad_right));
                Line::from(vec![
                    Span::styled("[ ", Style::default().fg(CATPPUCCIN_MOCHA.border)),
                    Span::styled(
                        padded_label,
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" ]", Style::default().fg(CATPPUCCIN_MOCHA.border)),
                ])
            };
            let btn_widget = Paragraph::new(text);
            f.render_widget(btn_widget, rect);
        };

    // Render the 4 buttons
    if rects.len() >= 4 {
        render_btn(
            f,
            rects[0],
            b0_label,
            app.hover_flash_action == Some(0),
            b0_color,
        );
        render_btn(
            f,
            rects[1],
            b1_label,
            app.hover_flash_action == Some(1),
            b1_color,
        );
        render_btn(
            f,
            rects[2],
            b2_label,
            app.hover_flash_action == Some(2),
            b2_color,
        );
        render_btn(
            f,
            rects[3],
            b3_label,
            app.hover_flash_action == Some(3),
            b3_color,
        );
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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
