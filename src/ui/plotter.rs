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
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(10),
            Constraint::Length(5),
        ])
        .split(area);

    draw_header_bar(f, app, main_layout[0]);

    let body_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(36)])
        .split(main_layout[1]);

    draw_connection_rail(f, app, body_layout[0]);
    draw_chart_panel(f, app, body_layout[1]);
    draw_send_panel(f, app, main_layout[2]);
}

fn draw_header_bar(f: &mut Frame, app: &App, area: Rect) {
    let selected_port = app
        .get_selected_port()
        .unwrap_or_else(|| "NONE".to_string());
    let is_running = app.simulation_active;
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

    let lines = vec![
        Line::from(vec![
            Span::styled(
                tr("plot_title", lang),
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.text)
                    .bg(mocha::SURFACE1)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            pill(if lang == "zh" { "端口" } else { "Port" }, &selected_port, CATPPUCCIN_MOCHA.primary),
            Span::raw("  "),
            pill(if lang == "zh" { "协议" } else { "Protocol" }, &protocol_str, CATPPUCCIN_MOCHA.accent),
            Span::raw("  "),
            pill(if lang == "zh" { "视图" } else { "View" }, &view_str, CATPPUCCIN_MOCHA.info),
            Span::raw("  "),
            pill(if lang == "zh" { "状态" } else { "State" }, stream_label, stream_color),
        ]),
        Line::from(vec![
            key_hint("Left/Right"),
            Span::raw(tr("plot_port_hint", lang)),
            key_hint("M"),
            Span::raw(tr("plot_protocol_hint", lang)),
            key_hint("V"),
            Span::raw(tr("plot_view_hint", lang)),
            key_hint("Space/S"),
            Span::raw(tr("plot_start_hint", lang)),
            key_hint("C"),
            Span::raw(tr("plot_clear_hint", lang)),
        ]),
    ];

    let p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(CATPPUCCIN_MOCHA.border)),
        )
        .style(Style::default().bg(mocha::MANTLE));
    f.render_widget(p, area);
}

fn pill(label: &str, value: &str, color: ratatui::style::Color) -> Span<'static> {
    Span::styled(
        format!(" {}: {} ", label, value),
        Style::default()
            .fg(color)
            .bg(mocha::SURFACE0)
            .add_modifier(Modifier::BOLD),
    )
}

fn key_hint<'a>(key: &'a str) -> Span<'a> {
    Span::styled(
        key,
        Style::default()
            .fg(CATPPUCCIN_MOCHA.accent)
            .add_modifier(Modifier::BOLD),
    )
}

