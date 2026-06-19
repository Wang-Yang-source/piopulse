use crate::app::App;
use crate::ui::theme::{CATPPUCCIN_MOCHA, mocha};
use crate::ui::tr;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::canvas::{Canvas, Line as CanvasLine},
    widgets::{
        Block, BorderType, Borders, Cell, Chart, Dataset, GraphType, Paragraph, Row, Table, Wrap,
    },
};

pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    let show_send_panel = area.height >= 14;
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(6),
            Constraint::Length(if show_send_panel { 4 } else { 0 }),
        ])
        .split(area);

    app.layout_zones.plotter_header = main_layout[0];
    app.layout_zones.plotter_send_panel = if show_send_panel { main_layout[2] } else { Rect::default() };

    draw_header_bar(f, app, main_layout[0]);

    if main_layout[1].width < 88 {
        app.layout_zones.plotter_port_selector = Rect::default();
        let show_stats = main_layout[1].height >= 14;
        let body_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(6),
                Constraint::Length(if show_stats { 8 } else { 0 }),
            ])
            .split(main_layout[1]);

        draw_chart_panel(f, app, body_layout[0]);
        if show_stats {
            draw_telemetry_stats(f, app, body_layout[1]);
        }
    } else {
        let body_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(28), Constraint::Min(36)])
            .split(main_layout[1]);

        draw_connection_rail(f, app, body_layout[0]);
        draw_chart_panel(f, app, body_layout[1]);
    }

    if show_send_panel {
        draw_send_panel(f, app, main_layout[2]);
    }
}

fn draw_header_bar(f: &mut Frame, app: &App, area: Rect) {
    let selected_port = app
        .get_selected_port()
        .unwrap_or_else(|| "NONE".to_string());
    let is_running = app.plotter_active;
    let lang = &app.tool_config.language;
    let stream_label = if is_running {
        if lang == "zh" { "运行中" } else { "RUNNING" }
    } else {
        if lang == "zh" { "已暂停" } else { "PAUSED" }
    };
    let stream_color = if is_running {
        CATPPUCCIN_MOCHA.success
    } else {
        CATPPUCCIN_MOCHA.warning
    };

    let protocol_str = format!("{:?}", app.vofa_mode);
    let view_str = format!("{:?}", app.plotter_mode);
    let items = crate::app::plotter_header_items(
        lang,
        true,
        selected_port,
        protocol_str,
        view_str,
        stream_label.to_string(),
    );

    let lines = vec![Line::from(vec![
        Span::styled(
            tr("plot_title", lang),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .bg(mocha::SURFACE1)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        pill(
            &items[0].0,
            &items[0].1,
            CATPPUCCIN_MOCHA.primary,
            app.hover_plotter_header_action == Some(0),
        ),
        Span::raw("  "),
        pill(
            &items[1].0,
            &items[1].1,
            CATPPUCCIN_MOCHA.accent,
            app.hover_plotter_header_action == Some(1),
        ),
        Span::raw("  "),
        pill(
            &items[2].0,
            &items[2].1,
            CATPPUCCIN_MOCHA.info,
            app.hover_plotter_header_action == Some(2),
        ),
        Span::raw("  "),
        pill(
            &items[3].0,
            &items[3].1,
            stream_color,
            app.hover_plotter_header_action == Some(3),
        ),
    ])];

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border)),
        )
        .style(Style::default().bg(mocha::MANTLE));
    f.render_widget(p, area);
}

fn pill(label: &str, value: &str, color: ratatui::style::Color, hovered: bool) -> Span<'static> {
    Span::styled(
        format!(" {}: {} ", label, value),
        Style::default()
            .fg(color)
            .bg(if hovered {
                CATPPUCCIN_MOCHA.selection_bg
            } else {
                mocha::SURFACE0
            })
            .add_modifier(
                Modifier::BOLD
                    | if hovered {
                        Modifier::REVERSED
                    } else {
                        Modifier::empty()
                    },
            ),
    )
}

