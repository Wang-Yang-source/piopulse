use crate::app::App;
use crate::ui::theme::CATPPUCCIN_MOCHA;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::Span,
    widgets::canvas::{Canvas, Line as CanvasLine},
    widgets::{Block, BorderType, Borders},
};

/// Project a 3D point through rotation + translation + perspective projection.
/// Returns (screen_x, screen_y, depth_z) for depth-based coloring.
fn project_vertex(
    v: &[f64; 3],
    pitch: f64,
    roll: f64,
    yaw: f64,
    tx: f64,
    ty: f64,
    tz: f64,
    zoom: f64,
) -> (f64, f64, f64) {
    let (rx, ry, rz) = (v[0], v[1], v[2]);

    // Rotate around X axis (pitch)
    let cos_p = pitch.cos();
    let sin_p = pitch.sin();
    let (x1, y1, z1) = (rx, ry * cos_p - rz * sin_p, ry * sin_p + rz * cos_p);

    // Rotate around Y axis (roll)
    let cos_r = roll.cos();
    let sin_r = roll.sin();
    let (x2, y2, z2) = (x1 * cos_r + z1 * sin_r, y1, -x1 * sin_r + z1 * cos_r);

    // Rotate around Z axis (yaw)
    let cos_y = yaw.cos();
    let sin_y = yaw.sin();
    let (x3, y3, z3) = (x2 * cos_y - y2 * sin_y, x2 * sin_y + y2 * cos_y, z2);

    // Apply translation
    let nx = x3 + tx;
    let ny = y3 + ty;
    let nz = z3 + tz;

    // Perspective projection
    let distance = 3.2;
    let scale = zoom * 1.95 / (distance - nz);
    let sx = nx * scale;
    let sy = ny * scale * 0.95;

    (sx, sy, nz)
}

