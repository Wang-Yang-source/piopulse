use crate::app::{ActiveTab, App, Channel};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Row, Table, Tabs},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    // Main layout: Vertical split for Header, Main Body, Footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Main Area
            Constraint::Length(1), // Footer
        ])
        .split(f.size());

    app.layout_zones.header = chunks[0];

    draw_header(f, app, chunks[0]);
    draw_main_area(f, app, chunks[1]);
    draw_footer(f, app, chunks[2]);

    if app.is_entering_password {
        let area = center_rect(45, 11, f.size());
        app.layout_zones.password_modal = area;
        draw_password_modal(f, app, area);
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let style = Style::default().bg(Color::Rgb(15, 23, 42)).fg(Color::White); // Slate 900
    
    let header_block = Block::default()
        .style(style)
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::Rgb(51, 65, 85))); // Slate 700

    let title = Span::styled(
        " ☕ PIOPULSE ESP32 FLASHER v0.1.0 ",
        Style::default()
            .fg(Color::Rgb(251, 146, 60)) // Orange 400
            .add_modifier(Modifier::BOLD),
    );

    let mode_span = if app.admin_mode {
        Span::styled(" [ADMIN MODE] ", Style::default().fg(Color::Rgb(239, 68, 68)).add_modifier(Modifier::BOLD)) // Red 500
    } else {
        Span::styled(" [OPERATOR MODE] ", Style::default().fg(Color::Rgb(34, 197, 94)).add_modifier(Modifier::BOLD)) // Green 500
    };

    let header_text = Line::from(vec![
        title,
        Span::raw(" | "),
        mode_span,
    ]);

    let header = Paragraph::new(header_text)
        .block(header_block)
        .style(style);

    f.render_widget(header, area);
}

fn draw_footer(f: &mut Frame, _app: &App, area: Rect) {
    let footer_text = Span::styled(
        " F1/Tab: Toggle Admin | Space: Start Flash | c: Clear Stats | 1/2/3: Tabs | Esc: Quit",
        Style::default().fg(Color::Rgb(148, 163, 184)), // Slate 400
    );
    f.render_widget(Paragraph::new(footer_text), area);
}

fn draw_main_area(f: &mut Frame, app: &mut App, area: Rect) {
    // Horizontal split: Left Workspace (70%), Right Panel (30%)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    draw_workspace(f, app, chunks[0]);
    draw_sidebar(f, app, chunks[1]);
}

fn draw_workspace(f: &mut Frame, app: &mut App, area: Rect) {
    // Vertical split for Tabs Bar and Content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(area);

    // Render Tabs
    let tab_titles = vec![" [1] Channels ", " [2] Logs ", " [3] Configuration "];
    
    let active_index = match app.active_tab {
        ActiveTab::Channels => 0,
        ActiveTab::Logs => 1,
        ActiveTab::Configuration => 2,
    };

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::Rgb(51, 65, 85))))
        .select(active_index)
        .style(Style::default().fg(Color::Rgb(148, 163, 184)))
        .highlight_style(
            Style::default()
                .fg(Color::Rgb(251, 146, 60))
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, chunks[0]);
    app.layout_zones.tabs = chunks[0];

    // Render Tab Content
    match app.active_tab {
        ActiveTab::Channels => draw_channels_tab(f, app, chunks[1]),
        ActiveTab::Logs => draw_logs_tab(f, app, chunks[1]),
        ActiveTab::Configuration => {
            app.layout_zones.config_table = chunks[1];
            draw_config_tab(f, app, chunks[1]);
        }
    }
}

fn draw_channels_tab(f: &mut Frame, app: &mut App, area: Rect) {
    if app.channels.is_empty() {
        let empty_msg = Paragraph::new(
            "\n\n\n\nNo serial devices detected.\n\nEnsure ESP32 devices are plugged in and powered.",
        )
        .alignment(ratatui::layout::Alignment::Center)
        .style(Style::default().fg(Color::Rgb(148, 163, 184)));
        f.render_widget(empty_msg, area);
        return;
    }

    // Grid Layout for channels: 2 columns
    let num_channels = app.channels.len();
    let num_cols = 2;
    let num_rows = (num_channels + num_cols - 1) / num_cols;

    // Split vertically into rows
    let row_constraints = vec![Constraint::Ratio(1, num_rows as u32); num_rows];
    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(area);

    for r in 0..num_rows {
        // Split horizontally into 2 columns for this row
        let col_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(row_chunks[r]);

        for c in 0..num_cols {
            let channel_idx = r * num_cols + c;
            if channel_idx < num_channels {
                draw_channel_card(f, app, &app.channels[channel_idx], col_chunks[c]);
            }
        }
    }
}

