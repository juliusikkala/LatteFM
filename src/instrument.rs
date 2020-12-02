use crate::wave::{i16cos, i16square, i16triangle, i16saw, i16noise};
use crate::channel::note_names;

#[repr(usize)]
#[derive(Clone, Copy)]
pub enum Waveform {
    Sine = 0, Square, Triangle, Saw, Noise
}

macro_rules! oscillator {
    (Waveform::Sine, $t:expr) => {i16cos($t)};
    (Waveform::Square, $t:expr) => {i16square($t)};
    (Waveform::Triangle, $t:expr) => {i16triangle($t)};
    (Waveform::Saw, $t:expr) => {i16saw($t)};
    (Waveform::Noise, $t:expr) => {i16noise($t)};
}

pub struct Instrument {
    pub carrier_waveform: Waveform,
    pub amplitude: u16,
    pub attack: u16,
    pub decay: u16,
    pub sustain: u16,
    pub release: u16,
    pub pan: u16,
    pub modulator_waveform: Waveform,
    pub modulator_amplitude: u16, // 16-bit fixed point
    pub modulator_mul: u16,
    pub modulator_div: u16,
    pub modulator_phase: u16,
}

// Note frequency lookup table, contains C8-B8. These are the frequencies
// multiplied by 65536 and rounded to nearest integer.
const NOTE_FREQ_LOOKUP: [u32; 12] = [
    274334289,
    290647054,
    307929828,
    326240288,
    345639545,
    366192342,
    387967272,
    411037006,
    435478539,
    461373440,
    488808132,
    517874176
];

pub type Wavegen = fn(
    instr: &Instrument,
    note_frames_done: i32,
    note_frames_left: i32,
    carrier_step: i32,
    carrier_phase: &mut i32,
    modulator_step: i32,
    modulator_phase: &mut i32,
    out: &mut [i8]
);

pub fn dummy_wavegen(
    _instr: &Instrument,
    _note_frames_done: i32,
    _note_frames_left: i32,
    carrier_step: i32,
    carrier_phase: &mut i32,
    modulator_step: i32,
    modulator_phase: &mut i32,
    out: &mut [i8]
){
    for x in out.iter_mut() {
        *x += (i16cos(*carrier_phase as i16) >> 8) as i8;
        *carrier_phase += carrier_step;
        *modulator_phase += modulator_step;
    }
}

macro_rules! wavegen_template {
    ($carrier_waveform:ident, $modulator_waveform:ident) => {
        {
            fn local_wavegen(
                instr: &Instrument,
                _note_frames_done: i32,
                _note_frames_left: i32,
                carrier_step: i32,
                carrier_phase: &mut i32,
                modulator_step: i32,
                modulator_phase: &mut i32,
                out: &mut [i8]
            ){
                for x in out.iter_mut() {
                    let modulator = oscillator!(Waveform::$modulator_waveform, *modulator_phase as i16);
                    let local_phase = ((modulator as i32) * (instr.modulator_amplitude as i32)) >> 16;
                    *x += (oscillator!(Waveform::$carrier_waveform, (*carrier_phase + local_phase) as i16) >> 8) as i8;
                    *carrier_phase += carrier_step;
                    *modulator_phase += modulator_step;
                }
            }
            local_wavegen
        }
    };
}

macro_rules! wavegen_set {
    ($carrier_waveform:ident) => {[
        wavegen_template!($carrier_waveform, Sine),
        wavegen_template!($carrier_waveform, Square),
        wavegen_template!($carrier_waveform, Triangle),
        wavegen_template!($carrier_waveform, Saw),
        wavegen_template!($carrier_waveform, Noise),
    ]};
}

const WAVEGEN_TABLE: [[Wavegen; 5]; 5] = [
    wavegen_set!(Sine),
    wavegen_set!(Square),
    wavegen_set!(Triangle),
    wavegen_set!(Saw),
    wavegen_set!(Noise)
];

impl Instrument {
    pub fn get_wavegen(&self) -> Wavegen {
        // All this song and dance is just to avoid a couple of match statements
        // in the tight loop in wavegen :D
        WAVEGEN_TABLE[self.carrier_waveform as usize][self.modulator_waveform as usize]
    }

    pub fn get_timer_steps(
        &self,
        samplerate: i32,
        pitch: i32,
        carrier_step: &mut i32,
        modulator_step: &mut i32
    ) {
        let max_note = note_names::B8 as i32;
        if pitch <= max_note {
            let octave = (max_note - pitch)/12;
            let lookup_index = octave * 12 + pitch - max_note + 11;
            let carrier_base_steps = (NOTE_FREQ_LOOKUP[lookup_index as usize] >> octave) as i32;
            let modulator_base_steps = carrier_base_steps * (self.modulator_mul as i32) / (self.modulator_div as i32);
            // Essentially just divide by samplerate, but round to nearest too.
            *carrier_step = (carrier_base_steps-(samplerate+1)/2)/samplerate+1;
            *modulator_step = (modulator_base_steps-(samplerate+1)/2)/samplerate+1;
        } else {
            // Dumbest way ever for marking pauses...
            *carrier_step = 0;
            *modulator_step = 0;
        }
    }
}