fn draw_chart_panel(f: &mut Frame, app: &App, area: Rect) {
    let lang = &app.tool_config.language;
    match app.plotter_mode {
        crate::app::PlotterMode::IMUCube | crate::app::PlotterMode::RoiImage => {
            draw_receive_console(f, app, area);
        }
        _ => {
            let selected_port = app.get_selected_port();
            let history = selected_port
                .as_ref()
                .and_then(|port| app.waveform_history.get(port));

            match history {
                Some(points) if !points.is_empty() => {
                    match app.plotter_mode {
                        crate::app::PlotterMode::Waveform => {
                            let (visible_start, visible_end) =
                                waveform_visible_range(app, points.len());
                            let visible_points = &points[visible_start..visible_end];
                            let num_channels = visible_points
                                .iter()
                                .map(Vec::len)
                                .max()
                                .unwrap_or_default();
                            let sample_count = points.len();
                            let visible_count = visible_points.len();
                            let first_x = visible_start as f64;
                            let last_x = visible_end.saturating_sub(1) as f64;
                            let x_max = last_x.max(first_x + 1.0);

                            let mut datasets_data: Vec<Vec<(f64, f64)>> =
                                vec![Vec::new(); num_channels];
                            for (x_idx, val_vector) in visible_points.iter().enumerate() {
                                for (ch_idx, &val) in val_vector.iter().enumerate() {
                                    if ch_idx < num_channels && val.is_finite() {
                                        datasets_data[ch_idx]
                                            .push(((visible_start + x_idx) as f64, val as f64));
                                    }
                                }
                            }

                            let colors = [
                                mocha::PEACH,
                                mocha::GREEN,
                                mocha::BLUE,
                                mocha::PINK,
                                mocha::MAUVE,
                                mocha::YELLOW,
                                mocha::TEAL,
                                mocha::SKY,
                                mocha::SAPPHIRE,
                            ];

                            let mut datasets = Vec::new();
                            for ch_idx in 0..num_channels {
                                if datasets_data[ch_idx].is_empty() {
                                    continue;
                                }
                                let color = colors[ch_idx % colors.len()];
                                let dataset = Dataset::default()
                                    .name(format!("CH {}", ch_idx))
                                    .marker(Marker::Braille)
                                    .graph_type(GraphType::Line)
                                    .style(Style::default().fg(color))
                                    .data(&datasets_data[ch_idx]);
                                datasets.push(dataset);
                            }

                            let (min_y, max_y) =
                                padded_y_bounds(visible_points, 0.10).unwrap_or((-10.0, 10.0));
                            let trigger_y = (min_y + max_y) / 2.0;
                            let x_bounds = [first_x, x_max];
                            let zero_line = vec![(first_x, 0.0), (x_max, 0.0)];
                            let trigger_line = vec![(first_x, trigger_y), (x_max, trigger_y)];
                            let cursor_line = vec![(last_x, min_y), (last_x, max_y)];

                            if min_y < 0.0 && max_y > 0.0 {
                                datasets.push(
                                    Dataset::default()
                                        .name(if lang == "zh" { "零位" } else { "Zero" })
                                        .marker(Marker::Braille)
                                        .graph_type(GraphType::Line)
                                        .style(Style::default().fg(CATPPUCCIN_MOCHA.border))
                                        .data(&zero_line),
                                );
                            }

                            datasets.push(
                                Dataset::default()
                                    .name(if lang == "zh" {
                                        "触发参考"
                                    } else {
                                        "Trigger Ref"
                                    })
                                    .marker(Marker::Braille)
                                    .graph_type(GraphType::Line)
                                    .style(Style::default().fg(CATPPUCCIN_MOCHA.warning))
                                    .data(&trigger_line),
                            );
                            datasets.push(
                                Dataset::default()
                                    .name(if lang == "zh" {
                                        "最新采样"
                                    } else {
                                        "Latest"
                                    })
                                    .marker(Marker::Braille)
                                    .graph_type(GraphType::Line)
                                    .style(Style::default().fg(CATPPUCCIN_MOCHA.text_muted))
                                    .data(&cursor_line),
                            );

                            let chart_area = if area.height >= 16 {
                                let chunks = Layout::default()
                                    .direction(Direction::Vertical)
                                    .constraints([Constraint::Length(3), Constraint::Min(8)])
                                    .split(area);
                                draw_waveform_info_bar(
                                    f,
                                    app,
                                    visible_points,
                                    sample_count,
                                    visible_start,
                                    visible_end,
                                    chunks[0],
                                );
                                chunks[1]
                            } else {
                                area
                            };

                            let x_labels = vec![
                                Span::styled(
                                    visible_start.to_string(),
                                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                                ),
                                Span::styled(
                                    format!("{:.0}", (first_x + last_x) / 2.0),
                                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                                ),
                                Span::styled(
                                    format!("{:.0} {}", last_x, tr("plot_latest", lang)),
                                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                                ),
                            ];

                            let y_labels = vec![
                                Span::styled(
                                    format!("{:.1}", min_y),
                                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                                ),
                                Span::styled(
                                    format!("{:.1}", (min_y + max_y) / 2.0),
                                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                                ),
                                Span::styled(
                                    format!("{:.1}", max_y),
                                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                                ),
                            ];

                            let status_tag = if app.plotter_active {
                                ""
                            } else {
                                if lang == "zh" {
                                    " [已暂停]"
                                } else {
                                    " [PAUSED]"
                                }
                            };
                            let title_str = format!(
                                " {} ({}){} | {} {}/{} | {} {:.2} ",
                                if lang == "zh" {
                                    "波形观测器 Scope"
                                } else {
                                    "Waveform Scope"
                                },
                                selected_port.unwrap(),
                                status_tag,
                                tr("plot_samples", lang),
                                visible_count,
                                sample_count,
                                tr("plot_trigger_ref", lang),
                                trigger_y
                            );
                            let chart = Chart::new(datasets)
                                .block(
                                    Block::default()
                                        .borders(Borders::ALL)
                                        .border_type(BorderType::Rounded)
                                        .border_style(
                                            Style::default().fg(CATPPUCCIN_MOCHA.border_focus),
                                        )
                                        .title(Span::styled(
                                            title_str,
                                            Style::default()
                                                .fg(CATPPUCCIN_MOCHA.text)
                                                .add_modifier(Modifier::BOLD),
                                        )),
                                )
                                .x_axis(
                                    ratatui::widgets::Axis::default()
                                        .bounds(x_bounds)
                                        .labels(x_labels)
                                        .style(Style::default().fg(CATPPUCCIN_MOCHA.border)),
                                )
                                .y_axis(
                                    ratatui::widgets::Axis::default()
                                        .bounds([min_y, max_y])
                                        .labels(y_labels)
                                        .style(Style::default().fg(CATPPUCCIN_MOCHA.border)),
                                );

                            f.render_widget(chart, chart_area);
                        }
                        crate::app::PlotterMode::BarChart => {
                            let latest = &points[points.len() - 1];
                            let num_channels = latest.len();

                            let colors = [
                                mocha::PEACH,
                                mocha::GREEN,
                                mocha::BLUE,
                                mocha::PINK,
                                mocha::MAUVE,
                                mocha::YELLOW,
                                mocha::TEAL,
                                mocha::SKY,
                                mocha::SAPPHIRE,
                            ];

                            let mut min_y = -10.0f64;
                            let mut max_y = 10.0f64;
                            let mut has_points = false;
                            for val_vector in points {
                                for &val in val_vector {
                                    let v = val as f64;
                                    if !v.is_nan() && !v.is_infinite() {
                                        if !has_points {
                                            min_y = v;
                                            max_y = v;
                                            has_points = true;
                                        } else {
                                            if v < min_y {
                                                min_y = v;
                                            }
                                            if v > max_y {
                                                max_y = v;
                                            }
                                        }
                                    }
                                }
                            }

                            // Add padding to bounds
                            if min_y == max_y {
                                min_y -= 1.0;
                                max_y += 1.0;
                            } else {
                                let diff = max_y - min_y;
                                min_y -= diff * 0.15;
                                max_y += diff * 0.15;
                            }

                            let status_tag = if app.plotter_active {
                                ""
                            } else {
                                if lang == "zh" {
                                    " [已暂停]"
                                } else {
                                    " [PAUSED]"
                                }
                            };
                            let title_str = format!(
                                " {} ({}){} ",
                                if lang == "zh" {
                                    "柱状图 - 通道实时数值"
                                } else {
                                    "Bar Chart - Latest Channel Values"
                                },
                                selected_port.unwrap(),
                                status_tag
                            );
                            let bar_canvas = Canvas::default()
                                .block(
                                    Block::default()
                                        .borders(Borders::ALL)
                                        .border_type(BorderType::Rounded)
                                        .border_style(
                                            Style::default().fg(CATPPUCCIN_MOCHA.border_focus),
                                        )
                                        .title(Span::styled(
                                            title_str,
                                            Style::default()
                                                .fg(CATPPUCCIN_MOCHA.text)
                                                .add_modifier(Modifier::BOLD),
                                        )),
                                )
                                .x_bounds([-0.5, num_channels as f64 - 0.5])
                                .y_bounds([min_y, max_y])
                                .paint(move |ctx| {
                                    if min_y < 0.0 && max_y > 0.0 {
                                        ctx.draw(&CanvasLine {
                                            x1: -0.5,
                                            y1: 0.0,
                                            x2: num_channels as f64 - 0.5,
                                            y2: 0.0,
                                            color: CATPPUCCIN_MOCHA.border,
                                        });
                                    }

                                    for ch_idx in 0..num_channels {
                                        let val = latest[ch_idx] as f64;
                                        let color = colors[ch_idx % colors.len()];

                                        let x_center = ch_idx as f64;
                                        let bar_width = 0.25;
                                        let step = 0.05;
                                        let mut x_offset = -bar_width;
                                        while x_offset <= bar_width {
                                            ctx.draw(&CanvasLine {
                                                x1: x_center + x_offset,
                                                y1: 0.0_f64.clamp(min_y, max_y),
                                                x2: x_center + x_offset,
                                                y2: val.clamp(min_y, max_y),
                                                color,
                                            });
                                            x_offset += step;
                                        }

                                        let label_y = if val >= 0.0 {
                                            (val + (max_y - min_y) * 0.03)
                                                .min(max_y - (max_y - min_y) * 0.05)
                                        } else {
                                            (val - (max_y - min_y) * 0.05)
                                                .max(min_y + (max_y - min_y) * 0.03)
                                        };
                                        ctx.print(x_center - 0.12, label_y, format!("{:.2}", val));

                                        let bottom_label_y = min_y + (max_y - min_y) * 0.03;
                                        ctx.print(
                                            x_center - 0.12,
                                            bottom_label_y,
                                            format!("CH{}", ch_idx),
                                        );
                                    }
                                });

                            f.render_widget(bar_canvas, area);
                        }
                        crate::app::PlotterMode::Histogram => {
                            let mut ch0_vals = Vec::new();
                            for pt in points {
                                if let Some(&val) = pt.get(0) {
                                    if !val.is_nan() && !val.is_infinite() {
                                        ch0_vals.push(val as f64);
                                    }
                                }
                            }

                            if ch0_vals.is_empty() {
                                let helper = Paragraph::new(if lang == "zh" {
                                    "无活动数据用于通道 0 直方图计算。"
                                } else {
                                    "No active data for CH 0 Histogram calculation."
                                })
                                .style(Style::default().fg(CATPPUCCIN_MOCHA.text_muted))
                                .alignment(Alignment::Center);
                                f.render_widget(helper, area);
                            } else {
                                let mut min_val =
                                    ch0_vals.iter().copied().fold(f64::INFINITY, f64::min);
                                let mut max_val =
                                    ch0_vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                                if min_val == max_val {
                                    min_val -= 1.0;
                                    max_val += 1.0;
                                }

                                let mut bins = [0; 8];
                                for &v in &ch0_vals {
                                    let pct = (v - min_val) / (max_val - min_val);
                                    let bin_idx = (pct * 8.0).floor() as usize;
                                    let bin_idx = bin_idx.min(7);
                                    bins[bin_idx] += 1;
                                }

                                let max_count = bins.iter().copied().max().unwrap_or(1) as f64;

                                let status_tag = if app.plotter_active {
                                    ""
                                } else {
                                    if lang == "zh" {
                                        " [已暂停]"
                                    } else {
                                        " [PAUSED]"
                                    }
                                };
                                let title_str = format!(
                                    " {} ({}){} ",
                                    if lang == "zh" {
                                        "实时直方图 - 通道 0 统计分布"
                                    } else {
                                        "Real-Time Histogram - CH 0 Statistical Distribution"
                                    },
                                    selected_port.unwrap(),
                                    status_tag
                                );
                                let hist_canvas = Canvas::default()
                                    .block(
                                        Block::default()
                                            .borders(Borders::ALL)
                                            .border_type(BorderType::Rounded)
                                            .border_style(
                                                Style::default().fg(CATPPUCCIN_MOCHA.border_focus),
                                            )
                                            .title(Span::styled(
                                                title_str,
                                                Style::default()
                                                    .fg(CATPPUCCIN_MOCHA.text)
                                                    .add_modifier(Modifier::BOLD),
                                            )),
                                    )
                                    .x_bounds([-0.5, 7.5])
                                    .y_bounds([0.0, max_count * 1.15])
                                    .paint(move |ctx| {
                                        for bin_idx in 0..8 {
                                            let count = bins[bin_idx] as f64;
                                            let x_center = bin_idx as f64;

                                            let bar_width = 0.35;
                                            let step = 0.05;
                                            let mut x_offset = -bar_width;
                                            while x_offset <= bar_width {
                                                ctx.draw(&CanvasLine {
                                                    x1: x_center + x_offset,
                                                    y1: 0.0,
                                                    x2: x_center + x_offset,
                                                    y2: count,
                                                    color: CATPPUCCIN_MOCHA.success,
                                                });
                                                x_offset += step;
                                            }

                                            ctx.print(
                                                x_center - 0.1,
                                                count + max_count * 0.02,
                                                format!("{}", bins[bin_idx]),
                                            );

                                            let bin_min = min_val
                                                + (bin_idx as f64) * (max_val - min_val) / 8.0;
                                            let bin_max = min_val
                                                + ((bin_idx + 1) as f64) * (max_val - min_val)
                                                    / 8.0;
                                            let mid = (bin_min + bin_max) / 2.0;
                                            ctx.print(
                                                x_center - 0.25,
                                                max_count * 0.05,
                                                format!("{:.1}", mid),
                                            );
                                        }

                                        ctx.print(
                                            4.5,
                                            max_count * 1.05,
                                            if lang == "zh" {
                                                format!("范围: [{:.1}, {:.1}]", min_val, max_val)
                                            } else {
                                                format!("Range: [{:.1}, {:.1}]", min_val, max_val)
                                            },
                                        );
                                        ctx.print(
                                            4.5,
                                            max_count * 0.95,
                                            if lang == "zh" {
                                                format!("样本数: {}", ch0_vals.len())
                                            } else {
                                                format!("Samples: {}", ch0_vals.len())
                                            },
                                        );
                                    });

                                f.render_widget(hist_canvas, area);
                            }
                        }
                        crate::app::PlotterMode::FftSpectrum => {
                            let mut ch0_vals = Vec::new();
                            for pt in points {
                                if let Some(&val) = pt.get(0) {
                                    if !val.is_nan() && !val.is_infinite() {
                                        ch0_vals.push(val as f64);
                                    }
                                }
                            }

                            if ch0_vals.len() < 8 {
                                let helper = Paragraph::new(if lang == "zh" {
                                    "数据点不足，无法进行 FFT 分析（需要至少 8 个样本）。"
                                } else {
                                    "Insufficient data points for FFT analysis (Need >= 8 samples)."
                                })
                                .style(Style::default().fg(CATPPUCCIN_MOCHA.text_muted))
                                .alignment(Alignment::Center);
                                f.render_widget(helper, area);
                            } else {
                                let mut x = vec![0.0; 32];
                                let n_points = ch0_vals.len();
                                if n_points >= 32 {
                                    for i in 0..32 {
                                        x[i] = ch0_vals[n_points - 32 + i];
                                    }
                                } else {
                                    for i in 0..n_points {
                                        x[i] = ch0_vals[i];
                                    }
                                }

                                let mut magnitudes = vec![0.0; 16];
                                let mut max_mag = 0.1f64;
                                for k in 0..16 {
                                    let mut re = 0.0;
                                    let mut im = 0.0;
                                    for n in 0..32 {
                                        let angle =
                                            2.0 * std::f64::consts::PI * (k as f64) * (n as f64)
                                                / 32.0;
                                        re += x[n] * angle.cos();
                                        im -= x[n] * angle.sin();
                                    }
                                    let mag = (re * re + im * im).sqrt();
                                    let norm_mag = if k == 0 { mag / 32.0 } else { mag / 16.0 };
                                    magnitudes[k] = norm_mag;
                                    if norm_mag > max_mag {
                                        max_mag = norm_mag;
                                    }
                                }

                                let mut peak_k = 0;
                                let mut peak_val = 0.0;
                                for k in 1..16 {
                                    if magnitudes[k] > peak_val {
                                        peak_val = magnitudes[k];
                                        peak_k = k;
                                    }
                                }

                                let sampling_freq = 10.0;
                                let hz_per_bin = sampling_freq / 32.0;
                                let peak_freq = peak_k as f64 * hz_per_bin;

                                let status_tag = if app.plotter_active {
                                    ""
                                } else {
                                    if lang == "zh" {
                                        " [已暂停]"
                                    } else {
                                        " [PAUSED]"
                                    }
                                };
                                let title_str = format!(
                                    " {} ({}){} ",
                                    if lang == "zh" {
                                        "实时 FFT 频谱图 - 通道 0 频域"
                                    } else {
                                        "Real-Time FFT Spectrum - CH 0 Frequency Domain"
                                    },
                                    selected_port.unwrap(),
                                    status_tag
                                );
                                let fft_canvas = Canvas::default()
                                    .block(
                                        Block::default()
                                            .borders(Borders::ALL)
                                            .border_type(BorderType::Rounded)
                                            .border_style(
                                                Style::default().fg(CATPPUCCIN_MOCHA.border_focus),
                                            )
                                            .title(Span::styled(
                                                title_str,
                                                Style::default()
                                                    .fg(CATPPUCCIN_MOCHA.text)
                                                    .add_modifier(Modifier::BOLD),
                                            )),
                                    )
                                    .x_bounds([-0.5, 15.5])
                                    .y_bounds([0.0, max_mag * 1.15])
                                    .paint(move |ctx| {
                                        for k in 0..16 {
                                            let mag_val = magnitudes[k];
                                            let x_center = k as f64;

                                            let bar_width = 0.25;
                                            let step = 0.05;
                                            let mut x_offset = -bar_width;
                                            while x_offset <= bar_width {
                                                ctx.draw(&CanvasLine {
                                                    x1: x_center + x_offset,
                                                    y1: 0.0,
                                                    x2: x_center + x_offset,
                                                    y2: mag_val,
                                                    color: CATPPUCCIN_MOCHA.primary,
                                                });
                                                x_offset += step;
                                            }

                                            if k % 2 == 0 {
                                                let freq = k as f64 * hz_per_bin;
                                                ctx.print(
                                                    x_center - 0.3,
                                                    max_mag * 0.04,
                                                    format!("{:.1}Hz", freq),
                                                );
                                            }
                                        }

                                        ctx.print(
                                            9.0,
                                            max_mag * 1.05,
                                            if lang == "zh" {
                                                format!("峰值频率: {:.2} Hz", peak_freq)
                                            } else {
                                                format!("Peak Freq: {:.2} Hz", peak_freq)
                                            },
                                        );
                                        ctx.print(
                                            9.0,
                                            max_mag * 0.95,
                                            if lang == "zh" {
                                                format!("分辨率: {:.4} Hz/bin", hz_per_bin)
                                            } else {
                                                format!("Resolution: {:.4} Hz/bin", hz_per_bin)
                                            },
                                        );
                                    });

                                f.render_widget(fft_canvas, area);
                            }
                        }
                        crate::app::PlotterMode::IMUCube | crate::app::PlotterMode::RoiImage => {
                            unreachable!()
                        }
                    }
                }
                _ => {
                    draw_receive_console(f, app, area);
                }
            }
        }
    }
}

