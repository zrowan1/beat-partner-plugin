use crate::ui::theme::*;
use egui::{Frame, Stroke, Ui};

pub fn glass_panel_widget(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    Frame::new()
        .fill(GLASS_BG)
        .corner_radius(12.0)
        .stroke(Stroke::new(1.0, BORDER))
        .inner_margin(16.0)
        .show(ui, add_contents);
}
