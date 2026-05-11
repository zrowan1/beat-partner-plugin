use realfft::RealFftPlanner;
use rustfft::num_complex::Complex;

/// Pre-allocated FFT-based spectrum calculator.
pub struct SpectrumCalculator {
    fft_size: usize,
    window: Vec<f32>,
    input_buffer: Vec<f32>,
    output_buffer: Vec<Complex<f32>>,
}

impl SpectrumCalculator {
    pub fn new(fft_size: usize) -> Self {
        Self {
            fft_size,
            window: hann_window(fft_size),
            input_buffer: vec![0.0; fft_size],
            output_buffer: vec![Complex::default(); fft_size / 2 + 1],
        }
    }

    /// Compute magnitude spectrum from mono time-domain samples.
    /// Returns a slice of magnitude values for bins 0..fft_size/2.
    pub fn process(&mut self, samples: &[f32]) -> &[f32] {
        let len = samples.len().min(self.fft_size);
        self.input_buffer[..len].copy_from_slice(&samples[..len]);
        self.input_buffer[len..].fill(0.0);

        for i in 0..self.fft_size {
            self.input_buffer[i] *= self.window[i];
        }

        let mut planner = RealFftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(self.fft_size);
        fft.process(&mut self.input_buffer, &mut self.output_buffer)
            .unwrap();

        // Return magnitude of the first fft_size/2 bins (excluding Nyquist)
        // We reuse input_buffer as the magnitude output to avoid an extra allocation.
        for i in 0..(self.fft_size / 2) {
            self.input_buffer[i] = self.output_buffer[i].norm();
        }

        &self.input_buffer[..self.fft_size / 2]
    }

    /// Map the raw FFT magnitude spectrum into a smaller number of logarithmic bars.
    pub fn bin_to_bars(&self, num_bars: usize, sample_rate: f32, spectrum: &[f32]) -> Vec<f32> {
        let nyquist = sample_rate / 2.0;
        let bin_count = spectrum.len();
        let bin_freq = nyquist / bin_count as f32;

        let mut bars = vec![0.0f32; num_bars];

        for bar in 0..num_bars {
            // Logarithmic frequency mapping (quadratic for more bass resolution)
            let t_low = bar as f32 / num_bars as f32;
            let t_high = (bar + 1) as f32 / num_bars as f32;
            let f_low = nyquist * t_low.powf(2.0);
            let f_high = nyquist * t_high.powf(2.0);

            let bin_low = (f_low / bin_freq) as usize;
            let bin_high = ((f_high / bin_freq).ceil() as usize).min(bin_count);

            let mut max_val = 0.0f32;
            for bin in bin_low..bin_high {
                max_val = max_val.max(spectrum[bin]);
            }

            // Rough calibration: scale so typical audio peaks around 0.7-1.0
            bars[bar] = (max_val / 50.0).min(1.0);
        }

        bars
    }
}

fn hann_window(size: usize) -> Vec<f32> {
    (0..size)
        .map(|i| {
            let phase = 2.0 * std::f32::consts::PI * i as f32 / (size - 1).max(1) as f32;
            0.5 * (1.0 - phase.cos())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spectrum_dc() {
        let mut calc = SpectrumCalculator::new(256);
        // Constant signal -> energy in DC bin
        let samples = vec![1.0f32; 256];
        let spectrum = calc.process(&samples);
        assert!(spectrum[0] > 0.0);
    }

    #[test]
    fn test_bin_to_bars() {
        let calc = SpectrumCalculator::new(256);
        let spectrum = vec![1.0f32; 128];
        let bars = calc.bin_to_bars(32, 48000.0, &spectrum);
        assert_eq!(bars.len(), 32);
        assert!(bars.iter().all(|&v| v > 0.0));
    }
}
