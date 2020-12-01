use crate::wave::i16cos;

pub enum Waveform {
    Sine, Square, Triangle, Saw, Noise
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
    pub modulator_amplitude: u16,
    pub modulator_mul: u16,
    pub modulator_div: u16,
    pub modulator_phase: u16,
}

impl Instrument {
    pub fn generate(
        &self,
        note_frames_done: i32,
        note_frames_left: i32,
        carrier_step: i32,
        carrier_phase: &mut i32,
        modulator_step: i32,
        modulator_phase: &mut i32,
        out: &mut [i8]
    ) {
        for x in out.iter_mut() {
            *x += (i16cos(*carrier_phase as i16) >> 8) as i8;
            *carrier_phase += carrier_step;
        }
    }
}
