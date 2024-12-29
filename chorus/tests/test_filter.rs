use chorus::filter::{Filter, LowpassFilter, HighpassFilter, BandpassFilter, ResonantFilter};
use chorus::SAMPLE_RATE;
use std::f32::consts::PI;

fn compute_response_amplitude(filter: &mut impl Filter, frequency: f32) -> f32 {
    let f = 2.0*PI*frequency/SAMPLE_RATE as f32;
    let mut max_amplitude = 0.0;
    for i in 0..(2*SAMPLE_RATE) {
        let x = (f*i as f32).sin();
        let y = filter.process(x);
        if i > 1000 {
            max_amplitude = f32::max(max_amplitude, y.abs());
        }
    }
    max_amplitude
}

#[test]
fn test_lowpass() {
    let mut filter = LowpassFilter::new(2000.0);
    assert!(compute_response_amplitude(&mut filter, 1000.0) > 0.5);
    assert!(compute_response_amplitude(&mut filter, 4000.0) < 0.5);
}

#[test]
fn test_highpass() {
    let mut filter = HighpassFilter::new(2000.0);
    assert!(compute_response_amplitude(&mut filter, 1000.0) < 0.5);
    assert!(compute_response_amplitude(&mut filter, 4000.0) > 0.5);
}

#[test]
fn test_bandpass() {
    let mut filter = BandpassFilter::new(2000.0, 3000.0);
    let y1 = compute_response_amplitude(&mut filter, 500.0);
    let y2 = compute_response_amplitude(&mut filter, 2500.0);
    let y3 = compute_response_amplitude(&mut filter, 4000.0);
    assert!(y2 > y1);
    assert!(y2 > y3);
}

#[test]
fn test_resonant() {
    let mut filter = ResonantFilter::new(2000.0, 1000.0);
    let y1 = compute_response_amplitude(&mut filter, 500.0);
    let y2 = compute_response_amplitude(&mut filter, 2000.0);
    let y3 = compute_response_amplitude(&mut filter, 4000.0);
    assert!(y2 > y1);
    assert!(y2 > y3);
    assert!(y2 > 1.0);
}
