use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Guides,
    Lyrics,
    Vocals,
    Theory,
    Analyzer,
    Settings,
}

impl Tab {
    pub fn label(&self) -> &'static str {
        match self {
            Tab::Guides => "Guides",
            Tab::Lyrics => "Lyrics",
            Tab::Vocals => "Vocals",
            Tab::Theory => "Theory",
            Tab::Analyzer => "Analyzer",
            Tab::Settings => "Settings",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Tab::Guides => "\u{2630}",   // trigram
            Tab::Lyrics => "\u{266A}",   // note
            Tab::Vocals => "\u{1F3A4}",  // studio mic
            Tab::Theory => "\u{266F}",   // sharp
            Tab::Analyzer => "\u{25F7}", // circle
            Tab::Settings => "\u{2699}", // gear
        }
    }
}

/// Shared state between the audio thread and the UI thread.
pub struct SharedState {
    /// Current BPM as reported by the DAW transport.
    pub daw_bpm: Arc<AtomicU32>,
    /// Whether the DAW is currently playing.
    pub is_playing: Arc<AtomicBool>,
    /// Latest detected BPM (if analysis is enabled).
    pub detected_bpm: Arc<AtomicU32>,
    /// Current project name (updated from DB).
    pub project_name: RwLock<String>,
    /// Smoothed spectrum bars (0..1 range, 128 bins).
    pub spectrum: Arc<RwLock<Vec<f32>>>,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            daw_bpm: Arc::new(AtomicU32::new(120)),
            is_playing: Arc::new(AtomicBool::new(false)),
            detected_bpm: Arc::new(AtomicU32::new(0)),
            project_name: RwLock::new("Untitled".to_string()),
            spectrum: Arc::new(RwLock::new(vec![0.0; 128])),
        }
    }
}

impl SharedState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
}

/// Main application state for the egui editor.
pub struct BeatPartnerApp {
    pub active_tab: Tab,
    pub sidebar_collapsed: bool,
    pub shared_state: Arc<SharedState>,
}

