// Intermediate representations for module data. Similar to LatteFM structures,
// but less restrictive. This allows the initial unoptimized intermediate format
// to be incompatible, which at least gives an opportunity for optimization to
// solve those issues. It is also independent of samplerate and any fixed-point
// shenanigans.
use lattefm::instrument::Waveform;
use std::collections::HashMap;
use std::cmp::Ordering;

#[derive(Clone, PartialEq)]
pub struct Instrument {
    carrier_waveform: Waveform,
    amplitude: f64,
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,
    modulator_waveform: Waveform,
    modulator_amplitude: f64,
    modulator_mul: i64,
    modulator_div: i64,
    modulator_phase: f64,
    semitone_offset: f64, // Used to correct sample pitches
    release_is_sustain: bool
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
            modulator_mul: 1,
            modulator_div: 1,
            modulator_phase: 0.0,
            semitone_offset: 0.0,
            release_is_sustain: false
        }
    }
}

impl Instrument {
    pub fn fit_adsr(
        &mut self,
        envelope_points: &Vec<(f64, f64)>,
        sustain_point_index: i64,
        looping: bool
    ) {
        if envelope_points.len() != 0 {
            let local_sustain_point_index = if sustain_point_index < 0 {
                envelope_points.len() - 1
            } else {
                sustain_point_index as usize
            };
            let (peak_t, peak_vol) = envelope_points[0..(local_sustain_point_index+1)].iter().fold(
                (0.0, 0.0),
                |(at, avol), (bt, bvol)| {
                    // Wow, just wow. WTF were they smoking. Who the hell thought
                    // this was a good idea??? "Opinionated" tools were a mistake
                    // because the people in charge always have moronic opinions
                    // fixated on edge cases at the cost of making common cases much
                    // harder.
                    match avol.partial_cmp(bvol) {
                        None => panic!("This cannot happen."),
                        Some(Ordering::Less) => (*bt, *bvol),
                        Some(Ordering::Greater) => (at, avol),
                        Some(Ordering::Equal) => {
                            match at.partial_cmp(bt) {
                                None => panic!("This cannot happen."),
                                Some(Ordering::Less) => (at, avol),
                                Some(_) => (*bt, *bvol),
                            }
                        }
                    }
                }
            );
            let (sustain_t, sustain_vol) = envelope_points[local_sustain_point_index];
            self.amplitude = peak_vol;
            self.attack = peak_t;
            self.decay = sustain_t - peak_t;
            self.sustain = sustain_vol;
            self.release = envelope_points.last().unwrap().0 - sustain_t;
            self.release_is_sustain = looping;
        } else {
            self.amplitude = 1.0;
            self.attack = 0.0;
            self.decay = 0.0;
            self.sustain = 1.0;
            self.release = 0.0;
            self.release_is_sustain = false;
        }
    }

    pub fn fit_to_sample(
        &mut self,
        sample_data: &Vec<f64>,
        semitone_offset: f64
    ) {
        // TODO: Determine base frequency from sample_data
        //self.semitone_offset = semitone_offset;

        // Determine max amplitude from sample.
        let max_amplitude = sample_data.iter().fold(0.0, |a, b| f64::max(a, f64::abs(*b)));
        self.amplitude *= max_amplitude;
        self.sustain *= max_amplitude;
    }
}

pub const PAUSE: u32 = 255;
// Release may become PAUSE during processing, this depends on the instrument.
pub const RELEASE: u32 = 256;