fn draw_channel_card(f: &mut Frame, _app: &App, channel: &Channel, area: Rect) {
    // Select border color based on status
    let (border_color, status_style) = if channel.finished {
        if channel.success {
            (Color::Rgb(34, 197, 94), Style::default().fg(Color::Rgb(34, 197, 94)).add_modifier(Modifier::BOLD)) // Green
        } else {
            (Color::Rgb(239, 68, 68), Style::default().fg(Color::Rgb(239, 68, 68)).add_modifier(Modifier::BOLD)) // Red
        }
    } else if channel.status == "Idle" {
        (Color::Rgb(71, 85, 105), Style::default().fg(Color::Rgb(148, 163, 184))) // Slate 600
    } else {
        (Color::Rgb(234, 179, 8), Style::default().fg(Color::Rgb(234, 179, 8)).add_modifier(Modifier::BOLD)) // Yellow
    };

    let title = format!(" Channel: {} ", channel.port);
    let card_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let card_area = card_block.inner(area);
    f.render_widget(card_block, area);

    // Inner details layout
    let detail_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // USB Device Details
            Constraint::Length(1), // Chip / MAC
            Constraint::Length(1), // Status / Speed
            Constraint::Length(2), // Progress Bar & Text
            Constraint::Min(0),    // Error log if failed
        ])
        .split(card_area);

    // Line 0: USB Device Details
    let manufacturer = channel.usb_manufacturer.as_deref().unwrap_or("");
    let product = channel.usb_product.as_deref().unwrap_or("USB Serial Device");
    let dev_name = if manufacturer.is_empty() {
        product.to_string()
    } else {
        format!("{} {}", manufacturer, product)
    };
    let dev_id_str = match (channel.vid, channel.pid) {
        (Some(v), Some(p)) => format!(" ({:04X}:{:04X})", v, p),
        _ => "".to_string(),
    };
    let line0 = Line::from(vec![
        Span::styled("Device: ", Style::default().fg(Color::Rgb(148, 163, 184))),
        Span::styled(format!("{}{}", dev_name, dev_id_str), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]);
    f.render_widget(Paragraph::new(line0), detail_chunks[0]);

    // Line 1: Chip & MAC
    let chip_str = channel.chip.as_deref().unwrap_or("Detecting...");
    let mac_str = channel.mac.as_deref().unwrap_or("XX:XX:XX:XX:XX:XX");
    let line1 = Line::from(vec![
        Span::styled("Chip:   ", Style::default().fg(Color::Rgb(148, 163, 184))),
        Span::styled(chip_str, Style::default().fg(Color::White)),
        Span::raw("   "),
        Span::styled("MAC: ", Style::default().fg(Color::Rgb(148, 163, 184))),
        Span::styled(mac_str, Style::default().fg(Color::White)),
    ]);
    f.render_widget(Paragraph::new(line1), detail_chunks[1]);

    // Line 2: Status & Speed
    let line2 = Line::from(vec![
        Span::styled("Status: ", Style::default().fg(Color::Rgb(148, 163, 184))),
        Span::styled(&channel.status, status_style),
        Span::raw("   "),
        Span::styled("Speed: ", Style::default().fg(Color::Rgb(148, 163, 184))),
        Span::styled(&channel.speed, Style::default().fg(Color::White)),
    ]);
    f.render_widget(Paragraph::new(line2), detail_chunks[2]);

    // Line 3 & 4: Progress Bar
    let gauge = Gauge::default()
        .block(Block::default())
        .gauge_style(Style::default().fg(border_color).bg(Color::Rgb(30, 41, 59)))
        .percent(channel.progress as u16);
    f.render_widget(gauge, detail_chunks[2]);

    // Line 5: Error Details
    if let Some(err) = &channel.error {
        let err_block = Paragraph::new(format!("Error: {}", err))
            .style(Style::default().fg(Color::Rgb(239, 68, 68)))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(err_block, detail_chunks[3]);
    }
}

