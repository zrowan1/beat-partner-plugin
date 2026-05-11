use nih_plug::prelude::*;
use std::sync::atomic::Ordering;
use std::sync::Arc;

mod audio;
mod editor;
mod error;
mod models;
mod params;
mod services;
mod ui;

use audio::ring_buffer::LockFreeRingBuffer;
use params::BeatPartnerParams;
use services::analysis_service::AnalysisService;
use ui::app::SharedState;

struct BeatPartner {
    params: Arc<BeatPartnerParams>,
    ring_buffer: Arc<LockFreeRingBuffer>,
    shared_state: Arc<SharedState>,
    analysis_service: Option<AnalysisService>,
}

impl Default for BeatPartner {
    fn default() -> Self {
        let ring_buffer = Arc::new(LockFreeRingBuffer::new(48_000 * 2));
        let shared_state = SharedState::new();

        Self {
            params: Arc::new(BeatPartnerParams::default()),
            ring_buffer,
            shared_state,
            analysis_service: None,
        }
    }
}

impl Plugin for BeatPartner {
    const NAME: &'static str = "BeatPartner";
    const VENDOR: &'static str = "BeatPartner";
    const URL: &'static str = "https://github.com/beatpartner/beat-partner-plugin";
    const EMAIL: &'static str = "";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.analysis_service = Some(AnalysisService::new(
            self.ring_buffer.clone(),
            self.shared_state.spectrum.clone(),
            buffer_config.sample_rate as f64,
        ));
        true
    }

    fn reset(&mut self) {
        self.ring_buffer.clear();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // 1. Read DAW transport info (primary BPM source)
        let transport = context.transport();
        if let Some(tempo) = transport.tempo {
            self.shared_state
                .daw_bpm
                .store(tempo as u32, Ordering::Relaxed);
        }
        self.shared_state
            .is_playing
            .store(transport.playing, Ordering::Relaxed);

        // 2. Write interleaved audio to the ring buffer for background analysis.
        //    We mix down to mono to keep the buffer size manageable.
        let num_samples = buffer.samples();
        let num_channels = buffer.channels();

        if num_channels > 0 && num_samples > 0 {
            const CHUNK_SIZE: usize = 512;
            let mut mono_chunk = [0.0f32; CHUNK_SIZE];

            let channels = buffer.as_slice_immutable();

            for chunk_start in (0..num_samples).step_by(CHUNK_SIZE) {
                let chunk_end = (chunk_start + CHUNK_SIZE).min(num_samples);
                let len = chunk_end - chunk_start;

                for s in 0..len {
                    let mut sum = 0.0f32;
                    for ch in 0..num_channels {
                        sum += channels[ch][chunk_start + s];
                    }
                    mono_chunk[s] = sum / num_channels as f32;
                }

                self.ring_buffer.write(&mono_chunk[..len]);
            }
        }

        ProcessStatus::Normal
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create_editor(self.params.clone(), self.shared_state.clone())
    }
}

impl Vst3Plugin for BeatPartner {
    const VST3_CLASS_ID: [u8; 16] = *b"BeatPartnerPlug_";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Analyzer, Vst3SubCategory::Tools];
}

nih_export_vst3!(BeatPartner);
