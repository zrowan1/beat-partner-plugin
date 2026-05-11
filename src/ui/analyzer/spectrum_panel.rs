use egui::Ui;

pub fn spectrum_panel(ui: &mut Ui, data: &[f32]) {
    if data.is_empty() {
        ui.label(crate::ui::theme::body_text("No spectrum data available."));
        return;
    }

    crate::ui::theme::glass_panel(ui, |ui| {
        ui.heading(crate::ui::theme::heading_text("Spectrum Analyzer"));
        ui.add_space(8.0);
        crate::ui::widgets::spectrum_view::spectrum_view(ui, data);
    });
}
