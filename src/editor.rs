use crate::params::BeatPartnerParams;
use crate::ui::app::{BeatPartnerApp, SharedState};
use nih_plug::prelude::*;
use nih_plug_egui::create_egui_editor;
use std::sync::Arc;

pub fn create_editor(
    params: Arc<BeatPartnerParams>,
    shared_state: Arc<SharedState>,
) -> Option<Box<dyn Editor>> {
    let app = BeatPartnerApp::new(shared_state);

    create_egui_editor(
        params.editor_state.clone(),
        app,
        |ctx, _app| {
            crate::ui::theme::apply_theme(ctx);
        },
        move |ctx, _setter, app| {
            app.update(ctx);
        },
    )
}
