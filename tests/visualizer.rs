use mutti::visualizer::Visualizer;
use std::f32::consts::PI;
use std::time::Duration;

const SAMPLE_RATE: u32 = 44_100;

fn sine_wave(freq: f32, seconds: f32, sample_rate: u32) -> Vec<f32> {
    let n = (seconds * sample_rate as f32) as usize;
    (0..n)
        .map(|i| (2.0 * PI * freq * i as f32 / sample_rate as f32).sin())
        .collect()
}

#[test]
fn returns_requested_number_of_bars() {
    let mut v = Visualizer::new();
    let samples = sine_wave(440.0, 1.0, SAMPLE_RATE);
    let bars = v.compute(&samples, SAMPLE_RATE, Duration::ZERO, 32);
    assert_eq!(bars.len(), 32);
}

#[test]
fn empty_samples_returns_zeroed_bars() {
    let mut v = Visualizer::new();
    let bars = v.compute(&[], SAMPLE_RATE, Duration::ZERO, 16);
    assert_eq!(bars, vec![0; 16]);
}

#[test]
fn zero_num_bars_returns_empty() {
    let mut v = Visualizer::new();
    let samples = sine_wave(440.0, 1.0, SAMPLE_RATE);
    let bars = v.compute(&samples, SAMPLE_RATE, Duration::ZERO, 0);
    assert!(bars.is_empty());
}

#[test]
fn zero_sample_rate_returns_zeroed_bars() {
    let mut v = Visualizer::new();
    let samples = sine_wave(440.0, 1.0, SAMPLE_RATE);
    let bars = v.compute(&samples, 0, Duration::ZERO, 8);
    assert_eq!(bars, vec![0; 8]);
}

#[test]
fn position_past_end_returns_zeroed_bars() {
    let mut v = Visualizer::new();
    let samples = sine_wave(440.0, 0.1, SAMPLE_RATE);
    // Ask for a position well beyond the available audio.
    let bars = v.compute(&samples, SAMPLE_RATE, Duration::from_secs(10), 8);
    assert_eq!(bars, vec![0; 8]);
}

#[test]
fn silence_produces_zero_energy() {
    let mut v = Visualizer::new();
    let samples = vec![0.0f32; SAMPLE_RATE as usize];
    let bars = v.compute(&samples, SAMPLE_RATE, Duration::ZERO, 16);
    // dB floor should clamp silence to exactly 0.
    assert!(bars.iter().all(|&b| b == 0), "expected all zeros, got {bars:?}");
}

#[test]
fn loud_sine_produces_some_nonzero_bar() {
    let mut v = Visualizer::new();
    // Full-scale 1 kHz sine — should produce visible energy somewhere.
    let samples = sine_wave(1000.0, 1.0, SAMPLE_RATE);
    let bars = v.compute(&samples, SAMPLE_RATE, Duration::ZERO, 32);
    assert!(
        bars.iter().any(|&b| b > 0),
        "expected at least one bar > 0 for a 1kHz full-scale sine, got {bars:?}"
    );
}

#[test]
fn bars_stay_within_db_mapping_range() {
    let mut v = Visualizer::new();
    let samples = sine_wave(1000.0, 1.0, SAMPLE_RATE);
    let bars = v.compute(&samples, SAMPLE_RATE, Duration::ZERO, 32);
    // The compute() mapping clamps dB to a 45-unit range scaled to 200.
    assert!(
        bars.iter().all(|&b| b <= 200),
        "bars exceeded documented ceiling: {bars:?}"
    );
}

#[test]
fn higher_frequency_energy_lands_in_higher_bars() {
    let mut v = Visualizer::new();
    let num_bars = 32;

    // Low-frequency tone: energy should sit in the lower half of the bars.
    let low = sine_wave(120.0, 1.0, SAMPLE_RATE);
    let low_bars = v.compute(&low, SAMPLE_RATE, Duration::ZERO, num_bars);

    // High-frequency tone: energy should sit in the upper half.
    let high = sine_wave(8000.0, 1.0, SAMPLE_RATE);
    let high_bars = v.compute(&high, SAMPLE_RATE, Duration::ZERO, num_bars);

    let half = num_bars / 2;
    let low_below: u64 = low_bars[..half].iter().sum();
    let low_above: u64 = low_bars[half..].iter().sum();
    let high_below: u64 = high_bars[..half].iter().sum();
    let high_above: u64 = high_bars[half..].iter().sum();

    assert!(
        low_below > low_above,
        "120Hz tone should dominate lower bars: low_bars={low_bars:?}"
    );
    assert!(
        high_above > high_below,
        "8kHz tone should dominate upper bars: high_bars={high_bars:?}"
    );
}
