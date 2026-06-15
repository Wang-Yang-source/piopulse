use crate::app::App;
use crate::ui::theme::{CATPPUCCIN_MOCHA, mocha};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::Span,
    widgets::canvas::{Canvas, Line as CanvasLine},
    widgets::{Block, BorderType, Borders},
};

pub fn draw(f: &mut Frame, app: &App, area: Rect, is_focused: bool) {
    let border_color = if is_focused {
        CATPPUCCIN_MOCHA.border_focus
    } else {
        CATPPUCCIN_MOCHA.border
    };
    let border_type = if is_focused {
        BorderType::Double
    } else {
        BorderType::Rounded
    };

    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    let mock_tx = (elapsed * 2.0).cos() * 1.0;
    let mock_ty = (elapsed * 1.5).sin() * 0.8;

    let (tx, ty) = {
        let selected_port = app.get_selected_port();
        let history = selected_port
            .as_ref()
            .and_then(|port| app.waveform_history.get(port));
        if let Some(points) = history {
            if !points.is_empty() {
                let latest = &points[points.len() - 1];
                let x_val = latest.get(0).cloned().unwrap_or(0.0) as f64;
                let y_val = latest.get(1).cloned().unwrap_or(0.0) as f64;
                (x_val, y_val)
            } else {
                (mock_tx, mock_ty)
            }
        } else {
            (mock_tx, mock_ty)
        }
    };

    let aspect = (area.width as f64 / area.height as f64) * 0.5;
    let x_limit = 1.5 * aspect;

    let canvas = Canvas::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(border_type)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(
                    format!(" {} ", crate::ui::tr("widget_image_title", &app.tool_config.language)),
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.text)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .x_bounds([-x_limit, x_limit])
        .y_bounds([-1.5, 1.5])
        .paint(move |ctx| {
            if !app.latest_image_data.is_empty() {
                let w = app.latest_image_width;
                let h = app.latest_image_height;
                let data = &app.latest_image_data;
                let is_rgb = data.len() >= w * h * 3;

                // Downsample to fit terminal canvas smoothly without lag
                let step_x = (w / 40).max(1);
                let step_y = (h / 30).max(1);

                for row_idx in (0..h).step_by(step_y) {
                    for col_idx in (0..w).step_by(step_x) {
                        let val_idx = if is_rgb {
                            (row_idx * w + col_idx) * 3
                        } else {
                            row_idx * w + col_idx
                        };
                        if val_idx < data.len() {
                            let color = if is_rgb {
                                ratatui::style::Color::Rgb(
                                    data[val_idx],
                                    data[val_idx + 1],
                                    data[val_idx + 2],
                                )
                            } else {
                                let g = data[val_idx];
                                ratatui::style::Color::Rgb(g, g, g)
                            };
                            let x =
                                -x_limit + (col_idx as f64 / (w - 1).max(1) as f64) * 2.0 * x_limit;
                            let y = 1.5 - (row_idx as f64 / (h - 1).max(1) as f64) * 3.0;
                            ctx.draw(&CanvasLine {
                                x1: x,
                                y1: y,
                                x2: x,
                                y2: y,
                                color,
                            });
                        }
                    }
                }
            } else {
                ctx.draw(&CanvasLine {
                    x1: -x_limit,
                    y1: 0.0,
                    x2: x_limit,
                    y2: 0.0,
                    color: mocha::SURFACE0,
                });
                ctx.draw(&CanvasLine {
                    x1: 0.0,
                    y1: -1.5,
                    x2: 0.0,
                    y2: 1.5,
                    color: mocha::SURFACE0,
                });

                let r = 1.3;
                for angle_idx in 0..36 {
                    let a1 = (angle_idx as f64) * 10.0f64.to_radians();
                    let a2 = ((angle_idx + 1) as f64) * 10.0f64.to_radians();
                    ctx.draw(&CanvasLine {
                        x1: a1.cos() * r,
                        y1: a1.sin() * r,
                        x2: a2.cos() * r,
                        y2: a2.sin() * r,
                        color: CATPPUCCIN_MOCHA.border,
                    });
                }
            }

            let box_size = 0.3;
            ctx.draw(&CanvasLine {
                x1: tx - box_size,
                y1: ty - box_size,
                x2: tx + box_size,
                y2: ty - box_size,
                color: CATPPUCCIN_MOCHA.success,
            });
            ctx.draw(&CanvasLine {
                x1: tx + box_size,
                y1: ty - box_size,
                x2: tx + box_size,
                y2: ty + box_size,
                color: CATPPUCCIN_MOCHA.success,
            });
            ctx.draw(&CanvasLine {
                x1: tx + box_size,
                y1: ty + box_size,
                x2: tx - box_size,
                y2: ty + box_size,
                color: CATPPUCCIN_MOCHA.success,
            });
            ctx.draw(&CanvasLine {
                x1: tx - box_size,
                y1: ty + box_size,
                x2: tx - box_size,
                y2: ty - box_size,
                color: CATPPUCCIN_MOCHA.success,
            });

            ctx.draw(&CanvasLine {
                x1: tx - 0.1,
                y1: ty,
                x2: tx + 0.1,
                y2: ty,
                color: CATPPUCCIN_MOCHA.danger,
            });
            ctx.draw(&CanvasLine {
                x1: tx,
                y1: ty - 0.1,
                x2: tx,
                y2: ty + 0.1,
                color: CATPPUCCIN_MOCHA.danger,
            });

            ctx.print(tx - 0.25, ty + box_size + 0.05, "ROI");
            ctx.print(-x_limit + 0.1, -1.2, format!("X:{:+.2} Y:{:+.2}", tx, ty));
        });

    f.render_widget(canvas, area);
}
