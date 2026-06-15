use crate::app::App;
use crate::ui::theme::{CATPPUCCIN_MOCHA, mocha};
use crate::ui::tr;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Row, Table, Wrap},
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let lang = &app.tool_config.language;
    let config_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
        .title(Span::styled(
            tr("config_title", lang),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ));

    let block_inner = config_block.inner(area);
    f.render_widget(config_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(6)])
        .split(block_inner);

    // Configuration Fields
    let cfg = &app.config;
    let fields = vec![
        (tr("config_proj_name", lang), cfg.name.clone()),
        (tr("config_chip_type", lang), cfg.chip_type.clone()),
        (tr("config_baud_rate", lang), cfg.baud_rate.to_string()),
        (tr("config_flash_mode", lang), cfg.flash_mode.clone()),
        (tr("config_flash_freq", lang), cfg.flash_freq.clone()),
        (tr("config_flash_size", lang), cfg.flash_size.clone()),
        (tr("config_bootloader_offset", lang), cfg.bootloader_offset.clone()),
        (tr("config_bootloader_path", lang), cfg.bootloader_path.clone()),
        (tr("config_partitions_offset", lang), cfg.partitions_offset.clone()),
        (tr("config_partitions_path", lang), cfg.partitions_path.clone()),
        (tr("config_otadata_offset", lang), cfg.otadata_offset.clone()),
        (tr("config_otadata_path", lang), cfg.otadata_path.clone()),
        (tr("config_app_offset", lang), cfg.app_offset.clone()),
        (tr("config_app_path", lang), cfg.app_path.clone()),
    ];

    let mut rows = Vec::new();
    for (i, (label, val)) in fields.iter().enumerate() {
        let is_selected = app.selected_config_field == i;

        let label_span = Span::styled(
            *label,
            if is_selected {
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted)
            },
        );

        let val_span = if is_selected {
            if app.is_editing_config {
                Span::styled(
                    format!("{}█", app.edit_buffer), // Mock cursor
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.accent)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                let color = if app.admin_mode {
                    CATPPUCCIN_MOCHA.accent
                } else {
                    CATPPUCCIN_MOCHA.primary
                };
                Span::styled(val, Style::default().fg(color).add_modifier(Modifier::BOLD))
            }
        } else {
            Span::styled(val, Style::default().fg(CATPPUCCIN_MOCHA.text))
        };

        // Flat uniform background, surface0 highlight for selected
        let row_bg = if is_selected {
            mocha::SURFACE0
        } else {
            mocha::BASE
        };

        rows.push(
            Row::new(vec![table_cell(label_span), table_cell(val_span)])
                .style(Style::default().bg(row_bg)),
        );
    }

    let widths = [Constraint::Length(22), Constraint::Min(20)];
    let table = Table::new(rows, widths)
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().bg(mocha::BASE));

    f.render_widget(table, chunks[0]);

    // Inspector and Guides
    let (selected_label, selected_val) = &fields[app.selected_config_field];
    let display_val = if app.is_editing_config {
        app.edit_buffer.clone()
    } else {
        selected_val.clone()
    };

    let inspector_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
        .title(Span::styled(
            tr("config_inspector_title", lang),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.accent)
                .add_modifier(Modifier::BOLD),
        ));

    let mut inspector_lines = Vec::new();

    // Line 1: Parameter Name and Status
    let clean_label = selected_label.trim_end_matches(':');
    let status_span = if app.admin_mode {
        if app.is_editing_config {
            Span::styled(
                tr("config_status_editing", lang),
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(
                tr("config_status_unlocked", lang),
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.success)
                    .add_modifier(Modifier::BOLD),
            )
        }
    } else {
        Span::styled(
            tr("config_status_locked", lang),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.danger)
                .add_modifier(Modifier::BOLD),
        )
    };

    inspector_lines.push(Line::from(vec![
        Span::styled(
            tr("config_parameter", lang),
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ),
        Span::styled(
            clean_label,
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            tr("config_status", lang),
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ),
        status_span,
    ]));

    // Line 2: Spacer
    inspector_lines.push(Line::from(""));

    // Line 3: Value
    let val_color = if app.is_editing_config {
        CATPPUCCIN_MOCHA.accent
    } else if app.admin_mode {
        CATPPUCCIN_MOCHA.success
    } else {
        CATPPUCCIN_MOCHA.primary
    };

    inspector_lines.push(Line::from(vec![
        Span::styled(
            tr("config_value", lang),
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ),
        Span::styled(
            display_val,
            Style::default().fg(val_color).add_modifier(Modifier::BOLD),
        ),
    ]));

    // Line 4: Spacer
    inspector_lines.push(Line::from(""));

    // Line 5: Help Guide
    let guide_spans = if !app.admin_mode {
        vec![
            Span::styled(tr("config_guide", lang), Style::default().fg(CATPPUCCIN_MOCHA.primary)),
            Span::styled(tr("config_guide_locked", lang), Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
        ]
    } else if app.is_editing_config {
        vec![
            Span::styled(tr("config_guide", lang), Style::default().fg(CATPPUCCIN_MOCHA.accent)),
            Span::styled(tr("config_guide_editing", lang), Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
        ]
    } else {
        vec![
            Span::styled(tr("config_guide", lang), Style::default().fg(CATPPUCCIN_MOCHA.success)),
            Span::styled(tr("config_guide_unlocked", lang), Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
        ]
    };
    inspector_lines.push(Line::from(guide_spans));

    let admin_help = Paragraph::new(inspector_lines)
        .block(inspector_block)
        .wrap(Wrap { trim: true });
    f.render_widget(admin_help, chunks[1]);
}

fn table_cell<'a>(span: Span<'a>) -> Line<'a> {
    Line::from(span)
}
