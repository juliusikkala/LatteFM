use crate::wave::{i16cos, i16square, i16triangle, i16saw, i16noise};
use crate::channel::note_names;

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(Clone, Copy)]
pub struct ADSRStep {
    pub frames_left: i32,
    pub amplitude_step: (i32, i32),
}

#[derive(Clone, Copy)]
pub struct ADSRState {
    pub cur_stage: usize,
    pub stages: [ADSRStep; 4]
}

impl Default for ADSRState {
    fn default() -> ADSRState {
        ADSRState {
            cur_stage: 0,
            stages: [ADSRStep {frames_left: 0, amplitude_step: (0, 0)}; 4]
        }
    }
}

impl ADSRState {
    pub fn init_stage_amplitude(&mut self, amplitude: &mut (i32, i32)) {
        while self.cur_stage < self.stages.len() && self.stages[self.cur_stage].frames_left == 0 {
            amplitude.0 += self.stages[self.cur_stage].amplitude_step.0;
            amplitude.1 += self.stages[self.cur_stage].amplitude_step.1;
            self.cur_stage += 1;
        }
    }
}

pub struct Instrument {
    pub carrier_waveform: Waveform,
    pub amplitude: u16, // 16-bit fixed point
    pub attack: u16,  // 12-bit fixed point, in seconds.
    pub decay: u16,   // 12-bit fixed point, in seconds.
    pub sustain: u16, // 16-bit fixed point (sustain amplitude)
    pub release: u16, // 12-bit fixed point, in seconds
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
    adsr: &mut ADSRState,
    amplitude: &mut (i32, i32),
    carrier_step: i32,
    carrier_phase: &mut i32,
    modulator_step: i32,
    modulator_phase: &mut i32,
    out: &mut [i8]
);

macro_rules! wavegen_template {
    ($carrier_waveform:ident, $modulator_waveform:ident) => {
        {
            fn local_wavegen(
                instr: &Instrument,
                adsr: &mut ADSRState,
                amplitude: &mut (i32, i32),
                carrier_step: i32,
                carrier_phase: &mut i32,
                modulator_step: i32,
                modulator_phase: &mut i32,
                out: &mut [i8]
            ){
                let mut frames_left: i32 = (out.len()>>1) as i32;
                let mut start_frame: usize = 0;

                while frames_left > 0 {
                    let stage = &mut adsr.stages[adsr.cur_stage];
                    let step_frames = if frames_left < stage.frames_left {
                        frames_left
                    } else {
                        stage.frames_left
                    };

                    let end_frame = start_frame+(step_frames as usize);
                    for i in start_frame..end_frame {
                        let modulator = oscillator!(Waveform::$modulator_waveform, *modulator_phase as i16);
                        let mod_value = ((modulator as i32) * (instr.modulator_amplitude as i32)) >> 20; // 12-bit fixed point
                        let carrier = oscillator!(Waveform::$carrier_waveform, *carrier_phase as i16) as i32;
                        out[i*2] += ((carrier*(amplitude.0 >> 9)) >> 23) as i8;
                        out[i*2+1] += ((carrier*(amplitude.1 >> 9)) >> 23) as i8;
                        *carrier_phase += carrier_step * (mod_value + (1<<11)) >> 11;
                        *modulator_phase += modulator_step;
                        amplitude.0 += stage.amplitude_step.0;
                        amplitude.1 += stage.amplitude_step.1;
                    }

                    start_frame = end_frame;
                    frames_left -= step_frames;
                    stage.frames_left -= step_frames;
                    if(stage.frames_left == 0) {
                        adsr.cur_stage += 1;
                        adsr.init_stage_amplitude(amplitude);
                    }
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

pub const WAVEGEN_TABLE: [[Wavegen; 5]; 5] = [
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

    pub fn get_adsr(&self, samplerate: i32, length: i32, pan: (i32, i32)) -> ADSRState {
        let mut adsr: ADSRState = Default::default();
        adsr.cur_stage = 0;

        // Attack
        adsr.stages[0].frames_left = (self.attack as i32)*samplerate >> 12;
        adsr.stages[0].amplitude_step.0 = (self.amplitude as i32) * pan.0;
        adsr.stages[0].amplitude_step.1 = (self.amplitude as i32) * pan.1;
        if adsr.stages[0].frames_left > 0 {
            adsr.stages[0].amplitude_step.0 /= adsr.stages[0].frames_left;
            adsr.stages[0].amplitude_step.1 /= adsr.stages[0].frames_left;
        }

        // Decay
        adsr.stages[1].frames_left = (self.decay as i32)*samplerate >> 12;
        let decay_base_amplitude = (self.sustain as i32) - (self.amplitude as i32);
        adsr.stages[1].amplitude_step = (decay_base_amplitude * pan.0, decay_base_amplitude * pan.1);
        if adsr.stages[1].frames_left > 0 {
            adsr.stages[1].amplitude_step.0 /= adsr.stages[1].frames_left;
            adsr.stages[1].amplitude_step.1 /= adsr.stages[1].frames_left;
        }

        // Release
        let frames_so_far = adsr.stages[0].frames_left + adsr.stages[1].frames_left;
        let frames_left = length - frames_so_far;
        let intended_release = (self.release as i32)*samplerate >> 12;

        adsr.stages[3].frames_left = if frames_left < intended_release {frames_left} else {intended_release};
        let release_base_amplitude = -(self.sustain as i32);
        adsr.stages[3].amplitude_step = (release_base_amplitude * pan.0, release_base_amplitude * pan.1);
        if adsr.stages[3].frames_left > 0 {
            adsr.stages[3].amplitude_step.0 /= adsr.stages[3].frames_left;
            adsr.stages[3].amplitude_step.1 /= adsr.stages[3].frames_left;
        }

        // Sustain
        adsr.stages[2].frames_left = length - adsr.stages[0].frames_left - adsr.stages[1].frames_left - adsr.stages[3].frames_left;
        adsr.stages[2].amplitude_step = (0, 0);

        adsr
    }
}
