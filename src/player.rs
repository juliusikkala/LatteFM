use crate::tune::Tune;
use crate::channel::{Command, note_names};

pub struct ChannelPlayer {
    command_index: usize,
    instrument_index: usize,
    note_frames_done: i32,
    note_frames_left: i32,
    carrier_step: i32,
    carrier_phase: i32,
    modulator_step: i32,
    modulator_phase: i32,
}

impl Default for ChannelPlayer {
    fn default() -> ChannelPlayer {
        ChannelPlayer {
            command_index: 0,
            instrument_index: 0,
            note_frames_done: 0,
            note_frames_left: 0,
            carrier_step: 0,
            carrier_phase: 0,
            modulator_step: 0,
            modulator_phase: 0
        }
    }
}

// Note frequency lookup table, contains C8-B8. These are the frequencies
// multiplied by 65536. The semitone index of that C8 is 95, B8 is 107.
const NOTE_FREQ_LOOKUP: [i32; 12] = [
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

impl ChannelPlayer {
    fn generate(
        &mut self,
        tune: &Tune,
        command_stream: &[Command],
        out: &mut [i8]
    ) {
        let mut frames_left: usize = out.len();
        let mut start_frame: usize = 0;

        while frames_left > 0 {
            let step_frames = if (frames_left as i32) < self.note_frames_left {
                frames_left as i32
            } else {
                self.note_frames_left
            };

            if self.instrument_index < tune.instruments.len() {
                tune.instruments[self.instrument_index].generate(
                    self.note_frames_done,
                    self.note_frames_left,
                    self.carrier_step,
                    &mut self.carrier_phase,
                    self.modulator_step,
                    &mut self.modulator_phase,
                    &mut out[start_frame..(start_frame+(step_frames as usize))]
                );
            }

            start_frame += step_frames as usize;
            frames_left -= step_frames as usize;
            self.note_frames_done += step_frames;
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
                    // TODO: Make this faster somehow if needed?
                    let max_note = note_names::B8 as i32;
                    if (pitch as i32) <= max_note {
                        let octave = (max_note - (pitch as i32))/12;
                        let lookup_index = octave * 12 + (pitch as i32) - max_note + 11;
                        let steps = NOTE_FREQ_LOOKUP[lookup_index as usize] >> octave;
                        self.carrier_step = (steps-(tune.samplerate+1)/2)/tune.samplerate+1;
                        // TODO: Depends on instrument; use modulator_mul,
                        // modulator_div
                        // self.modulator_step =
                    } else {
                        // Dumbest way ever for marking pauses...
                        self.carrier_step = 0;
                        self.modulator_step = 0;
                    }
                },
                Command::SetInstrument(index) => {
                    self.instrument_index = index as usize;
                },
                Command::Beat(beats) => {
                    self.note_frames_left = tune.beat_length * beats as i32;
                    self.note_frames_done = 0;
                    self.carrier_phase = self.carrier_phase&0xFFFF;
                    self.modulator_phase = self.modulator_phase&0xFFFF;
                    break;
                },
                Command::Jump(index) => self.command_index = index as usize,
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
