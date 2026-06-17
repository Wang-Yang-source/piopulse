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
    let lang = &app.tool_config.language;
    let block = Block::default()
        .title(Span::styled(
            crate::ui::tr("auth_title", lang),
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

    let msg = Paragraph::new(crate::ui::tr("auth_msg", lang))
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

    let cancel_msg = Paragraph::new(crate::ui::tr("auth_cancel", lang)).style(
        Style::default()
            .fg(CATPPUCCIN_MOCHA.text_muted)
            .bg(mocha::MANTLE),
    );
    f.render_widget(cancel_msg, text_chunks[2]);

    if app.password_incorrect {
        let err_msg = Paragraph::new(crate::ui::tr("auth_error", lang)).style(
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

    let lang = &app.tool_config.language;

    // 1. Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled("[!] ", Style::default()),
        Span::styled(
            crate::ui::tr("exit_title", lang),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.accent)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center)
    .style(Style::default().bg(mocha::MANTLE));
    f.render_widget(title, chunks[0]);

    // 2. Text question
    let question = Paragraph::new(crate::ui::tr("exit_question", lang))
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
    let settings_text = Paragraph::new(crate::ui::tr("exit_settings", lang))
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
    let quit_text = Paragraph::new(crate::ui::tr("exit_quit", lang))
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
    let hint = Paragraph::new(crate::ui::tr("exit_hint", lang))
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text_muted)
                .bg(mocha::MANTLE),
        );
    f.render_widget(hint, chunks[3]);
}

pub fn draw_tool_settings(f: &mut Frame, app: &App, area: Rect) {
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

    let lang = &app.tool_config.language;

    // 1. Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled("[⚙] ", Style::default()),
        Span::styled(
            crate::ui::tr("tool_settings_title", lang),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.accent)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center)
    .style(Style::default().bg(mocha::MANTLE));
    f.render_widget(title, chunks[0]);

    // 2. Text question
    let question = Paragraph::new(crate::ui::tr("tool_settings_question", lang))
        .alignment(Alignment::Center)
        .style(Style::default().fg(CATPPUCCIN_MOCHA.text).bg(mocha::MANTLE));
    f.render_widget(question, chunks[1]);

    // 3. Option Cards
    let card_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    // Card 0: English
    let is_en_selected = app.tool_settings_selected == 0;
    let en_border_style = if is_en_selected {
        Style::default().fg(CATPPUCCIN_MOCHA.border_focus)
    } else {
        Style::default().fg(CATPPUCCIN_MOCHA.border)
    };
    let en_block = Block::default()
        .borders(Borders::ALL)
        .border_type(if is_en_selected {
            BorderType::Double
        } else {
            BorderType::Rounded
        })
        .border_style(en_border_style)
        .style(if is_en_selected {
            Style::default().bg(CATPPUCCIN_MOCHA.selection_bg)
        } else {
            Style::default().bg(mocha::MANTLE)
        });
    let en_text = Paragraph::new(crate::ui::tr("tool_settings_en", lang))
        .alignment(Alignment::Center)
        .block(en_block)
        .style(if is_en_selected {
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted)
        });
    f.render_widget(en_text, card_chunks[0]);

    // Card 1: Chinese
    let is_zh_selected = app.tool_settings_selected == 1;
    let zh_border_style = if is_zh_selected {
        Style::default().fg(CATPPUCCIN_MOCHA.border_focus)
    } else {
        Style::default().fg(CATPPUCCIN_MOCHA.border)
    };
    let zh_block = Block::default()
        .borders(Borders::ALL)
        .border_type(if is_zh_selected {
            BorderType::Double
        } else {
            BorderType::Rounded
        })
        .border_style(zh_border_style)
        .style(if is_zh_selected {
            Style::default().bg(CATPPUCCIN_MOCHA.selection_bg)
        } else {
            Style::default().bg(mocha::MANTLE)
        });
    let zh_text = Paragraph::new(crate::ui::tr("tool_settings_zh", lang))
        .alignment(Alignment::Center)
        .block(zh_block)
        .style(if is_zh_selected {
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted)
        });
    f.render_widget(zh_text, card_chunks[1]);

    // 4. Hint
    let hint = Paragraph::new(crate::ui::tr("tool_settings_hint", lang))
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text_muted)
                .bg(mocha::MANTLE),
        );
    f.render_widget(hint, chunks[3]);
}

