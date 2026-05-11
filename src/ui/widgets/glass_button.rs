use crate::ui::theme::*;
use egui::{Response, Stroke, Ui, WidgetText};

pub fn glass_button_widget(ui: &mut Ui, label: impl Into<WidgetText>) -> Response {
    ui.add(
        egui::Button::new(label)
            .fill(SURFACE_SECONDARY)
            .corner_radius(8.0)
            .stroke(Stroke::new(1.0, BORDER)),
    )
}