fn draw_logs_tab(f: &mut Frame, app: &App, area: Rect) {
    let logs_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(51, 65, 85)))
        .title(" System Event Log ");

    let list_items: Vec<ListItem> = app
        .logs
        .iter()
        .rev() // Show latest logs at the top
        .skip(app.logs_scroll_offset)
        .map(|log| {
            // Give specific coloring to certain messages
            let style = if log.contains("FAILED") || log.contains("Error") {
                Style::default().fg(Color::Rgb(239, 68, 68))
            } else if log.contains("PASSED") || log.contains("Success") {
                Style::default().fg(Color::Rgb(34, 197, 94))
            } else if log.contains("Start Batch") {
                Style::default().fg(Color::Rgb(59, 130, 246)).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(226, 232, 240))
            };
            ListItem::new(log.as_str()).style(style)
        })
        .collect();

    let list = List::new(list_items)
        .block(logs_block)
        .style(Style::default().bg(Color::Rgb(9, 15, 29)));
        
    f.render_widget(list, area);
}

fn draw_config_tab(f: &mut Frame, app: &App, area: Rect) {
    let config_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(51, 65, 85)))
        .title(" Firmware Flashing Configuration ");

    let block_inner = config_block.inner(area);
    f.render_widget(config_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
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
        let is_selected = app.admin_mode && app.selected_config_field == i;
        
        let label_span = Span::styled(
            *label,
            Style::default().fg(Color::Rgb(148, 163, 184)),
        );
        
        let val_span = if is_selected {
            if app.is_editing_config {
                Span::styled(
                    format!("{} █", app.edit_buffer), // Mock cursor
                    Style::default()
                        .fg(Color::Rgb(251, 146, 60))
                        .add_modifier(Modifier::REVERSED),
                )
            } else {
                Span::styled(
                    val,
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Rgb(251, 146, 60))
                        .add_modifier(Modifier::BOLD),
                )
            }
        } else {
            Span::styled(val, Style::default().fg(Color::White))
        };

        rows.push(Row::new(vec![
            table_cell(label_span),
            table_cell(val_span),
        ]));
    }

    let widths = [Constraint::Percentage(30), Constraint::Percentage(70)];
    let table = Table::new(rows, widths)
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().bg(Color::Rgb(9, 15, 29)));

    f.render_widget(table, chunks[0]);

    // Admin Info helper block
    let admin_help_text = if !app.admin_mode {
        Line::from(vec![
            Span::styled("🛡️ Admin Mode is Locked. ", Style::default().fg(Color::Rgb(239, 68, 68))),
            Span::styled("Unlock (F1/Tab) to edit config fields.", Style::default().fg(Color::Rgb(148, 163, 184))),
        ])
    } else if app.is_editing_config {
        Line::from(vec![
            Span::styled("✏️ EDITING MODE: ", Style::default().fg(Color::Rgb(251, 146, 60)).add_modifier(Modifier::BOLD)),
            Span::styled("Type value, press Enter to Save, Esc to Cancel.", Style::default().fg(Color::Rgb(226, 232, 240))),
        ])
    } else {
        Line::from(vec![
            Span::styled("⚙️ ADMIN CONTROL: ", Style::default().fg(Color::Rgb(239, 68, 68)).add_modifier(Modifier::BOLD)),
            Span::styled("Use ↑↓ to navigate, Enter to Edit, Tab to lock.", Style::default().fg(Color::Rgb(226, 232, 240))),
        ])
    };

    let admin_help = Paragraph::new(admin_help_text)
        .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(Color::Rgb(51, 65, 85))));
    f.render_widget(admin_help, chunks[1]);
}

// Helper struct for table cells
fn table_cell<'a>(span: Span<'a>) -> Line<'a> {
    Line::from(span)
}

fn draw_sidebar(f: &mut Frame, app: &mut App, area: Rect) {
    // Split vertical: Stats (50%), Instructions (50%)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    app.layout_zones.help_sidebar = chunks[1];

    draw_stats(f, app, chunks[0]);
    draw_instructions(f, app, chunks[1]);
}

