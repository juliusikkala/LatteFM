// Intermediate representations for module data. Similar to LatteFM structures,
// but less restrictive. This allows the initial unoptimized intermediate format
// to be incompatible, which at least gives an opportunity for optimization to
// solve those issues. It is also independent of samplerate and any fixed-point
// shenanigans.
use lattefm::instrument::Waveform;

#[derive(Clone)]
pub struct Instrument {
    carrier_waveform: Waveform,
    amplitude: f64,
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,
    modulator_waveform: Waveform,
    modulator_amplitude: f64,
    modulator_mul: f64,
    modulator_phase: f64,
    semitone_offset: f64 // Used to correct sample pitches
}

impl Default for Instrument {
    fn default() -> Instrument {
        Instrument {
            carrier_waveform: Waveform::Sine,
            amplitude: 0.0,
            attack: 0.0,
            decay: 0.0,
            sustain: 0.0,
            release: 0.0,
            modulator_waveform: Waveform::Sine,
            modulator_amplitude: 0.0,
            modulator_mul: 0.0,
            modulator_phase: 0.0,
            semitone_offset: 0.0,
        }
    }
}


impl Instrument {
    pub fn fit_adsr(
        &mut self,
        envelope_points: &Vec<(f64, f64)>,
        sustain_point_index: i64
    ) {
    }

    pub fn fit_to_sample(
        &mut self,
        sample_data: &Vec<f64>,
        semitone_offset: f64
    ) {
    }
}

pub const PAUSE: u32 = 255;

pub enum Command {
    Note(u32),
    SetInstrument(u32),
    Play(u32),
    Jump(u32),
    Repeat(u32),
    Pan(i8),
}

pub struct Module {
    pub tick_length: f64, // in seconds
    pub instruments: Vec<Instrument>,
    pub channels: Vec<Vec<Command>>
}

impl Module {
    pub fn optimize(self) -> Self {
        self
    }

    pub fn print_as_source(&self) {
        println!("TODO");
    }
}