pub fn draw_port_menu(f: &mut Frame, app: &App, area: Rect) {
    let lang = &app.tool_config.language;

    let block = Block::default()
        .title(Span::styled(
            format!(" {} ", crate::ui::tr("port_menu_title", lang)),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.accent)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border_focus))
        .style(Style::default().bg(mocha::MANTLE));

    let inner_area = block.inner(area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    // Layout inside modal: List of ports (Min(0)) and Hint (1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // List of ports
            Constraint::Length(1), // Hint
        ])
        .split(inner_area);

    let mut items = Vec::new();
    for channel in &app.channels {
        let display_name = if let Some(ref prod) = channel.usb_product {
            format!("{} ({})", channel.port, prod)
        } else {
            channel.port.clone()
        };
        items.push(display_name);
    }

    let list_items: Vec<ratatui::widgets::ListItem> = items
        .iter()
        .enumerate()
        .map(|(idx, name)| {
            let is_selected = idx == app.port_menu_selected;
            let style = if is_selected {
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .bg(CATPPUCCIN_MOCHA.selection_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(CATPPUCCIN_MOCHA.text)
            };

            let prefix = if is_selected { "● " } else { "  " };
            ratatui::widgets::ListItem::new(format!("{}{}", prefix, name)).style(style)
        })
        .collect();

    let list = ratatui::widgets::List::new(list_items).style(Style::default().bg(mocha::MANTLE));
    f.render_widget(list, chunks[0]);

    let hint = Paragraph::new(crate::ui::tr("port_menu_hint", lang))
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text_muted)
                .bg(mocha::MANTLE),
        );
    f.render_widget(hint, chunks[1]);
}

pub fn draw_custom_baud(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border_focus))
        .style(Style::default().bg(mocha::MANTLE));

    let inner_area = block.inner(area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Length(3), // Input box for custom rate
            Constraint::Length(3), // Presets hint
            Constraint::Min(0),    // Help hint
        ])
        .split(inner_area);

    let lang = &app.tool_config.language;

    // 1. Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled("[⚡] ", Style::default()),
        Span::styled(
            if lang == "zh" { "设置波特率" } else { "Set Baud Rate" },
            Style::default()
                .fg(CATPPUCCIN_MOCHA.accent)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center)
    .style(Style::default().bg(mocha::MANTLE));
    f.render_widget(title, chunks[0]);

    // 2. Input box
    let custom_baud_box = Paragraph::new(app.custom_baud_input.as_str())
        .block(
            Block::default()
                .title(Span::styled(
                    if lang == "zh" { " 自定义波特率 (输入后按回车) " } else { " Custom Baud Rate (Type & Enter) " },
                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(CATPPUCCIN_MOCHA.success)),
        )
        .style(Style::default().fg(CATPPUCCIN_MOCHA.text).bg(mocha::MANTLE));
    f.render_widget(custom_baud_box, chunks[1]);

    // 3. Presets hint
    let presets_msg = if lang == "zh" {
        "常用预设: 9600, 115200, 921600, 1152000, 1500000"
    } else {
        "Presets: 9600, 115200, 921600, 1152000, 1500000"
    };
    let presets = Paragraph::new(presets_msg)
        .alignment(Alignment::Center)
        .style(Style::default().fg(CATPPUCCIN_MOCHA.text_muted).bg(mocha::MANTLE));
    f.render_widget(presets, chunks[2]);

    // 4. Auto-detect hint
    let auto_detect_hint = if lang == "zh" {
        "按 [Tab] 开始自动识别波特率 | Esc: 取消"
    } else {
        "Press [Tab] to Auto-Detect Baud Rate | Esc: Cancel"
    };
    let hint = Paragraph::new(auto_detect_hint)
        .alignment(Alignment::Center)
        .style(Style::default().fg(CATPPUCCIN_MOCHA.accent).add_modifier(Modifier::ITALIC).bg(mocha::MANTLE));
    f.render_widget(hint, chunks[3]);
}