#[derive(Debug, PartialEq)]
pub enum Command {
    Note(u32),
    SetInstrument(u32),
    SetVolume(u32),
    Play(u32),
    JumpTick(u32), // Unlike LatteFM, this jumps to a specific tick.
    Jump(u32), // This one jumps normally to a specific command.
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
        self.merge_identical_instruments()
            .prune_instruments()
            .apply_semitone_offsets()
            .normalize_volume()
            .resolve_release()
            .prune_commands()
            .prune_channels()
            .split_long_plays()
            .resolve_jumps()
    }
    
    fn apply_semitone_offsets(mut self) -> Self {
        for channel in self.channels.iter_mut() {
            let mut semitone_offset = 0.0;
            for command in channel.iter_mut() {
                match command {
                    Command::Note(n) => *n = ((*n as i32) + (f64::round(semitone_offset) as i32)) as u32,
                    Command::SetInstrument(ins) => semitone_offset = self.instruments[*ins as usize].semitone_offset,
                    _ => ()
                }
            }
        }

        self
    }

    fn merge_identical_instruments(mut self) -> Self {
        let mut instrument_equivalency = HashMap::new();
        for (i1, ins1) in self.instruments.iter().enumerate() {
            for (i2, ins2) in self.instruments.iter().enumerate() {
                if *ins1 == *ins2 {
                    instrument_equivalency.insert(i1, i2);
                    break;
                }
            }
        }

        for channel in self.channels.iter_mut() {
            for command in channel.iter_mut() {
                if let Command::SetInstrument(ins) = command {
                    *command = Command::SetInstrument(
                        *instrument_equivalency.get(&(*ins as usize)).unwrap() as u32
                    );
                }
            }
        }

        self
    }

    fn prune_instruments(mut self) -> Self {
        let mut used = vec![false; self.instruments.len()];
        for channel in self.channels.iter() {
            for command in channel.iter() {
                if let Command::SetInstrument(ins) = command {
                    used[*ins as usize] = true;
                }
            }
        }

        let mut index_update = HashMap::new();
        let mut counter = 0;
        for i in 0..used.len() {
            index_update.insert(i, counter);
            if used[i] {
                counter += 1;
            } else {
                self.instruments.remove(counter);
            }
        }

        for channel in self.channels.iter_mut() {
            for command in channel.iter_mut() {
                if let Command::SetInstrument(ins) = command {
                    *command = Command::SetInstrument(
                        *index_update.get(&(*ins as usize)).unwrap() as u32
                    );
                }
            }
        }

        self
    }

    fn prune_commands(mut self) -> Self {
        for channel in self.channels.iter_mut() {
            let mut cur_instrument = -1;
            let mut cur_pan = 0;
            let mut cur_volume = 255;
            let mut cur_note = PAUSE;
            let mut i = 0;
            while i < channel.len() {
                match channel[i] {
                    Command::Note(n) => {
                        if n == cur_note {
                            channel.remove(i);
                            continue;
                        }
                        cur_note = n;
                    },
                    Command::SetInstrument(ins) => {
                        if (ins as i32) == cur_instrument {
                            channel.remove(i);
                            continue;
                        }
                        cur_instrument = ins as i32;
                    },
                    Command::SetVolume(vol) => {
                        if vol == cur_volume {
                            channel.remove(i);
                            continue;
                        }
                        cur_volume = vol;
                    },
                    Command::Pan(p) => {
                        if p == cur_pan {
                            channel.remove(i);
                            continue;
                        }
                        cur_pan = p;
                    },
                    _ => ()
                }
                i += 1
            }
        }
        self
    }

    fn prune_channels(mut self) -> Self {
        self.channels.retain(|channel| {
            // The channel is considered non-empty if there's a non-pause note
            // in it and a non-zero Play.
            let mut has_note: bool = false;
            let mut has_play: bool = false;
            for command in channel.iter() {
                match *command {
                    Command::Note(n) if n != PAUSE => has_note = true,
                    Command::Play(t) if t != 0 => has_play = true,
                    _ => ()
                }
            }
            has_note && has_play
        });
        self
    }

    fn split_long_plays(mut self) -> Self {
        for channel in self.channels.iter_mut() {
            let mut i = 0;
            while i < channel.len() {
                if let Command::Play(ticks) = channel[i] {
                    if ticks > (u8::MAX as u32) {
                        channel[i] = Command::Play(ticks/2);
                        channel.insert(i+1, Command::Play((ticks+1)/2));
                    } else {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }
        }
        self
    }

    fn resolve_jumps(mut self) -> Self {
        // Ensure that all ticks occur between commands
        for channel in self.channels.iter_mut() {
            let mut target_by_tick = HashMap::new();

            // Find needed jump target ticks.
            for command in channel.iter() {
                if let Command::JumpTick(tick) = command {
                    target_by_tick.insert(*tick, -1i32);
                }
            }

            let mut tick = 0;
            let mut i = 0;
            while i < channel.len() {
                let prev_tick = tick;
                let mut step_ticks = 0;
                if let Command::Play(ticks) = channel[i] {
                    step_ticks = ticks;
                    tick += step_ticks;
                }

                // In the same loop, we handle the easy cases where the ticks fall
                // in between commands nicely and save the earliest required
                // split position if the above isn't true for some jump target.
                let mut earliest_split = 0;
                for (&jump_tick, _) in target_by_tick.iter() {
                    if jump_tick > prev_tick && jump_tick < tick {
                        // Jump target was in between, so we need to split up.
                        earliest_split = jump_tick - prev_tick;
                    }
                }

                if earliest_split != 0 {
                    channel[i] = Command::Play(earliest_split);
                    channel.insert(
                        i+1,
                        Command::Play(step_ticks-earliest_split)
                    );
                }

                i += 1;
            }

            // Find targets for the jump ticks.
            tick = 0;
            for (i, command) in channel.iter().enumerate() {
                for (&jump_tick, target_index) in target_by_tick.iter_mut() {
                    if tick == jump_tick && *target_index == -1 {
                        *target_index = i as i32;
                    }
                }
                if let Command::Play(ticks) = command {
                    tick += ticks;
                }
            }

            // Translate all JumpTicks into regular Jumps
            for command in channel.iter_mut() {
                if let Command::JumpTick(tick) = command {
                    *command = Command::Jump(*target_by_tick.get(tick).unwrap() as u32);
                }
            }
        }
        self
    }

    fn resolve_release(mut self) -> Self {
        for channel in self.channels.iter_mut() {
            let mut release_is_sustain = false;

            // First pass: convert releases that should be pauses, into pauses.
            for command in channel.iter_mut() {
                match command {
                    Command::SetInstrument(ins) => {
                        release_is_sustain = self.instruments[*ins as usize].release_is_sustain;
                    },
                    Command::Note(key) if *key == RELEASE => {
                        if !release_is_sustain {
                            *key = PAUSE;
                        }
                    },
                    _ => {}
                }
            }

            // Second pass: remove remaining releases, merge their surrounding
            // plays.
            let mut i = 0;
            while i < channel.len() {
                if channel[i] == Command::Note(RELEASE) {
                    channel.remove(i);
                    // Why doesn't if let support &&, that seems like such an
                    // obvious defect...
                    if i > 0 {
                        if let Command::Play(b_ticks) = channel[i] {
                            if let Command::Play(a_ticks) = &mut channel[i-1] {
                                *a_ticks += b_ticks;
                                channel.remove(i);
                            }
                        }
                    }
                } else {
                    i += 1;
                }
            }
        }
        self
    }

    fn normalize_volume(mut self) -> Self {
        // Works by determining the maximum possible amplitude at all ticks
        // during the song, then reducing all amplitudes of all instruments by
        // a factor that causes that maximum amplitude to be below 1.
        let mut max_amplitudes = vec![];

        for channel in self.channels.iter() {
            let mut amplitude = 0.0;
            let mut tick = 0;
            let mut volume_scale = 1.0;

            for command in channel.iter() {
                match command {
                    &Command::SetInstrument(ins) => {
                        amplitude = f64::max(
                            self.instruments[ins as usize].amplitude,
                            self.instruments[ins as usize].sustain
                        );
                    },
                    &Command::SetVolume(vol) => {
                        volume_scale = (vol as f64)/255.0;
                    },
                    &Command::Play(ticks) => {
                        for _ in 0..ticks {
                            if max_amplitudes.len() <= tick {
                                max_amplitudes.push(amplitude*volume_scale);
                            } else {
                                max_amplitudes[tick] += amplitude*volume_scale;
                            }
                            tick += 1;
                        }
                    },
                    _ => (),
                }
            }
        }

        let max_amplitude = max_amplitudes.iter().fold(
            0.0, |max_amplitude, &cur_amplitude| f64::max(max_amplitude, cur_amplitude)
        );

        // Adjust all amplitudes 
        for instrument in self.instruments.iter_mut() {
            instrument.amplitude /= max_amplitude;
            instrument.sustain /= max_amplitude;
        }

        self
    }

    pub fn print_as_source(&self) {
        println!(
            concat!(
                "use lattefm::instrument::{{Instrument, Waveform}};\n",
                "use lattefm::tune::Tune;\n",
                "use lattefm::channel::{{Command, note_names::*, Command::*}};\n\n",
                "const INSTRUMENTS: [Instrument; {}] = ["
            ),
            self.instruments.len()
        );
        for ins in self.instruments.iter() {
            println!(
                concat!(
                    "    Instrument {{\n",
                    "        carrier_waveform: Waveform::{:?},\n",
                    "        amplitude: {},\n",
                    "        attack: {},\n",
                    "        decay: {},\n",
                    "        sustain: {},\n",
                    "        release: {},\n",
                    "        modulator_waveform: Waveform::{:?},\n",
                    "        modulator_amplitude: {},\n",
                    "        modulator_mul: {},\n",
                    "        modulator_div: {},\n",
                    "        modulator_phase: {},\n",
                    "    }},"
                ),
                ins.carrier_waveform,
                (ins.amplitude * (u16::MAX as f64)).floor() as u16,
                (ins.attack * ((1<<12) as f64)) as u16,
                (ins.decay * ((1<<12) as f64)) as u16,
                (ins.sustain * (u16::MAX as f64)).floor() as u16,
                (ins.release * ((1<<12) as f64)) as u16,
                ins.modulator_waveform,
                (ins.modulator_amplitude * (u16::MAX as f64)).floor() as u16,
                ins.modulator_mul,
                ins.modulator_div,
                (ins.modulator_phase * (u16::MAX as f64)).floor() as u16
            );
        }
        println!("];\n");

        for (i, channel) in self.channels.iter().enumerate() {
            println!("const CHANNEL{}: [Command; {}] = [", i, channel.len());
            for command in channel.iter() {
                if let Command::Note(n) = command {
                    let note_name = if *n == PAUSE {
                        String::from("PAUSE")
                    } else {
                        String::from([
                            "C", "CH", "D", "DH", "E", "F",
                            "FH", "G", "GH", "A", "AH", "B"
                        ][(n%12) as usize]) + &(n/12).to_string()
                    };
                    println!("    Note({}),", note_name);
                } else {
                    println!("    {:?},", command);
                }
            }
            println!("];\n");
        }

        println!("const CHANNELS: [&'static[Command]; {}] = [", self.channels.len());
        for i in 0..self.channels.len() {
            println!("    &CHANNEL{},",i);
        }
        println!("];\n");

        println!(
            concat!(
                "pub const TUNE: Tune = Tune {{\n",
                "    samplerate: {},\n",
                "    tick_length: {},\n",
                "    instruments: &INSTRUMENTS,\n",
                "    channels: &CHANNELS,\n",
                "}};"
            ),
            44100,
            (44100.0 * self.tick_length).round() as i32
        );
    }
}
