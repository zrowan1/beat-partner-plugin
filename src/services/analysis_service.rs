use crate::audio::ring_buffer::LockFreeRingBuffer;
use crate::audio::spectrum::SpectrumCalculator;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Background analysis service that polls the ring buffer,
/// performs FFT, and writes smoothed spectrum bars to shared state.
pub struct AnalysisService {
    shutdown: Arc<AtomicBool>,
    _handle: JoinHandle<()>,
}

impl AnalysisService {
    pub fn new(
        ring_buffer: Arc<LockFreeRingBuffer>,
        spectrum_output: Arc<RwLock<Vec<f32>>>,
        sample_rate: f64,
    ) -> Self {
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();

        let handle = thread::spawn(move || {
            const FFT_SIZE: usize = 2048;
            const NUM_BARS: usize = 128;
            const SMOOTHING: f32 = 0.3;

            let mut calculator = SpectrumCalculator::new(FFT_SIZE);
            let mut fft_buffer = vec![0.0f32; FFT_SIZE];
            let mut smoothed = vec![0.0f32; NUM_BARS];

            while !shutdown_clone.load(Ordering::Relaxed) {
                let available = ring_buffer.available();

                if available >= FFT_SIZE {
                    let read = ring_buffer.read(&mut fft_buffer);
                    if read >= FFT_SIZE {
                        let spectrum = calculator.process(&fft_buffer[..FFT_SIZE]);
                        let spectrum_vec = spectrum.to_vec();
                        let bars =
                            calculator.bin_to_bars(NUM_BARS, sample_rate as f32, &spectrum_vec);
                        for i in 0..NUM_BARS {
                            smoothed[i] = SMOOTHING * smoothed[i] + (1.0 - SMOOTHING) * bars[i];
                        }

                        if let Ok(mut guard) = spectrum_output.write() {
                            if guard.len() != NUM_BARS {
                                guard.resize(NUM_BARS, 0.0);
                            }
                            guard.copy_from_slice(&smoothed);
                        }
                    }
                }

                thread::sleep(Duration::from_millis(16)); // ~60 Hz poll
            }
        });

        Self {
            shutdown,
            _handle: handle,
        }
    }
}

impl Drop for AnalysisService {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}