fn draw_waveform_info_bar(
    f: &mut Frame,
    app: &App,
    points: &[Vec<f32>],
    total_samples: usize,
    visible_start: usize,
    visible_end: usize,
    area: Rect,
) {
    let lang = &app.tool_config.language;
    let latest = points.last();
    let channel_count = points.iter().map(Vec::len).max().unwrap_or_default();
    let latest_ch0 = latest.and_then(|row| row.first()).copied();
    let (min_y, max_y) = raw_y_bounds(points).unwrap_or((0.0, 0.0));
    let trigger_y = (min_y + max_y) / 2.0;
    let ch0_stats = channel_stats(points, 0);

    let quality = if latest_ch0.is_some_and(f32::is_finite) {
        tr("plot_signal_locked", lang)
    } else {
        tr("plot_signal_waiting", lang)
    };
    let quality_color = if latest_ch0.is_some_and(f32::is_finite) {
        CATPPUCCIN_MOCHA.success
    } else {
        CATPPUCCIN_MOCHA.warning
    };

    let mut spans = vec![
        Span::styled(
            tr("plot_overview", lang),
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        metric_span(
            tr("plot_samples", lang),
            &format!("{}/{}", points.len(), total_samples),
        ),
        Span::raw("  "),
        metric_span(
            if lang == "zh" { "窗口" } else { "Window" },
            &format!("{}..{}", visible_start, visible_end.saturating_sub(1)),
        ),
        Span::raw("  "),
        metric_span(
            if lang == "zh" { "偏移" } else { "Offset" },
            &app.plotter_view_offset.to_string(),
        ),
        Span::raw("  "),
        metric_span(tr("plot_channels", lang), &channel_count.to_string()),
        Span::raw("  "),
        metric_span(
            tr("plot_range", lang),
            &format!("{:.2}..{:.2}", min_y, max_y),
        ),
        Span::raw("  "),
        metric_span(tr("plot_trigger_ref", lang), &format!("{:.2}", trigger_y)),
        Span::raw("  "),
        Span::styled(
            quality,
            Style::default()
                .fg(quality_color)
                .bg(mocha::SURFACE0)
                .add_modifier(Modifier::BOLD),
        ),
    ];

    if let Some(stats) = ch0_stats {
        spans.push(Span::raw("  "));
        spans.push(metric_span(
            if lang == "zh" { "CH0均值" } else { "CH0 Avg" },
            &format!("{:.2}", stats.avg),
        ));
    }

    let panel = Paragraph::new(Line::from(spans))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border)),
        )
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel))
        .wrap(Wrap { trim: true });

    f.render_widget(panel, area);
}

