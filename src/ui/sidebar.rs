use crate::app::App;
use ratatui::{Frame, layout::Rect};

pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    app.layout_zones.monitor_panel = area;
    crate::ui::channels::draw_guided_burning_panel(f, app, area);
}
