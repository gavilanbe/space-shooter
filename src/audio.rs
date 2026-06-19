use rodio::{OutputStream, Sink, Source};
use std::time::Duration;

struct SpaceMusic {
    sample_rate: u32,
    sample_idx: u64,
    notes: Vec<f64>,
    bass_notes: Vec<f64>,
    note_duration_samples: u64,
}

impl SpaceMusic {
    fn new(sample_rate: u32) -> Self {
        // Dark minor scale melody — cinematic space feel
        let notes: Vec<f64> = vec![
            // Phrase 1: mysterious intro
            329.63, 311.13, 293.66, 261.63, 293.66, 311.13, 329.63, 293.66,
            // Phrase 2: tension rising
            349.23, 329.63, 311.13, 349.23, 392.00, 349.23, 329.63, 311.13,
            // Phrase 3: action
            392.00, 440.00, 392.00, 349.23, 329.63, 349.23, 392.00, 440.00,
            // Phrase 4: resolve
            329.63, 293.66, 329.63, 349.23, 329.63, 293.66, 261.63, 293.66,
        ];

        let bass_notes: Vec<f64> = vec![
            130.81, 130.81, 146.83, 146.83, 164.81, 164.81, 130.81, 130.81,
            146.83, 146.83, 155.56, 155.56, 164.81, 164.81, 146.83, 146.83,
            164.81, 164.81, 174.61, 174.61, 196.00, 196.00, 174.61, 174.61,
            130.81, 130.81, 146.83, 146.83, 130.81, 130.81, 123.47, 123.47,
        ];

        let note_duration_samples = (sample_rate as f64 * 0.25) as u64;

        Self {
            sample_rate,
            sample_idx: 0,
            notes,
            bass_notes,
            note_duration_samples,
        }
    }
}

impl Source for SpaceMusic {
    fn current_frame_len(&self) -> Option<usize> { None }
    fn channels(&self) -> u16 { 1 }
    fn sample_rate(&self) -> u32 { self.sample_rate }
    fn total_duration(&self) -> Option<Duration> { None }
}

impl Iterator for SpaceMusic {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let total = self.note_duration_samples * self.notes.len() as u64;
        let pos = self.sample_idx % total;
        let note_idx = (pos / self.note_duration_samples) as usize;
        let bass_idx = note_idx % self.bass_notes.len();

        let freq = self.notes[note_idx];
        let bass_freq = self.bass_notes[bass_idx];

        let t = self.sample_idx as f64 / self.sample_rate as f64;

        // Lead: saw-ish wave (sin + harmonics)
        let lead = (std::f64::consts::TAU * freq * t).sin() * 0.4
            + (std::f64::consts::TAU * freq * 2.0 * t).sin() * 0.15
            + (std::f64::consts::TAU * freq * 3.0 * t).sin() * 0.05;

        // Bass: deep sine
        let bass = (std::f64::consts::TAU * bass_freq * t).sin() * 0.3;

        // Pad: very soft detuned layer
        let pad = (std::f64::consts::TAU * freq * 0.5 * t).sin() * 0.1
            + (std::f64::consts::TAU * freq * 0.501 * t).sin() * 0.1;

        // Note envelope
        let note_pos = (pos % self.note_duration_samples) as f64
            / self.note_duration_samples as f64;
        let envelope = if note_pos < 0.03 {
            note_pos / 0.03
        } else if note_pos > 0.7 {
            (1.0 - note_pos) / 0.3
        } else {
            1.0
        };

        let sample = (lead + bass + pad) * envelope * 0.12;

        self.sample_idx += 1;
        Some(sample as f32)
    }
}

pub struct Audio {
    _stream: Option<OutputStream>,
    _sink: Option<Sink>,
}

impl Audio {
    pub fn new() -> Self {
        match Self::init() {
            Ok((s, k)) => Self { _stream: Some(s), _sink: Some(k) },
            Err(_) => Self { _stream: None, _sink: None },
        }
    }

    fn init() -> Result<(OutputStream, Sink), Box<dyn std::error::Error>> {
        let (stream, handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&handle)?;
        sink.append(SpaceMusic::new(44100));
        sink.set_volume(0.5);
        Ok((stream, sink))
    }
}