fn waveform_visible_range(app: &App, total: usize) -> (usize, usize) {
    if total == 0 {
        return (0, 0);
    }

    let span = app.plotter_view_samples.clamp(1, 100).min(total);
    let max_offset = total.saturating_sub(span);
    let offset = app.plotter_view_offset.min(max_offset);
    let end = total.saturating_sub(offset).max(1);
    let start = end.saturating_sub(span);

    (start, end)
}

fn metric_span(label: &str, value: &str) -> Span<'static> {
    Span::styled(
        format!("{} {}", label, value),
        Style::default()
            .fg(CATPPUCCIN_MOCHA.info)
            .bg(mocha::SURFACE0),
    )
}

#[derive(Clone, Copy)]
struct ChannelStats {
    avg: f32,
}

fn channel_stats(points: &[Vec<f32>], channel: usize) -> Option<ChannelStats> {
    let mut sum = 0.0;
    let mut count = 0;

    for row in points {
        if let Some(&value) = row.get(channel) {
            if value.is_finite() {
                sum += value;
                count += 1;
            }
        }
    }

    if count == 0 {
        None
    } else {
        Some(ChannelStats {
            avg: sum / count as f32,
        })
    }
}

fn padded_y_bounds(points: &[Vec<f32>], padding: f64) -> Option<(f64, f64)> {
    let (mut min_y, mut max_y) = raw_y_bounds(points)?;

    if min_y == max_y {
        min_y -= 1.0;
        max_y += 1.0;
    } else {
        let diff = max_y - min_y;
        min_y -= diff * padding;
        max_y += diff * padding;
    }

    Some((min_y, max_y))
}

