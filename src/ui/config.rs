use crate::app::App;
use crate::ui::theme::{CATPPUCCIN_MOCHA, mocha};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Row, Table, Wrap},
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let config_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
        .title(Span::styled(
            " Configuration ",
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
        ("Project Name:", cfg.name.clone()),
        ("Chip Type:", cfg.chip_type.clone()),
        ("Baud Rate:", cfg.baud_rate.to_string()),
        ("Flash Mode:", cfg.flash_mode.clone()),
        ("Flash Freq:", cfg.flash_freq.clone()),
        ("Flash Size:", cfg.flash_size.clone()),
        ("Bootloader Offset:", cfg.bootloader_offset.clone()),
        ("Bootloader Path:", cfg.bootloader_path.clone()),
        ("Partitions Offset:", cfg.partitions_offset.clone()),
        ("Partitions Path:", cfg.partitions_path.clone()),
        ("OTA Data Offset:", cfg.otadata_offset.clone()),
        ("OTA Data Path:", cfg.otadata_path.clone()),
        ("App Offset:", cfg.app_offset.clone()),
        ("App Path:", cfg.app_path.clone()),
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
            " Inspector ",
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
                "EDITING",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(
                "UNLOCKED",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.success)
                    .add_modifier(Modifier::BOLD),
            )
        }
    } else {
        Span::styled(
            "LOCKED (READ-ONLY)",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.danger)
                .add_modifier(Modifier::BOLD),
        )
    };

    inspector_lines.push(Line::from(vec![
        Span::styled(
            "  Parameter: ",
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ),
        Span::styled(
            clean_label,
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "  ·  Status: ",
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
            "  Value: ",
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
            Span::styled("  Guide: ", Style::default().fg(CATPPUCCIN_MOCHA.primary)),
            Span::styled("Press ", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
            Span::styled(
                "F1",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " to unlock. Use ",
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                "Up/Down Arrows",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " or click to select fields.",
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
        ]
    } else if app.is_editing_config {
        vec![
            Span::styled("  Guide: ", Style::default().fg(CATPPUCCIN_MOCHA.accent)),
            Span::styled(
                "Type new value. Press ",
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " to save, ",
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.danger)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " to cancel.",
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
        ]
    } else {
        vec![
            Span::styled("  Guide: ", Style::default().fg(CATPPUCCIN_MOCHA.success)),
            Span::styled("Press ", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " or click to edit. Press ",
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                "F1",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " to lock.",
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
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
