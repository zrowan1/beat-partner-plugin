use egui::Ui;

pub fn spectrum_view(ui: &mut Ui, data: &[f32]) {
    let desired_size = egui::vec2(ui.available_width(), 200.0);
    let (response, painter) = ui.allocate_painter(desired_size, egui::Sense::hover());
    let rect = response.rect;

    let bar_count = data.len();
    if bar_count == 0 {
        return;
    }

    let bar_width = rect.width() / bar_count as f32;
    let gap = 1.0f32;
    let effective_bar_width = (bar_width - gap).max(1.0);

    for (i, &value) in data.iter().enumerate() {
        let t = (i as f32 / bar_count.max(1) as f32).clamp(0.0, 1.0);

        // Gradient from cyan (low freq) to purple (high freq)
        let color_low = crate::ui::theme::ACCENT_CYAN;
        let color_high = crate::ui::theme::ACCENT_PURPLE;
        let bar_color = egui::Color32::from_rgb(
            (color_low.r() as f32 * (1.0 - t) + color_high.r() as f32 * t) as u8,
            (color_low.g() as f32 * (1.0 - t) + color_high.g() as f32 * t) as u8,
            (color_low.b() as f32 * (1.0 - t) + color_high.b() as f32 * t) as u8,
        );

        let x = rect.min.x + i as f32 * bar_width;
        let bar_height = value * rect.height();
        let bar_rect = egui::Rect::from_min_size(
            egui::pos2(x, rect.max.y - bar_height),
            egui::vec2(effective_bar_width, bar_height),
        );

        painter.rect_filled(bar_rect, 2.0, bar_color);
    }
}