fn raw_y_bounds(points: &[Vec<f32>]) -> Option<(f64, f64)> {
    let mut min_y = 0.0f64;
    let mut max_y = 0.0f64;
    let mut has_points = false;

    for val_vector in points {
        for &val in val_vector {
            if val.is_finite() {
                let value = val as f64;
                if has_points {
                    min_y = min_y.min(value);
                    max_y = max_y.max(value);
                } else {
                    min_y = value;
                    max_y = value;
                    has_points = true;
                }
            }
        }
    }

    has_points.then_some((min_y, max_y))
}

fn draw_connection_rail(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(9),
            Constraint::Min(8),
        ])
        .split(area);

    app.layout_zones.plotter_port_selector = chunks[0];
    draw_port_selector(f, app, chunks[0]);
    draw_serial_profile(f, app, chunks[1]);
    draw_telemetry_stats(f, app, chunks[2]);
}

fn draw_receive_console(f: &mut Frame, app: &App, area: Rect) {
    let selected_port = app
        .get_selected_port()
        .unwrap_or_else(|| "NONE".to_string());
    let mut console_lines = Vec::new();
    let lang = &app.tool_config.language;

    console_lines.push(Line::from(vec![
        Span::styled(
            if lang == "zh" {
                "接收控制台"
            } else {
                "RX Console"
            },
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "  UTF-8  ",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.success)
                .bg(mocha::SURFACE0),
        ),
        Span::styled(
            "  HEX  ",
            Style::default()
                .fg(CATPPUCCIN_MOCHA.text_muted)
                .bg(mocha::SURFACE0),
        ),
        Span::styled(
            if lang == "zh" {
                "  自动滚动 开启  "
            } else {
                "  AutoScroll ON  "
            },
            Style::default()
                .fg(CATPPUCCIN_MOCHA.info)
                .bg(mocha::SURFACE0),
        ),
    ]));
    console_lines.push(Line::from(""));

    let visible_logs: Vec<&String> = app.logs.iter().rev().take(10).collect();
    if visible_logs.is_empty() {
        console_lines.push(Line::from(Span::styled(
            tr("plot_no_data", lang),
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        )));
    } else {
        for log in visible_logs.iter().rev() {
            let color = if log.contains("FAILED") || log.contains("failed") || log.contains("Error")
            {
                CATPPUCCIN_MOCHA.danger
            } else if log.contains("SUCCESS") || log.contains("PASSED") || log.contains("Success") {
                CATPPUCCIN_MOCHA.success
            } else {
                CATPPUCCIN_MOCHA.text_muted
            };
            console_lines.push(Line::from(Span::styled(
                log.to_string(),
                Style::default().fg(color),
            )));
        }
    }

    let footer = Line::from(vec![
        Span::styled("RX ", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
        Span::styled(
            format!("{}{}", app.logs.len(), tr("plot_lines", lang)),
            Style::default().fg(CATPPUCCIN_MOCHA.text),
        ),
        Span::raw("   "),
        Span::styled(
            tr("plot_port", lang),
            Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
        ),
        Span::styled(
            selected_port,
            Style::default()
                .fg(CATPPUCCIN_MOCHA.primary)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    console_lines.push(Line::from(""));
    console_lines.push(footer);

    let p = Paragraph::new(console_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border_focus))
                .title(Span::styled(
                    tr("plot_rx_parsed_title", lang),
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.text)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .style(Style::default().bg(mocha::MANTLE))
        .wrap(Wrap { trim: false });

    f.render_widget(p, area);
}

fn draw_port_selector(f: &mut Frame, app: &App, area: Rect) {
    let ports: Vec<String> = app.channels.iter().map(|c| c.port.clone()).collect();

    let active_port = app
        .get_selected_port()
        .unwrap_or_else(|| "NONE".to_string());

    let lang = &app.tool_config.language;
    let mut port_lines = Vec::new();
    if ports.is_empty() {
        port_lines.push(Line::from(Span::styled(
            tr("plot_no_ports", lang),
            Style::default().fg(CATPPUCCIN_MOCHA.danger),
        )));
    } else {
        for port in &ports {
            let is_selected = port == &active_port;
            let line = if is_selected {
                let status = tr("plot_active", lang);
                let status_color = CATPPUCCIN_MOCHA.success;
                Line::from(vec![
                    Span::styled(
                        "> ",
                        Style::default()
                            .fg(CATPPUCCIN_MOCHA.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        port,
                        Style::default()
                            .fg(CATPPUCCIN_MOCHA.text)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(status, Style::default().fg(status_color)),
                ])
            } else {
                Line::from(vec![
                    Span::styled("  ", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
                    Span::styled(port, Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
                ])
            };
            port_lines.push(line);
        }
    }

    let p = Paragraph::new(port_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
                .title(Span::styled(
                    tr("plot_ports_list", lang),
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.text)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    f.render_widget(p, area);
}

fn draw_serial_profile(f: &mut Frame, app: &App, area: Rect) {
    let status_color = if app.plotter_active {
        CATPPUCCIN_MOCHA.success
    } else {
        CATPPUCCIN_MOCHA.warning
    };
    let lang = &app.tool_config.language;
    let lines = vec![
        Line::from(vec![
            Span::styled(
                tr("plot_state", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                if app.plotter_active {
                    tr("plot_connected", lang)
                } else {
                    tr("plot_paused", lang)
                },
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                tr("plot_baud", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled("115200", Style::default().fg(CATPPUCCIN_MOCHA.text)),
        ]),
        Line::from(vec![
            Span::styled(
                tr("plot_format", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled("8-N-1", Style::default().fg(CATPPUCCIN_MOCHA.text)),
        ]),
        Line::from(vec![
            Span::styled(
                tr("plot_flow", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                tr("plot_none", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                tr("plot_line_end", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled("\\n", Style::default().fg(CATPPUCCIN_MOCHA.text)),
        ]),
        Line::from(vec![
            Span::styled(
                tr("plot_buffer", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled("OK", Style::default().fg(CATPPUCCIN_MOCHA.success)),
        ]),
    ];

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
                .title(Span::styled(
                    tr("plot_profile_title", lang),
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.text)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    f.render_widget(p, area);
}

fn draw_telemetry_stats(f: &mut Frame, app: &App, area: Rect) {
    let selected_port = app.get_selected_port();
    let history = selected_port
        .as_ref()
        .and_then(|port| app.waveform_history.get(port));

    let colors = [
        mocha::PEACH,
        mocha::GREEN,
        mocha::BLUE,
        mocha::PINK,
        mocha::MAUVE,
        mocha::YELLOW,
        mocha::TEAL,
        mocha::SKY,
        mocha::SAPPHIRE,
    ];

    let mut rows = Vec::new();

    if let Some(points) = history {
        if !points.is_empty() {
            let latest = &points[points.len() - 1];
            let num_channels = latest.len();

            for ch_idx in 0..num_channels {
                let mut min = f32::MAX;
                let mut max = f32::MIN;
                let mut sum = 0.0;
                let mut count = 0;

                for pt in points {
                    if let Some(&val) = pt.get(ch_idx) {
                        if !val.is_nan() && !val.is_infinite() {
                            if val < min {
                                min = val;
                            }
                            if val > max {
                                max = val;
                            }
                            sum += val;
                            count += 1;
                        }
                    }
                }

                let avg = if count > 0 { sum / count as f32 } else { 0.0 };
                let current = latest.get(ch_idx).cloned().unwrap_or(0.0);
                let color = colors[ch_idx % colors.len()];

                let ch_cell = Cell::from(Span::styled(
                    format!("■ CH {}", ch_idx),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ));
                let cur_cell = Cell::from(Span::styled(
                    format!("{:.2}", current),
                    Style::default().fg(CATPPUCCIN_MOCHA.text),
                ));
                let min_cell = Cell::from(Span::styled(
                    format!("{:.2}", min),
                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                ));
                let max_cell = Cell::from(Span::styled(
                    format!("{:.2}", max),
                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                ));
                let avg_cell = Cell::from(Span::styled(
                    format!("{:.2}", avg),
                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                ));

                let row_style = if ch_idx % 2 == 0 {
                    Style::default().bg(mocha::MANTLE)
                } else {
                    Style::default().bg(mocha::BASE)
                };

                rows.push(
                    Row::new(vec![ch_cell, cur_cell, min_cell, max_cell, avg_cell])
                        .style(row_style),
                );
            }
        }
    }

    let lang = &app.tool_config.language;
    let stats_table = if rows.is_empty() {
        Table::new(
            vec![Row::new(vec![Cell::from(tr("plot_waiting_data", lang))])],
            [Constraint::Percentage(100)],
        )
    } else {
        let headers = if lang == "zh" {
            vec!["通道", "当前值", "最小值", "最大值", "平均值"]
        } else {
            vec!["Channel", "Current", "Min", "Max", "Avg"]
        };
        let header_cells = headers.into_iter().map(|h| {
            Cell::from(Span::styled(
                h,
                Style::default()
                    .fg(mocha::SUBTEXT1)
                    .add_modifier(Modifier::BOLD),
            ))
        });
        let header = Row::new(header_cells).style(Style::default().bg(mocha::SURFACE0));

        Table::new(
            rows,
            [
                Constraint::Percentage(24),
                Constraint::Percentage(20),
                Constraint::Percentage(18),
                Constraint::Percentage(18),
                Constraint::Percentage(20),
            ],
        )
        .header(header)
    };

    let stats_block = stats_table
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
                .title(Span::styled(
                    tr("plot_live_stats", lang),
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.text)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .style(Style::default().bg(CATPPUCCIN_MOCHA.panel));

    f.render_widget(stats_block, area);
}

fn draw_send_panel(f: &mut Frame, app: &App, area: Rect) {
    let selected_port = app
        .get_selected_port()
        .unwrap_or_else(|| "NONE".to_string());
    let lang = &app.tool_config.language;
    let has_port = app.get_selected_port().is_some();
    let tx_state = if has_port {
        tr("plot_tx_state_ready", lang)
    } else {
        tr("plot_tx_state_inactive", lang)
    };
    let tx_color = if has_port {
        CATPPUCCIN_MOCHA.success
    } else {
        CATPPUCCIN_MOCHA.text_muted
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("TX ", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
            Span::styled(
                tx_state,
                Style::default().fg(tx_color).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                tr("plot_tx_mode", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                "Text",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.info)
                    .bg(mocha::SURFACE0)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                "HEX",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.text_disabled)
                    .bg(mocha::SURFACE0),
            ),
            Span::raw("  "),
            Span::styled(
                tr("plot_tx_eol", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled("\\n", Style::default().fg(CATPPUCCIN_MOCHA.text)),
            Span::raw("  "),
            Span::styled(
                tr("plot_port", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            Span::styled(
                selected_port,
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.primary)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                tr("plot_tx_quick", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            ),
            quick_button("RESET", app.hover_plotter_quick_command == Some(0)),
            Span::raw(" "),
            quick_button("VERSION?", app.hover_plotter_quick_command == Some(1)),
            Span::raw(" "),
            quick_button("START", app.hover_plotter_quick_command == Some(2)),
            Span::raw(" "),
            quick_button("STOP", app.hover_plotter_quick_command == Some(3)),
            Span::raw(" "),
            quick_button("QA PING", app.hover_plotter_quick_command == Some(4)),
        ]),
    ];

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border_focus))
                .title(Span::styled(
                    tr("plot_tx_send_title", lang),
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.text)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .style(Style::default().bg(mocha::MANTLE))
        .wrap(Wrap { trim: true });

    f.render_widget(p, area);
}

fn quick_button(label: &str, hovered: bool) -> Span<'_> {
    Span::styled(
        format!("[{}]", label),
        Style::default()
            .fg(if hovered {
                mocha::CRUST
            } else {
                CATPPUCCIN_MOCHA.text
            })
            .bg(if hovered {
                CATPPUCCIN_MOCHA.accent
            } else {
                mocha::SURFACE0
            })
            .add_modifier(Modifier::BOLD),
    )
}

#[allow(dead_code)]
fn draw_code_reference(f: &mut Frame, app: &App, area: Rect) {
    let mode_str = match app.vofa_mode {
        crate::vofa::VofaMode::FireWater => "FireWater (CSV String)",
        crate::vofa::VofaMode::JustFloat => "JustFloat (Binary Hex)",
        crate::vofa::VofaMode::IndexFloat => "IndexFloat (Indexed Hex)",
    };

    let code_lines = match app.vofa_mode {
        crate::vofa::VofaMode::FireWater => vec![
            Line::from(Span::styled(
                "// C Language Print Template",
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            )),
            Line::from(Span::styled(
                "void send_telemetry(float a, float b) {",
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            )),
            Line::from(Span::styled(
                "  printf(\"%f,%f\\n\", a, b);",
                Style::default().fg(CATPPUCCIN_MOCHA.success),
            )),
            Line::from(Span::styled(
                "}",
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            )),
            Line::from(Span::styled(
                "Note: Separate with commas. End with newline.",
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            )),
        ],
        crate::vofa::VofaMode::JustFloat => vec![
            Line::from(Span::styled(
                "// C Language Binary Template",
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            )),
            Line::from(Span::styled(
                "void send_telemetry(float a, float b) {",
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            )),
            Line::from(Span::styled(
                "  float data[2] = {a, b};",
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            )),
            Line::from(Span::styled(
                "  uart_write(data, sizeof(data));",
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            )),
            Line::from(Span::styled(
                "  uart_write(\"\\x00\\x00\\x80\\x7f\", 4); // NaN",
                Style::default().fg(CATPPUCCIN_MOCHA.success),
            )),
            Line::from(Span::styled(
                "}",
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            )),
        ],
        crate::vofa::VofaMode::IndexFloat => vec![
            Line::from(Span::styled(
                "// C Language Indexed Binary Template",
                Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
            )),
            Line::from(Span::styled(
                "void send_telemetry(int start, float a, float b) {",
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            )),
            Line::from(Span::styled(
                "  float data[3] = {(float)start, a, b};",
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            )),
            Line::from(Span::styled(
                "  uart_write(data, sizeof(data));",
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            )),
            Line::from(Span::styled(
                "  uart_write(\"\\x00\\x00\\x80\\x7f\", 4); // NaN",
                Style::default().fg(CATPPUCCIN_MOCHA.success),
            )),
            Line::from(Span::styled(
                "}",
                Style::default().fg(CATPPUCCIN_MOCHA.text),
            )),
        ],
    };

    let title_str = format!(" C Template [{}] ", mode_str);
    let p = Paragraph::new(code_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border))
                .title(Span::styled(
                    title_str,
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.text)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .style(Style::default().bg(mocha::MANTLE));

    f.render_widget(p, area);
}
