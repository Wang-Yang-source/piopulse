use crate::app::App;
use crate::ui::theme::CATPPUCCIN_MOCHA;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::Span,
    widgets::canvas::{Canvas, Line as CanvasLine},
    widgets::{Block, BorderType, Borders},
};

pub fn draw(f: &mut Frame, app: &App, area: Rect, is_focused: bool) {
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

    let is_zh = app.tool_config.language == "zh";
    let title_suffix = if app.manual_imu_override {
        if is_zh { " [手动模式 🎮] " } else { " [MANUAL 🎮] " }
    } else {
        if is_zh { " [传感器遥测 ⚡] " } else { " [TELEMETRY ⚡] " }
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

    let cube_canvas = Canvas::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(border_type)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(
                    if is_focused {
                        if is_zh {
                            format!(
                                " 立方体：3D 姿态{} (T: 手动模式, UJIKOL/方向键: 控制) ",
                                title_suffix
                            )
                        } else {
                            format!(
                                " cube: 3D Orientation{} (T: Manual Mode, UJIKOL/Arrows: Ctrl) ",
                                title_suffix
                            )
                        }
                    } else {
                        if is_zh {
                            format!(" 立方体：3D 姿态{} ", title_suffix)
                        } else {
                            format!(" cube: 3D Orientation{} ", title_suffix)
                        }
                    },
                    Style::default()
                        .fg(CATPPUCCIN_MOCHA.text)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .x_bounds([-x_limit, x_limit])
        .y_bounds([-2.0, 2.0])
        .paint(move |ctx| {
            let size = 0.85;
            let vertices = [
                [-size, -size, -size],
                [size, -size, -size],
                [size, size, -size],
                [-size, size, -size],
                [-size, -size, size],
                [size, -size, size],
                [size, size, size],
                [-size, size, size],
            ];

            let mut projected = Vec::new();
            for v in &vertices {
                let (rx, ry, rz) = (v[0], v[1], v[2]);

                let rotate_x = |x: f64, y: f64, z: f64, angle: f64| {
                    let cos = angle.cos();
                    let sin = angle.sin();
                    (x, y * cos - z * sin, y * sin + z * cos)
                };
                let rotate_y = |x: f64, y: f64, z: f64, angle: f64| {
                    let cos = angle.cos();
                    let sin = angle.sin();
                    (x * cos + z * sin, y, -x * sin + z * cos)
                };
                let rotate_z = |x: f64, y: f64, z: f64, angle: f64| {
                    let cos = angle.cos();
                    let sin = angle.sin();
                    (x * cos - y * sin, x * sin + y * cos, z)
                };

                let (x1, y1, z1) = rotate_x(rx, ry, rz, pitch);
                let (x2, y2, z2) = rotate_y(x1, y1, z1, roll);
                let (x3, y3, z3) = rotate_z(x2, y2, z2, yaw);

                let nx_trans = x3 + tx;
                let ny_trans = y3 + ty;
                let nz_trans = z3 + tz;

                let distance = 3.2;
                let scale = 1.95 / (distance - nz_trans);
                let sx = nx_trans * scale;
                let sy = ny_trans * scale * 0.95;

                projected.push((sx, sy));
            }

            let edges = [
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
        });

    f.render_widget(cube_canvas, area);
}