impl BeatPartnerApp {
    pub fn new(shared_state: Arc<SharedState>) -> Self {
        Self {
            active_tab: Tab::Guides,
            sidebar_collapsed: false,
            shared_state,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        crate::ui::theme::apply_theme(ctx);

        self.render_top_bar(ctx);
        self.render_sidebar(ctx);
        self.render_main_content(ctx);
        self.render_status_bar(ctx);
    }

    fn render_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_bar")
            .exact_height(40.0)
            .frame(
                egui::Frame::new()
                    .fill(crate::ui::theme::SURFACE_SECONDARY)
                    .stroke(egui::Stroke::new(1.0, crate::ui::theme::BORDER)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(12.0);
                    ui.label(crate::ui::theme::heading_text("BeatPartner"));
                    ui.add_space(24.0);

                    for tab in [
                        Tab::Guides,
                        Tab::Lyrics,
                        Tab::Theory,
                        Tab::Analyzer,
                        Tab::Settings,
                    ] {
                        let is_active = self.active_tab == tab;
                        let text = if is_active {
                            egui::RichText::new(tab.label())
                                .size(crate::ui::theme::BODY_SIZE)
                                .color(crate::ui::theme::ACCENT_CYAN)
                                .strong()
                        } else {
                            egui::RichText::new(tab.label())
                                .size(crate::ui::theme::BODY_SIZE)
                                .color(crate::ui::theme::TEXT_SECONDARY)
                        };

                        if ui
                            .add(
                                egui::Button::new(text)
                                    .fill(egui::Color32::TRANSPARENT)
                                    .frame(false),
                            )
                            .clicked()
                        {
                            self.active_tab = tab;
                        }
                        ui.add_space(8.0);
                    }
                });
            });
    }

    fn render_sidebar(&mut self, ctx: &egui::Context) {
        let width = if self.sidebar_collapsed { 48.0 } else { 150.0 };

        egui::SidePanel::left("sidebar")
            .exact_width(width)
            .frame(
                egui::Frame::new()
                    .fill(crate::ui::theme::SURFACE_SECONDARY)
                    .stroke(egui::Stroke::new(1.0, crate::ui::theme::BORDER)),
            )
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(8.0);

                    if ui
                        .add(
                            egui::Button::new(if self.sidebar_collapsed {
                                "\u{25B6}"
                            } else {
                                "\u{25C0}"
                            })
                            .fill(egui::Color32::TRANSPARENT)
                            .frame(false),
                        )
                        .clicked()
                    {
                        self.sidebar_collapsed = !self.sidebar_collapsed;
                    }

                    ui.separator();

                    let tabs = [
                        Tab::Guides,
                        Tab::Lyrics,
                        Tab::Vocals,
                        Tab::Theory,
                        Tab::Analyzer,
                        Tab::Settings,
                    ];

                    for tab in tabs {
                        let is_active = self.active_tab == tab;
                        let response = if is_active {
                            crate::ui::theme::glass_button_selected(ui, tab.icon())
                        } else {
                            crate::ui::theme::glass_button(ui, tab.icon())
                        };

                        if response.clicked() {
                            self.active_tab = tab;
                        }

                        if !self.sidebar_collapsed {
                            ui.label(
                                egui::RichText::new(tab.label())
                                    .size(crate::ui::theme::LABEL_SIZE)
                                    .color(if is_active {
                                        crate::ui::theme::ACCENT_CYAN
                                    } else {
                                        crate::ui::theme::TEXT_SECONDARY
                                    }),
                            );
                        }

                        ui.add_space(4.0);
                    }
                });
            });
    }

    fn render_main_content(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(crate::ui::theme::SURFACE_PRIMARY))
            .show(ctx, |ui| {
                crate::ui::theme::glass_panel(ui, |ui| match self.active_tab {
                    Tab::Guides => self.render_guides(ui),
                    Tab::Lyrics => self.render_lyrics(ui),
                    Tab::Vocals => self.render_vocals(ui),
                    Tab::Theory => self.render_theory(ui),
                    Tab::Analyzer => self.render_analyzer(ui),
                    Tab::Settings => self.render_settings(ui),
                });
            });
    }

    fn render_guides(&mut self, ui: &mut egui::Ui) {
        ui.heading(crate::ui::theme::heading_text("Welcome to BeatPartner"));
        ui.add_space(12.0);
        ui.label(crate::ui::theme::body_text(
            "BeatPartner is your companion inside the DAW. Use the sidebar to navigate between tools.",
        ));
    }

    fn render_lyrics(&mut self, ui: &mut egui::Ui) {
        ui.heading(crate::ui::theme::heading_text("Lyrics"));
        ui.add_space(12.0);
        ui.label(crate::ui::theme::body_text(
            "Lyrics editor coming in Phase 4.",
        ));
    }

    fn render_vocals(&mut self, ui: &mut egui::Ui) {
        ui.heading(crate::ui::theme::heading_text("Vocals"));
        ui.add_space(12.0);
        ui.label(crate::ui::theme::body_text(
            "Vocal production tools coming in Phase 4.",
        ));
    }

    fn render_theory(&mut self, ui: &mut egui::Ui) {
        ui.heading(crate::ui::theme::heading_text("Theory"));
        ui.add_space(12.0);
        ui.label(crate::ui::theme::body_text(
            "Music theory helper coming in Phase 3.",
        ));
    }

    fn render_analyzer(&mut self, ui: &mut egui::Ui) {
        let spectrum = self
            .shared_state
            .spectrum
            .read()
            .map(|g| g.clone())
            .unwrap_or_else(|_| vec![0.0; 128]);

        crate::ui::analyzer::spectrum_panel::spectrum_panel(ui, &spectrum);
    }

    fn render_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading(crate::ui::theme::heading_text("Settings"));
        ui.add_space(12.0);
        ui.label(crate::ui::theme::body_text(
            "Plugin settings coming in Phase 1 completion.",
        ));
    }

    fn render_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(28.0)
            .frame(
                egui::Frame::new()
                    .fill(crate::ui::theme::SURFACE_SECONDARY)
                    .stroke(egui::Stroke::new(1.0, crate::ui::theme::BORDER)),
            )
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.add_space(12.0);

                    let bpm = self.shared_state.daw_bpm.load(Ordering::Relaxed);
                    let detected = self.shared_state.detected_bpm.load(Ordering::Relaxed);
                    let playing = self.shared_state.is_playing.load(Ordering::Relaxed);

                    let project = self
                        .shared_state
                        .project_name
                        .read()
                        .map(|g| g.clone())
                        .unwrap_or_else(|_| "Untitled".to_string());

                    ui.label(crate::ui::theme::mono_text(&format!(
                        "Project: {}",
                        project
                    )));
                    ui.add_space(16.0);

                    if playing {
                        ui.label(crate::ui::theme::mono_text(&format!("DAW BPM: {}", bpm)));
                    } else {
                        ui.label(crate::ui::theme::mono_text("DAW: Stopped"));
                    }

                    if detected > 0 {
                        ui.add_space(16.0);
                        ui.label(crate::ui::theme::mono_text(&format!(
                            "Detected BPM: {}",
                            detected
                        )));
                    }

                    ui.add_space(16.0);
                    ui.label(crate::ui::theme::mono_text(&format!(
                        "Status: {}",
                        if playing { "Playing" } else { "Ready" }
                    )));
                });
            });
    }
}
