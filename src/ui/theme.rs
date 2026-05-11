use egui::{Color32, Frame, RichText, Stroke, Ui, WidgetText};

// ---------------------------------------------------------------------------
// Color palette — Dark glassmorphism
// ---------------------------------------------------------------------------
pub const SURFACE_PRIMARY: Color32 = Color32::from_rgb(0x02, 0x02, 0x04);
pub const SURFACE_SECONDARY: Color32 = Color32::from_rgb(0x08, 0x08, 0x0c);
pub const SURFACE_TERTIARY: Color32 = Color32::from_rgb(0x0f, 0x0f, 0x14);

pub const ACCENT_CYAN: Color32 = Color32::from_rgb(0x22, 0xd3, 0xee);
pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(0xa7, 0x8b, 0xfa);
pub const ACCENT_MAGENTA: Color32 = Color32::from_rgb(0xf4, 0x72, 0xb6);

pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xe0, 0xe0, 0xe0);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0x99, 0x99, 0x99);

pub const BORDER: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 30);
pub const GLASS_BG: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 20);

// Tag colors (for lyrics annotations)
pub const TAG_FLOW: Color32 = Color32::from_rgb(0xf5, 0xb0, 0x40);
pub const TAG_EMPHASIS: Color32 = Color32::from_rgb(0xfb, 0x71, 0x85);

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------
pub const LABEL_SIZE: f32 = 11.0;
pub const BODY_SIZE: f32 = 13.0;
pub const HEADING_SIZE: f32 = 15.0;
pub const TITLE_SIZE: f32 = 18.0;

// ---------------------------------------------------------------------------
// Glass panel container
// ---------------------------------------------------------------------------
pub fn glass_panel(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    Frame::new()
        .fill(GLASS_BG)
        .corner_radius(12.0)
        .stroke(Stroke::new(1.0, BORDER))
        .inner_margin(16.0)
        .show(ui, add_contents);
}

/// A smaller glass panel variant with less padding.
pub fn glass_panel_compact(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    Frame::new()
        .fill(GLASS_BG)
        .corner_radius(8.0)
        .stroke(Stroke::new(1.0, BORDER))
        .inner_margin(12.0)
        .show(ui, add_contents);
}

// ---------------------------------------------------------------------------
// Glass button
// ---------------------------------------------------------------------------
pub fn glass_button(ui: &mut Ui, label: impl Into<WidgetText>) -> egui::Response {
    ui.add(
        egui::Button::new(label)
            .fill(SURFACE_SECONDARY)
            .corner_radius(8.0)
            .stroke(Stroke::new(1.0, BORDER)),
    )
}

/// A glass button that appears selected (accent border).
pub fn glass_button_selected(ui: &mut Ui, label: impl Into<WidgetText>) -> egui::Response {
    ui.add(
        egui::Button::new(label)
            .fill(SURFACE_TERTIARY)
            .corner_radius(8.0)
            .stroke(Stroke::new(1.5, ACCENT_CYAN)),
    )
}

// ---------------------------------------------------------------------------
// Text helpers
// ---------------------------------------------------------------------------
pub fn label_text(text: &str) -> RichText {
    RichText::new(text).size(LABEL_SIZE).color(TEXT_SECONDARY)
}

pub fn body_text(text: &str) -> RichText {
    RichText::new(text).size(BODY_SIZE).color(TEXT_PRIMARY)
}

pub fn heading_text(text: &str) -> RichText {
    RichText::new(text).size(HEADING_SIZE).color(TEXT_PRIMARY)
}

pub fn title_text(text: &str) -> RichText {
    RichText::new(text).size(TITLE_SIZE).color(TEXT_PRIMARY)
}

pub fn mono_text(text: &str) -> RichText {
    RichText::new(text)
        .size(BODY_SIZE)
        .color(TEXT_PRIMARY)
        .monospace()
}

// ---------------------------------------------------------------------------
// Apply global egui style overrides
// ---------------------------------------------------------------------------
pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.visuals.override_text_color = Some(TEXT_PRIMARY);
    style.visuals.panel_fill = SURFACE_PRIMARY;
    style.visuals.window_fill = SURFACE_SECONDARY;
    style.visuals.window_stroke = Stroke::new(1.0, BORDER);
    style.visuals.widgets.noninteractive.bg_fill = SURFACE_SECONDARY;
    style.visuals.widgets.inactive.bg_fill = SURFACE_TERTIARY;
    style.visuals.widgets.hovered.bg_fill = SURFACE_TERTIARY;
    style.visuals.widgets.active.bg_fill = SURFACE_TERTIARY;
    style.visuals.widgets.open.bg_fill = SURFACE_TERTIARY;
    style.visuals.selection.bg_fill = ACCENT_CYAN;
    style.visuals.selection.stroke = Stroke::new(1.0, ACCENT_CYAN);

    ctx.set_style(style);
}