fn draw_stats(f: &mut Frame, app: &App, area: Rect) {
    let stats_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(51, 65, 85)))
        .title(" Production Statistics ");

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
            Span::styled("Total Attempted: ", Style::default().fg(Color::Rgb(148, 163, 184))),
            Span::styled(app.stats.total_attempted.to_string(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Passed (OK):     ", Style::default().fg(Color::Rgb(148, 163, 184))),
            Span::styled(app.stats.total_passed.to_string(), Style::default().fg(Color::Rgb(34, 197, 94)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Failed (FAIL):   ", Style::default().fg(Color::Rgb(148, 163, 184))),
            Span::styled(app.stats.total_failed.to_string(), Style::default().fg(Color::Rgb(239, 68, 68)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Yield Rate:      ", Style::default().fg(Color::Rgb(148, 163, 184))),
            Span::styled(format!("{:.1}%", yield_rate), Style::default().fg(if yield_rate >= 95.0 || total == 0 { Color::Rgb(34, 197, 94) } else { Color::Rgb(239, 68, 68) }).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Elapsed Time:    ", Style::default().fg(Color::Rgb(148, 163, 184))),
            Span::styled(elapsed_str, Style::default().fg(Color::White)),
        ]),
    ];

    let inner_area = stats_block.inner(area);
    f.render_widget(stats_block, area);

    let stats_widget = Paragraph::new(stats_text)
        .block(Block::default())
        .style(Style::default().bg(Color::Rgb(9, 15, 29)));
        
    f.render_widget(stats_widget, inner_area);
}

fn draw_instructions(f: &mut Frame, _app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(51, 65, 85)))
        .title(" User Interface Help ");

    let help_text = vec![
        Line::from(vec![
            Span::styled("Space", Style::default().fg(Color::Rgb(251, 146, 60)).add_modifier(Modifier::BOLD)),
            Span::styled("  - Trigger Flashing Process", Style::default().fg(Color::Rgb(226, 232, 240))),
        ]),
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Rgb(251, 146, 60)).add_modifier(Modifier::BOLD)),
            Span::styled("    - Toggle Admin Auth Mode", Style::default().fg(Color::Rgb(226, 232, 240))),
        ]),
        Line::from(vec![
            Span::styled("c", Style::default().fg(Color::Rgb(251, 146, 60)).add_modifier(Modifier::BOLD)),
            Span::styled("      - Clear Statistics Counters", Style::default().fg(Color::Rgb(226, 232, 240))),
        ]),
        Line::from(vec![
            Span::styled("1/2/3", Style::default().fg(Color::Rgb(251, 146, 60)).add_modifier(Modifier::BOLD)),
            Span::styled("  - Switch Active Workspace Tab", Style::default().fg(Color::Rgb(226, 232, 240))),
        ]),
        Line::from(vec![
            Span::styled("Esc", Style::default().fg(Color::Rgb(251, 146, 60)).add_modifier(Modifier::BOLD)),
            Span::styled("    - Exit Application Safely", Style::default().fg(Color::Rgb(226, 232, 240))),
        ]),
    ];

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let help_widget = Paragraph::new(help_text)
        .block(Block::default())
        .style(Style::default().bg(Color::Rgb(9, 15, 29)));
        
    f.render_widget(help_widget, inner_area);
}

fn draw_password_modal(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Admin Authorization Required ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(239, 68, 68)));

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

    let msg = Paragraph::new("Enter administrator password: (Default: admin)")
        .style(Style::default().fg(Color::Rgb(226, 232, 240)));
    f.render_widget(msg, text_chunks[0]);

    // Mask password characters
    let masked_input: String = std::iter::repeat('*').take(app.password_input.len()).collect();
    let input_widget = Paragraph::new(masked_input)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Rgb(251, 146, 60))))
        .style(Style::default().fg(Color::White));
    f.render_widget(input_widget, text_chunks[1]);

    let cancel_msg = Paragraph::new("Press Enter to submit | Click outside / Esc to cancel")
        .style(Style::default().fg(Color::Rgb(148, 163, 184)));
    f.render_widget(cancel_msg, text_chunks[2]);

    if app.password_incorrect {
        let err_msg = Paragraph::new("Incorrect password. Press Enter to retry.")
            .style(Style::default().fg(Color::Rgb(239, 68, 68)));
        f.render_widget(err_msg, text_chunks[3]);
    }
}

// Center rect helper
fn center_rect(percent_x: u16, height_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height.saturating_sub(height_y)) / 2),
            Constraint::Length(height_y),
            Constraint::Min(0),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