fn draw_chart_panel(f: &mut Frame, app: &App, area: Rect) {
    let lang = &app.tool_config.language;
    match app.plotter_mode {
        crate::app::PlotterMode::IMUCube | crate::app::PlotterMode::RoiImage => {
            draw_receive_console(f, app, area);
        }
        _ => {
            let requested_port = app.get_selected_port();
            let requested_has_points = requested_port
                .as_ref()
                .and_then(|port| app.waveform_history.get(port))
                .is_some_and(|points| !points.is_empty());
            let simulated_has_points = app
                .waveform_history
                .get("SIMULATED")
                .is_some_and(|points| !points.is_empty());
            let selected_port = if requested_has_points {
                requested_port
            } else if simulated_has_points {
                Some("SIMULATED".to_string())
            } else {
                requested_port
            };
            let history = selected_port
                .as_ref()
                .and_then(|port| app.waveform_history.get(port));

            match history {
                Some(points) if !points.is_empty() => {
                    match app.plotter_mode {
                        crate::app::PlotterMode::Waveform => {
                            let num_channels = points[0].len();

                            let mut datasets_data: Vec<Vec<(f64, f64)>> =
                                vec![Vec::new(); num_channels];
                            for (x_idx, val_vector) in points.iter().enumerate() {
                                for (ch_idx, &val) in val_vector.iter().enumerate() {
                                    if ch_idx < num_channels {
                                        datasets_data[ch_idx].push((x_idx as f64, val as f64));
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
                                let color = colors[ch_idx % colors.len()];
                                let dataset = Dataset::default()
                                    .name(format!("CH {}", ch_idx))
                                    .marker(Marker::Braille)
                                    .graph_type(GraphType::Line)
                                    .style(Style::default().fg(color))
                                    .data(&datasets_data[ch_idx]);
                                datasets.push(dataset);
                            }

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
                                min_y -= diff * 0.1;
                                max_y += diff * 0.1;
                            }

                            let x_labels = vec![
                                Span::styled("0", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
                                Span::styled(
                                    "50",
                                    Style::default().fg(CATPPUCCIN_MOCHA.text_muted),
                                ),
                                Span::styled(
                                    "100 (Latest)",
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

                            let status_tag = if app.simulation_active {
                                ""
                            } else {
                                if lang == "zh" { " [已暂停]" } else { " [PAUSED]" }
                            };
                            let title_str = format!(
                                " {} ({}){} ",
                                if lang == "zh" { "波形观测器 Scope" } else { "Waveform Scope" },
                                selected_port.unwrap(),
                                status_tag
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
                                        .bounds([0.0, 100.0])
                                        .labels(x_labels)
                                        .style(Style::default().fg(CATPPUCCIN_MOCHA.border)),
                                )
                                .y_axis(
                                    ratatui::widgets::Axis::default()
                                        .bounds([min_y, max_y])
                                        .labels(y_labels)
                                        .style(Style::default().fg(CATPPUCCIN_MOCHA.border)),
                                );

                            f.render_widget(chart, area);
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

                            let status_tag = if app.simulation_active {
                                ""
                            } else {
                                if lang == "zh" { " [已暂停]" } else { " [PAUSED]" }
                            };
                            let title_str = format!(
                                " {} ({}){} ",
                                if lang == "zh" { "柱状图 - 通道实时数值" } else { "Bar Chart - Latest Channel Values" },
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
                                let helper = Paragraph::new(
                                    if lang == "zh" { "无活动数据用于通道 0 直方图计算。" } else { "No active data for CH 0 Histogram calculation." },
                                )
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

                                let status_tag = if app.simulation_active {
                                    ""
                                } else {
                                    if lang == "zh" { " [已暂停]" } else { " [PAUSED]" }
                                };
                                let title_str = format!(
                                    " {} ({}){} ",
                                    if lang == "zh" { "实时直方图 - 通道 0 统计分布" } else { "Real-Time Histogram - CH 0 Statistical Distribution" },
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
                                            if lang == "zh" { format!("范围: [{:.1}, {:.1}]", min_val, max_val) } else { format!("Range: [{:.1}, {:.1}]", min_val, max_val) },
                                        );
                                        ctx.print(
                                            4.5,
                                            max_count * 0.95,
                                            if lang == "zh" { format!("样本数: {}", ch0_vals.len()) } else { format!("Samples: {}", ch0_vals.len()) },
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
                                let helper = Paragraph::new(
                                    if lang == "zh" { "数据点不足，无法进行 FFT 分析（需要至少 8 个样本）。" } else { "Insufficient data points for FFT analysis (Need >= 8 samples)." },
                                )
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

                                let status_tag = if app.simulation_active {
                                    ""
                                } else {
                                    if lang == "zh" { " [已暂停]" } else { " [PAUSED]" }
                                };
                                let title_str = format!(
                                    " {} ({}){} ",
                                    if lang == "zh" { "实时 FFT 频谱图 - 通道 0 频域" } else { "Real-Time FFT Spectrum - CH 0 Frequency Domain" },
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
                                            if lang == "zh" { format!("峰值频率: {:.2} Hz", peak_freq) } else { format!("Peak Freq: {:.2} Hz", peak_freq) },
                                        );
                                        ctx.print(
                                            9.0,
                                            max_mag * 0.95,
                                            if lang == "zh" { format!("分辨率: {:.4} Hz/bin", hz_per_bin) } else { format!("Resolution: {:.4} Hz/bin", hz_per_bin) },
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
            if lang == "zh" { "接收控制台" } else { "RX Console" },
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
            if lang == "zh" { "  自动滚动 开启  " } else { "  AutoScroll ON  " },
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
        Span::styled(tr("plot_port", lang), Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
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
    let mut ports: Vec<String> = app.channels.iter().map(|c| c.port.clone()).collect();
    ports.push("SIMULATED".to_string());

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
                let status = if port == "SIMULATED" {
                    if app.simulation_active {
                        tr("plot_sim_on", lang)
                    } else {
                        tr("plot_sim_off", lang)
                    }
                } else {
                    tr("plot_active", lang)
                };
                let status_color = if port == "SIMULATED" && !app.simulation_active {
                    CATPPUCCIN_MOCHA.warning
                } else {
                    CATPPUCCIN_MOCHA.success
                };
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
                let suffix = if port == "SIMULATED" {
                    if app.simulation_active {
                        tr("plot_sim_on", lang)
                    } else {
                        tr("plot_sim_off", lang)
                    }
                } else {
                    ""
                };
                Line::from(vec![
                    Span::styled("  ", Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
                    Span::styled(port, Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
                    Span::styled(suffix, Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
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
    let status_color = if app.simulation_active {
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
                if app.simulation_active {
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
            Span::styled(tr("plot_none", lang), Style::default().fg(CATPPUCCIN_MOCHA.text)),
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
            vec![Row::new(vec![Cell::from(
                tr("plot_waiting_data", lang),
            )])],
            [Constraint::Percentage(100)],
        )
    } else {
        let headers = if lang == "zh" {
            vec!["通道", "当前值", "最小值", "最大值", "平均值"]
        } else {
            vec!["Channel", "Current", "Min", "Max", "Avg"]
        };
        let header_cells = headers
            .into_iter()
            .map(|h| {
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
            Span::styled(tr("plot_tx_mode", lang), Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
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
            Span::styled(tr("plot_tx_eol", lang), Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
            Span::styled("\\n", Style::default().fg(CATPPUCCIN_MOCHA.text)),
            Span::raw("  "),
            Span::styled(tr("plot_port", lang), Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
            Span::styled(
                selected_port,
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.primary)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "> ",
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                tr("plot_tx_placeholder", lang),
                Style::default().fg(CATPPUCCIN_MOCHA.text_disabled),
            ),
            Span::raw(" "),
            Span::styled(
                tr("plot_tx_enter_send", lang),
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.success)
                    .bg(mocha::SURFACE0),
            ),
            Span::raw(" "),
            Span::styled(
                tr("plot_tx_history_hint", lang),
                Style::default()
                    .fg(CATPPUCCIN_MOCHA.text_muted)
                    .bg(mocha::SURFACE0),
            ),
        ]),
        Line::from(vec![
            Span::styled(tr("plot_tx_quick", lang), Style::default().fg(CATPPUCCIN_MOCHA.text_muted)),
            quick_button("RESET"),
            Span::raw(" "),
            quick_button("VERSION?"),
            Span::raw(" "),
            quick_button("START"),
            Span::raw(" "),
            quick_button("STOP"),
            Span::raw(" "),
            quick_button("QA PING"),
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

fn quick_button<'a>(label: &'a str) -> Span<'a> {
    Span::styled(
        format!("[{}]", label),
        Style::default()
            .fg(CATPPUCCIN_MOCHA.text)
            .bg(mocha::SURFACE0)
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
