use crate::tune::Tune;
use crate::channel::Command;
use crate::instrument::{Wavegen, WAVEGEN_TABLE, ADSRState};

#[derive(Clone, Copy)]
pub struct ChannelPlayer {
    command_index: usize,
    instrument_index: usize,
    wavegen: Wavegen,
    note_frames_left: i32,
    carrier_step: i32,
    carrier_phase: i32,
    modulator_step: i32,
    modulator_phase: i32,
    pan: (i32, i32), // 8-bit fixed point
    amplitude: (i32, i32), // 24-bit fixed point (to avoid some rounding stupidity)
    adsr: ADSRState,
    repeat_counter: i32,
}

impl Default for ChannelPlayer {
    fn default() -> ChannelPlayer {
        ChannelPlayer {
            command_index: 0,
            instrument_index: 0,
            wavegen: WAVEGEN_TABLE[0][0],
            note_frames_left: 0,
            carrier_step: 0,
            carrier_phase: 0,
            modulator_step: 0,
            modulator_phase: 0,
            pan: (1<<8, 1<<8),
            amplitude: (0, 0),
            adsr: Default::default(),
            repeat_counter: 0
        }
    }
}

impl ChannelPlayer {
    fn generate(
        &mut self,
        tune: &Tune,
        command_stream: &[Command],
        out: &mut [i8]
    ) {
        let mut frames_left: i32 = (out.len()>>1) as i32;
        let mut start_frame: usize = 0;

        while frames_left > 0 {
            let step_frames = if frames_left < self.note_frames_left {
                frames_left
            } else {
                self.note_frames_left
            };

            let end_frame = start_frame+(step_frames as usize);
            if self.carrier_step != 0 {
                (self.wavegen)(
                    &tune.instruments[self.instrument_index],
                    &mut self.adsr,
                    &mut self.amplitude,
                    self.carrier_step,
                    &mut self.carrier_phase,
                    self.modulator_step,
                    &mut self.modulator_phase,
                    &mut out[start_frame*2..end_frame*2]
                );
            }

            start_frame += step_frames as usize;
            frames_left -= step_frames;
            self.note_frames_left -= step_frames;

            if self.note_frames_left <= 0 {
                self.execute(tune, command_stream);
            }
        }
    }

    fn execute(
        &mut self,
        tune: &Tune,
        command_stream: &[Command]
    ) {
        loop {
            let cur_command_index = self.command_index;
            self.command_index += 1;
            match command_stream[cur_command_index] {
                Command::Note(pitch) => {
                    tune.instruments[self.instrument_index].get_timer_steps(
                        tune.samplerate,
                        pitch as i32,
                        &mut self.carrier_step,
                        &mut self.modulator_step
                    );
                },
                Command::SetInstrument(index) => {
                    self.instrument_index = index as usize;
                    self.wavegen = tune.instruments[self.instrument_index].get_wavegen();
                },
                Command::Beat(beats) => {
                    self.note_frames_left = tune.beat_length * beats as i32;
                    self.amplitude = (0, 0);
                    let instrument = &tune.instruments[self.instrument_index]; 
                    self.adsr = instrument.get_adsr(tune.samplerate, self.note_frames_left, self.pan);
                    self.adsr.init_stage_amplitude(&mut self.amplitude);

                    // We can only reset phase if the initial amplitude is zero.
                    if self.amplitude == (0, 0) {
                        self.carrier_phase = 0;
                        self.modulator_phase = instrument.modulator_phase as i32;
                    } else {
                        // Otherwise, we have to continue where we left off to
                        // avoid clicks in the sound. This messes up
                        // carrier-modulator synchronization.
                        self.carrier_phase = self.carrier_phase&0xFFFF;
                        self.modulator_phase = self.modulator_phase&0xFFFF;
                    }
                    break;
                },
                Command::Jump(index) => self.command_index = index as usize,
                Command::Repeat(count) => {
                    if self.repeat_counter == 0 {
                        self.repeat_counter = count as i32;
                    } else {
                        self.repeat_counter -= 1;
                        if self.repeat_counter == 0 {
                            self.command_index += 1;
                        }
                    }
                },
                Command::Pan(pan) => {
                    if pan <= 0 {
                        self.pan.0 = 1<<8;
                        self.pan.1 = (1<<8) + (pan as i32)*2;
                    } else {
                        self.pan.0 = (1<<8) - (pan as i32)*2 - 2;
                        self.pan.1 = 1<<8;
                    }
                }
            }
        }
    }
}

pub struct Player<'a> {
    pub tune: &'static Tune,
    pub channels: &'a mut [ChannelPlayer]
}

impl<'a> Player<'a> {
    pub fn new(
        tune: &'static Tune,
        channels: &'a mut [ChannelPlayer]
    ) -> Self {
        Self {
            tune,
            channels
        }
    }

    pub fn generate(&mut self, out: &mut[i8]) {
        for x in out.iter_mut() {
            *x = 0;
        }
        for i in 0..self.tune.channels.len() {
            self.channels[i].generate(self.tune, self.tune.channels[i], out);
        }
    }
}
