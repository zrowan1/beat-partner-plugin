use nih_plug::params::EnumParam;
use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Enum, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize, Default)]
pub enum AnalysisMode {
    #[default]
    #[id = "off"]
    Off,
    #[id = "bpm"]
    Bpm,
    #[id = "key"]
    Key,
    #[id = "both"]
    Both,
}

#[derive(Enum, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize, Default)]
pub enum BpmSource {
    #[default]
    #[id = "auto"]
    AutoDetect,
    #[id = "daw"]
    DawTransport,
}

#[derive(Params)]
pub struct BeatPartnerParams {
    pub editor_state: Arc<EguiState>,

    #[id = "analysis_mode"]
    pub analysis_mode: EnumParam<AnalysisMode>,

    #[id = "spectrum_smoothing"]
    pub spectrum_smoothing: FloatParam,

    #[id = "bpm_source"]
    pub bpm_source: EnumParam<BpmSource>,

    #[id = "show_spectrum"]
    pub show_spectrum: BoolParam,
}

impl Default for BeatPartnerParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(900, 600),

            analysis_mode: EnumParam::new("Analysis Mode", AnalysisMode::Off),

            spectrum_smoothing: FloatParam::new(
                "Spectrum Smoothing",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            bpm_source: EnumParam::new("BPM Source", BpmSource::AutoDetect),

            show_spectrum: BoolParam::new("Show Spectrum", true),
        }
    }
}