pub fn draw_auto_reply(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border_focus))
        .style(Style::default().bg(mocha::MANTLE));

    let inner_area = block.inner(area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Length(3), // Match pattern input
            Constraint::Length(3), // Response input
            Constraint::Min(0),    // Help hint
        ])
        .split(inner_area);

    let lang = &app.tool_config.language;

    // 1. Title
    let title_text = if lang == "zh" { "自动回复配置" } else { "Auto-Reply Configuration" };
    let title = Paragraph::new(Line::from(vec![
        Span::styled("[🔄] ", Style::default()),
        Span::styled(
            title_text,
            Style::default()
                .fg(CATPPUCCIN_MOCHA.accent)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center)
    .style(Style::default().bg(mocha::MANTLE));
    f.render_widget(title, chunks[0]);

    // 2. Match Pattern input box
    let pattern_focused = app.auto_reply_focused_field == 0;
    let pattern_border = if pattern_focused { CATPPUCCIN_MOCHA.border_focus } else { CATPPUCCIN_MOCHA.border };
    let pattern_title = if lang == "zh" { " 匹配模式 (支持 Regex / 文本) " } else { " Match Pattern (Regex/Text) " };
    let pattern_box = Paragraph::new(app.auto_reply_pattern_input.as_str())
        .block(
            Block::default()
                .title(Span::styled(pattern_title, Style::default().fg(pattern_border)))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(pattern_border)),
        )
        .style(Style::default().fg(CATPPUCCIN_MOCHA.text).bg(mocha::MANTLE));
    f.render_widget(pattern_box, chunks[1]);

    // 3. Response input box
    let response_focused = app.auto_reply_focused_field == 1;
    let response_border = if response_focused { CATPPUCCIN_MOCHA.border_focus } else { CATPPUCCIN_MOCHA.border };
    let response_title = if lang == "zh" { " 回复数据 (支持 \\r, \\n 转义) " } else { " Response Data (Supports \\r, \\n escapes) " };
    let response_box = Paragraph::new(app.auto_reply_response_input.as_str())
        .block(
            Block::default()
                .title(Span::styled(response_title, Style::default().fg(response_border)))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(response_border)),
        )
        .style(Style::default().fg(CATPPUCCIN_MOCHA.text).bg(mocha::MANTLE));
    f.render_widget(response_box, chunks[2]);

    // 4. Help hint
    let hint_msg = if lang == "zh" {
        "Tab: 切换输入框 | Enter: 保存并启用 | Esc: 取消"
    } else {
        "Tab: Switch input | Enter: Save & Enable | Esc: Cancel"
    };
    let hint = Paragraph::new(hint_msg)
        .alignment(Alignment::Center)
        .style(Style::default().fg(CATPPUCCIN_MOCHA.text_muted).bg(mocha::MANTLE));
    f.render_widget(hint, chunks[3]);
}

