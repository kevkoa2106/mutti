use rustfft::{FftPlanner, num_complex::Complex};
use std::time::Duration;

const FFT_SIZE: usize = 2048;

pub struct Visualizer {
    planner: FftPlanner<f32>,
    window: Vec<f32>,
}

impl Visualizer {
    pub fn new() -> Self {
        // Hann window
        let window = (0..FFT_SIZE)
            .map(|i| {
                let x = (i as f32) / (FFT_SIZE as f32 - 1.0);
                0.5 - 0.5 * (2.0 * std::f32::consts::PI * x).cos()
            })
            .collect();
        Self {
            planner: FftPlanner::new(),
            window,
        }
    }

    /// Compute a magnitude spectrum from `samples` (mono) at the playback `position`,
    /// grouped into `num_bars` log-spaced frequency bins.
    pub fn compute(
        &mut self,
        samples: &[f32],
        sample_rate: u32,
        position: Duration,
        num_bars: usize,
    ) -> Vec<u64> {
        if samples.is_empty() || num_bars == 0 || sample_rate == 0 {
            return vec![0; num_bars];
        }

        let start = (position.as_secs_f64() * sample_rate as f64) as usize;
        if start + FFT_SIZE >= samples.len() {
            return vec![0; num_bars];
        }

        let mut buf: Vec<Complex<f32>> = samples[start..start + FFT_SIZE]
            .iter()
            .zip(self.window.iter())
            .map(|(s, w)| Complex { re: s * w, im: 0.0 })
            .collect();

        let fft = self.planner.plan_fft_forward(FFT_SIZE);
        fft.process(&mut buf);

        // Use first half (real spectrum) and skip DC bin
        let bins: Vec<f32> = buf[1..FFT_SIZE / 2]
            .iter()
            .map(|c| (c.re * c.re + c.im * c.im).sqrt())
            .collect();

        // Log-spaced frequency bands from ~40Hz to nyquist
        let nyquist = sample_rate as f32 / 2.0;
        let bin_hz = nyquist / bins.len() as f32;
        let f_min: f32 = 40.0;
        let f_max: f32 = nyquist;

        let mut out = Vec::with_capacity(num_bars);
        for i in 0..num_bars {
            let lo_f = f_min * (f_max / f_min).powf(i as f32 / num_bars as f32);
            let hi_f = f_min * (f_max / f_min).powf((i + 1) as f32 / num_bars as f32);
            let lo = ((lo_f / bin_hz) as usize).max(0);
            let hi = ((hi_f / bin_hz) as usize).min(bins.len()).max(lo + 1);

            let slice = &bins[lo..hi];
            // Peak (not avg) so transient bass hits aren't smeared away.
            let peak = slice.iter().copied().fold(0.0f32, f32::max);

            // Mild bass emphasis so kicks dominate without flooring everything else.
            let bass_boost = 1.0 + 0.8 * (1.0 - i as f32 / num_bars as f32).powi(2);
            let mag = peak * bass_boost;

            // Map dB range [+10, +55] → [0, 200]. Typical music sits in the
            // middle; only real beats/hits push the bars to the top.
            let db = 20.0 * (mag + 1e-6).log10();
            let v = (((db - 10.0).clamp(0.0, 45.0)) * (200.0 / 45.0)) as u64;
            out.push(v);
        }
        out
    }
}