pub fn draw(f: &mut Frame, app: &App, area: Rect, is_focused: bool, _widget_idx: usize) {
    let (pitch, roll, yaw, tx, ty, tz) = if app.manual_imu_override {
        (
            app.manual_pitch,
            app.manual_roll,
            app.manual_yaw,
            app.manual_tx,
            app.manual_ty,
            app.manual_tz,
        )
    } else {
        let selected_port = app.get_selected_port();
        let history = selected_port
            .as_ref()
            .and_then(|port| app.waveform_history.get(port));
        if let Some(points) = history {
            if !points.is_empty() {
                let latest = &points[points.len() - 1];
                let p_val = latest.get(0).cloned().unwrap_or(0.0);
                let r_val = latest.get(1).cloned().unwrap_or(0.0);
                let y_val = latest.get(2).cloned().unwrap_or(0.0);
                let tx_val = latest.get(3).cloned().unwrap_or(0.0);
                let ty_val = latest.get(4).cloned().unwrap_or(0.0);
                let tz_val = latest.get(5).cloned().unwrap_or(0.0);

                let to_rad = |v: f32| {
                    if v.abs() > 2.0 * std::f32::consts::PI {
                        v.to_radians()
                    } else {
                        v
                    }
                };
                (
                    to_rad(p_val) as f64,
                    to_rad(r_val) as f64,
                    to_rad(y_val) as f64,
                    tx_val as f64,
                    ty_val as f64,
                    tz_val as f64,
                )
            } else {
                (0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
            }
        } else {
            (0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
        }
    };

    // Build title
    let is_zh = app.tool_config.language == "zh";
    let mode_suffix = if app.manual_imu_override {
        if is_zh {
            " [手动模式 🎮]"
        } else {
            " [MANUAL 🎮]"
        }
    } else {
        if is_zh {
            " [传感器遥测 ⚡]"
        } else {
            " [TELEMETRY ⚡]"
        }
    };

    let title_text = if is_focused {
        if is_zh {
            format!(
                " 立方体：3D 姿态{} (T: 手动, X: 坐标轴, UJIKOL: 控制, +/-: 缩放) ",
                mode_suffix
            )
        } else {
            format!(
                " cube: 3D Orientation{} (T: Manual, X: Axes, UJIKOL: Ctrl, +/-: Zoom) ",
                mode_suffix
            )
        }
    } else {
        if is_zh {
            format!(" 立方体：3D 姿态{} ", mode_suffix)
        } else {
            format!(" cube: 3D Orientation{} ", mode_suffix)
        }
    };

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

    let aspect = (area.width as f64 / area.height as f64) * 0.5;
    let x_limit = 2.0 * aspect;

    let zoom = app.cube_zoom;
    let show_axes = app.show_cube_axes;

    let cube_canvas = Canvas::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(border_type)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(
                    title_text,
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.text)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .x_bounds([-x_limit, x_limit])
        .y_bounds([-2.0, 2.0])
        .marker(Marker::Braille)
        .paint(move |ctx| {
            // ========== DEFAULT CUBE RENDERING ==========
            let size = 0.85;
            let cube_verts: [[f64; 3]; 8] = [
                [-size, -size, -size],
                [size, -size, -size],
                [size, size, -size],
                [-size, size, -size],
                [-size, -size, size],
                [size, -size, size],
                [size, size, size],
                [-size, size, size],
            ];

            let projected: Vec<(f64, f64)> = cube_verts
                .iter()
                .map(|v| {
                    let (sx, sy, _) = project_vertex(v, pitch, roll, yaw, tx, ty, tz, zoom);
                    (sx, sy)
                })
                .collect();

            let edges: [(usize, usize); 12] = [
                (0, 1),
                (1, 2),
                (2, 3),
                (3, 0),
                (4, 5),
                (5, 6),
                (6, 7),
                (7, 4),
                (0, 4),
                (1, 5),
                (2, 6),
                (3, 7),
            ];

            for &(i, j) in &edges {
                let (x1, y1) = projected[i];
                let (x2, y2) = projected[j];

                if x1.is_finite() && y1.is_finite() && x2.is_finite() && y2.is_finite() {
                    let color = if i < 4 && j < 4 {
                        crate::ui::theme::mocha::PEACH
                    } else if i >= 4 && j >= 4 {
                        CATPPUCCIN_MOCHA.success
                    } else {
                        CATPPUCCIN_MOCHA.primary
                    };

                    ctx.draw(&CanvasLine {
                        x1,
                        y1,
                        x2,
                        y2,
                        color,
                    });
                }
            }

            let text_x = -x_limit + 0.1;
            ctx.print(text_x, 1.7, format!("Pitch: {:+5.1}°", pitch.to_degrees()));
            ctx.print(text_x, 1.4, format!("Roll : {:+5.1}°", roll.to_degrees()));
            ctx.print(text_x, 1.1, format!("Yaw  : {:+5.1}°", yaw.to_degrees()));

            if show_axes {
                let x_neg = [-1.5, 0.0, 0.0];
                let x_pos = [1.5, 0.0, 0.0];
                let y_neg = [0.0, -1.5, 0.0];
                let y_pos = [0.0, 1.5, 0.0];
                let z_neg = [0.0, 0.0, -1.5];
                let z_pos = [0.0, 0.0, 1.5];

                let (xn, yn, _) = project_vertex(&x_neg, pitch, roll, yaw, tx, ty, tz, zoom);
                let (xx, xy, _) = project_vertex(&x_pos, pitch, roll, yaw, tx, ty, tz, zoom);
                let (yn_x, yn_y, _) = project_vertex(&y_neg, pitch, roll, yaw, tx, ty, tz, zoom);
                let (yx, yy, _) = project_vertex(&y_pos, pitch, roll, yaw, tx, ty, tz, zoom);
                let (zn_x, zn_y, _) = project_vertex(&z_neg, pitch, roll, yaw, tx, ty, tz, zoom);
                let (zx, zy, _) = project_vertex(&z_pos, pitch, roll, yaw, tx, ty, tz, zoom);

                let axis_red = Color::Rgb(120, 60, 60);
                let axis_green = Color::Rgb(60, 120, 60);
                let axis_blue = Color::Rgb(60, 60, 120);

                let text_red = Color::Rgb(180, 100, 100);
                let text_green = Color::Rgb(100, 180, 100);
                let text_blue = Color::Rgb(100, 100, 180);

                ctx.draw(&CanvasLine {
                    x1: xn,
                    y1: yn,
                    x2: xx,
                    y2: xy,
                    color: axis_red,
                });
                ctx.draw(&CanvasLine {
                    x1: yn_x,
                    y1: yn_y,
                    x2: yx,
                    y2: yy,
                    color: axis_green,
                });
                ctx.draw(&CanvasLine {
                    x1: zn_x,
                    y1: zn_y,
                    x2: zx,
                    y2: zy,
                    color: axis_blue,
                });

                ctx.print(xx, xy, Span::styled("X", Style::default().fg(text_red)));
                ctx.print(yx, yy, Span::styled("Y", Style::default().fg(text_green)));
                ctx.print(zx, zy, Span::styled("Z", Style::default().fg(text_blue)));
            }
        });

    f.render_widget(cube_canvas, area);
}