pub fn draw_manifest_edit(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border_focus))
        .style(Style::default().bg(mocha::MANTLE));

    let inner_area = block.inner(area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Length(3), // Input box
            Constraint::Min(0),    // Help hint
        ])
        .split(inner_area);

    let lang = &app.tool_config.language;

    // 1. Title
    let title_text = if app.manifest_edit_is_offset {
        if lang == "zh" {
            format!("修改偏移地址 [{}]", app.manifest_edit_image_label)
        } else {
            format!("Edit Offset [{}]", app.manifest_edit_image_label)
        }
    } else {
        if lang == "zh" {
            format!("选择 bin 文件 [{}]", app.manifest_edit_image_label)
        } else {
            format!("Select Bin File [{}]", app.manifest_edit_image_label)
        }
    };

    let title = Paragraph::new(Line::from(vec![
        Span::styled("[📝] ", Style::default()),
        Span::styled(
            title_text,
            Style::default()
                .fg(CATPPUCCIN_MOCHA.accent)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center)
    .style(Style::default().bg(mocha::MANTLE));
    f.render_widget(title, chunks[0]);

    // 2. Input box
    let input_title = if app.manifest_edit_is_offset {
        if lang == "zh" { " 偏移地址 (例如: 0x10000) " } else { " Offset Address (e.g. 0x10000) " }
    } else {
        if lang == "zh" { " bin 文件路径 (输入/粘贴后按回车) " } else { " Bin File Path (Type/Paste & Enter) " }
    };

    let input_widget = Paragraph::new(app.manifest_edit_input.as_str())
        .block(
            Block::default()
                .title(Span::styled(input_title, Style::default().fg(CATPPUCCIN_MOCHA.border_focus)))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(CATPPUCCIN_MOCHA.success)),
        )
        .style(Style::default().fg(CATPPUCCIN_MOCHA.text).bg(mocha::MANTLE));
    f.render_widget(input_widget, chunks[1]);

    // 3. Help hint
    let hint_msg = if lang == "zh" {
        "Enter: 确认保存 | Esc: 取消"
    } else {
        "Enter: Confirm & Save | Esc: Cancel"
    };
    let hint = Paragraph::new(hint_msg)
        .alignment(Alignment::Center)
        .style(Style::default().fg(CATPPUCCIN_MOCHA.text_muted).bg(mocha::MANTLE));
    f.render_widget(hint, chunks[2]);
}

pub fn draw_file_picker(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border_focus))
        .style(Style::default().bg(mocha::MANTLE));

    let inner_area = block.inner(area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Current Directory Title
            Constraint::Length(3), // Filter Input
            Constraint::Min(5),    // List of Items
            Constraint::Length(1), // Footer hint
        ])
        .split(inner_area);

    let lang = &app.tool_config.language;

    // 1. Current Directory Path
    let path_str = app.file_picker_current_dir.to_string_lossy().to_string();
    let title_line = Line::from(vec![
        Span::styled("📂 ", Style::default()),
        Span::styled(
            if lang == "zh" { "当前目录: " } else { "Dir: " },
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted)
        ),
        Span::styled(
            path_str,
            Style::default().fg(CATPPUCCIN_MOCHA.accent).add_modifier(Modifier::BOLD)
        ),
    ]);
    f.render_widget(Paragraph::new(title_line).alignment(Alignment::Left), chunks[0]);

    // 2. Filter input box
    let filter_title = if lang == "zh" { " 输入过滤文件名 (支持实时筛选) " } else { " Filter filename (real-time) " };
    let filter_text = if app.file_picker_search_input.is_empty() {
        Span::styled(
            if lang == "zh" { "输入筛选..." } else { "Type to filter..." },
            Style::default().fg(CATPPUCCIN_MOCHA.text_disabled).add_modifier(Modifier::ITALIC)
        )
    } else {
        Span::styled(
            format!("{}█", app.file_picker_search_input),
            Style::default().fg(CATPPUCCIN_MOCHA.text).add_modifier(Modifier::BOLD)
        )
    };

    let filter_box = Paragraph::new(Line::from(filter_text))
        .block(
            Block::default()
                .title(Span::styled(filter_title, Style::default().fg(CATPPUCCIN_MOCHA.border_focus)))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border_focus)),
        )
        .style(Style::default().bg(mocha::MANTLE));
    f.render_widget(filter_box, chunks[1]);

    // 3. List of Items
    let headers = if lang == "zh" {
        vec!["名称", "大小"]
    } else {
        vec!["Name", "Size"]
    };

    let mut rows = Vec::new();
    for (idx, item) in app.file_picker_items.iter().enumerate() {
        let is_selected = idx == app.file_picker_selected_idx;
        
        let prefix = if item.is_dir { "📁 " } else if item.name.ends_with(".bin") { "🟢 " } else { "📄 " };
        
        let name_style = if is_selected {
            Style::default().fg(CATPPUCCIN_MOCHA.accent).add_modifier(Modifier::BOLD)
        } else if item.is_dir {
            Style::default().fg(CATPPUCCIN_MOCHA.primary).add_modifier(Modifier::BOLD)
        } else if item.name.ends_with(".bin") {
            Style::default().fg(CATPPUCCIN_MOCHA.success).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(CATPPUCCIN_MOCHA.text)
        };

        let size_style = if is_selected {
            Style::default().fg(CATPPUCCIN_MOCHA.accent)
        } else {
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted)
        };

        let row_bg = if is_selected {
            mocha::SURFACE0
        } else {
            mocha::BASE
        };

        rows.push(
            ratatui::widgets::Row::new(vec![
                ratatui::widgets::Cell::from(Span::styled(format!("{}{}", prefix, item.name), name_style)),
                ratatui::widgets::Cell::from(Span::styled(item.size_str.clone(), size_style)),
            ])
            .style(Style::default().bg(row_bg))
        );
    }

    let widths = [
        Constraint::Min(30),
        Constraint::Length(12),
    ];

    let table = ratatui::widgets::Table::new(rows, widths)
        .header(
            ratatui::widgets::Row::new(headers.into_iter().map(ratatui::widgets::Cell::from).collect::<Vec<_>>()).style(
                Style::default()
                    .fg(mocha::SUBTEXT1)
                    .bg(mocha::SURFACE0)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .block(Block::default().borders(Borders::NONE));

    app.layout_zones.file_picker_table = chunks[2];
    f.render_widget(table, chunks[2]);

    // 4. Help hint
    let hint_msg = if lang == "zh" {
        "▲/▼: 选择 | Enter: 打开/确认 | Backspace: 返回上级 (或删除字符) | Esc: 关闭"
    } else {
        "▲/▼: Select | Enter: Open/Confirm | Backspace: Up Directory (or delete) | Esc: Close"
    };
    let hint = Paragraph::new(hint_msg)
        .alignment(Alignment::Center)
        .style(Style::default().fg(CATPPUCCIN_MOCHA.text_muted).bg(mocha::MANTLE));
    f.render_widget(hint, chunks[3]);
}

