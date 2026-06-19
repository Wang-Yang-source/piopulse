use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub fn center_rect(percent_x: u16, height_y: u16, r: Rect) -> Rect {
    if r.width == 0 || r.height == 0 {
        return Rect::default();
    }

    let percent_x = percent_x.clamp(1, 100);
    let height_y = height_y.min(r.height);
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

pub fn responsive_modal_rect(preferred_width: u16, preferred_height: u16, r: Rect) -> Rect {
    if r.width == 0 || r.height == 0 {
        return Rect::default();
    }

    let margin_x = if r.width >= 64 { 4 } else { 1 };
    let margin_y = if r.height >= 16 { 2 } else { 0 };
    let width = preferred_width.min(r.width.saturating_sub(margin_x * 2).max(1));
    let height = preferred_height.min(r.height.saturating_sub(margin_y * 2).max(1));
    let x = r.x + r.width.saturating_sub(width) / 2;
    let y = r.y + r.height.saturating_sub(height) / 2;

    Rect {
        x,
        y,
        width,
        height,
    }
}
