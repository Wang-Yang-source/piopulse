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

/// Returns the list of (field_index, label_key) pairs visible for the current config.
/// ESP-specific fields are hidden when the project is not ESP-based.
fn visible_fields(app: &App) -> Vec<(usize, &'static str)> {
    let cfg = &app.config;
    let is_esp = cfg.chip_type.to_lowercase().contains("esp")
        || cfg.framework.to_lowercase().contains("espidf")
        || cfg.framework.to_lowercase().contains("arduino");
    // If framework is empty and chip is not explicitly non-ESP, check paths
    let is_esp = is_esp || (cfg.framework.is_empty() && !cfg.bootloader_path.is_empty());

    let mut fields: Vec<(usize, &'static str)> = Vec::new();

    // Universal fields
    fields.push((0, "config_proj_name"));
    fields.push((1, "config_chip_type"));
    fields.push((30, "config_framework"));
    fields.push((31, "config_upload_protocol"));
    fields.push((32, "config_debug_tool"));
    fields.push((2, "config_baud_rate"));

    if is_esp {
        // ESP-specific fields
        fields.push((3, "config_flash_mode"));
        fields.push((4, "config_flash_freq"));
        fields.push((5, "config_flash_size"));
        fields.push((6, "config_bootloader_offset"));
        fields.push((7, "config_bootloader_path"));
        fields.push((8, "config_partitions_offset"));
        fields.push((9, "config_partitions_path"));
        fields.push((10, "config_otadata_offset"));
        fields.push((11, "config_otadata_path"));
    }

    fields.push((12, "config_app_offset"));
    fields.push((13, "config_app_path"));

    if is_esp {
        fields.push((14, "config_nvs_offset"));
    }

    fields.push((15, "config_verify_method"));
    fields.push((16, "config_blank_check"));
    fields.push((17, "config_erase_mode"));
    fields.push((18, "config_incremental_programming"));

    if is_esp {
        fields.push((19, "config_secure_boot"));
        fields.push((20, "config_flash_encryption"));
        fields.push((21, "config_lock_after_flash"));
    }

    fields.push((22, "config_operator_role"));
    fields.push((23, "config_firmware_version"));
    fields.push((24, "config_sn_prefix"));
    fields.push((25, "config_lot_code"));
    fields.push((26, "config_mes_endpoint"));
    fields.push((27, "config_label_template"));
    fields.push((28, "config_qa_test_script"));
    fields.push((29, "config_do_not_chg_bin"));

    fields
}

pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    let lang = app.tool_config.language.clone();
    let config_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
        .title(Span::styled(
            tr("config_title", &lang),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ));

    let block_inner = config_block.inner(area);
    f.render_widget(config_block, area);

    let show_inspector = block_inner.height >= 14;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(if show_inspector { 6 } else { 0 }),
        ])
        .split(block_inner);

    // Build visible fields list with (field_index, label, value)
    let cfg = &app.config;
    let vis_fields = visible_fields(app);
    let fields: Vec<(usize, &str, String)> = vis_fields
        .iter()
        .map(|(idx, label_key)| (*idx, tr(label_key, &lang), cfg.get_field(*idx)))
        .collect();

    // Compute visible rows and adjust scroll
    let table_height = chunks[0].height as usize;
    app.ensure_config_field_visible(table_height);
    let scroll_offset = app.config_scroll_offset;
    let total_fields = fields.len();

    let mut rows = Vec::new();
    for (row_idx, (_field_idx, label, val)) in fields.iter().enumerate() {
        if row_idx < scroll_offset {
            continue;
        }
        if row_idx >= scroll_offset + table_height {
            break;
        }
        let is_selected = app.selected_config_field == row_idx;

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
                Span::styled(
                    val.as_str(),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                )
            }
        } else {
            Span::styled(val.as_str(), Style::default().fg(CATPPUCCIN_MOCHA.text))
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

    // Scroll indicator in block title
    let scroll_info = if total_fields > table_height {
        format!(" [{}/{}] ", scroll_offset + 1, total_fields)
    } else {
        String::new()
    };
    let widths = [Constraint::Length(22), Constraint::Min(20)];
    let table = Table::new(rows, widths)
        .block(Block::default().borders(Borders::NONE).title(Span::styled(
            scroll_info,
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        )))
        .style(Style::default().bg(mocha::BASE));

    f.render_widget(table, chunks[0]);

    // Inspector and Guides
    if show_inspector {
        let selected_row = app
            .selected_config_field
            .min(fields.len().saturating_sub(1));
        let (_field_idx, selected_label, selected_val) = &fields[selected_row];
        let display_val = if app.is_editing_config {
            app.edit_buffer.clone()
        } else {
            selected_val.clone()
        };

        let inspector_block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
            .title(Span::styled(
                tr("config_inspector_title", &lang),
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
                    tr("config_status_editing", &lang),
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.accent)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    tr("config_status_unlocked", &lang),
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.success)
                        .add_modifier(Modifier::BOLD),
                )
            }
        } else {
            Span::styled(
                tr("config_status_locked", &lang),
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.text_muted)
                    .add_modifier(Modifier::BOLD),
            )
        };

        inspector_lines.push(Line::from(vec![
            Span::styled(
                format!("{} ", clean_label),
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.text)
                    .add_modifier(Modifier::BOLD),
            ),
            status_span,
        ]));

        // Line 2: Current Value
        inspector_lines.push(Line::from(vec![
            Span::styled(
                if lang == "zh" {
                    "  当前值: "
                } else {
                    "  Value:  "
                },
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(display_val, Style::default().fg(CATPPUCCIN_MOCHA.primary)),
        ]));

        // Line 3: Description / Metadata
        let desc_key = format!("desc_{}", selected_label.trim_end_matches(':'));
        let desc = tr(&desc_key, &lang);
        inspector_lines.push(Line::from(vec![
            Span::styled(
                if lang == "zh" {
                    "  说明:   "
                } else {
                    "  Desc:   "
                },
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(desc, Style::default().fg(CATPPUCCIN_MOCHA.text)),
        ]));

        // Line 4: Spacer
        inspector_lines.push(Line::from(""));

        // Line 5: Help Guide
        let guide_spans = if !app.admin_mode {
            vec![
                Span::styled(
                    tr("config_guide", &lang),
                    Style::default().fg(CATPPUCCIN_MOCHA.primary),
                ),
                Span::styled(
                    tr("config_guide_locked", &lang),
                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                ),
            ]
        } else if app.is_editing_config {
            vec![
                Span::styled(
                    tr("config_guide", &lang),
                    Style::default().fg(CATPPUCCIN_MOCHA.accent),
                ),
                Span::styled(
                    tr("config_guide_editing", &lang),
                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                ),
            ]
        } else {
            vec![
                Span::styled(
                    tr("config_guide", &lang),
                    Style::default().fg(CATPPUCCIN_MOCHA.success),
                ),
                Span::styled(
                    tr("config_guide_unlocked", &lang),
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
}

fn table_cell<'a>(span: Span<'a>) -> Line<'a> {
    Line::from(span)
}
