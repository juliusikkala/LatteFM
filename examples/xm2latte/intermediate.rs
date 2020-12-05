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
    modulator_mul: i64,
    modulator_div: i64,
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
            modulator_mul: 0,
            modulator_div: 0,
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

#[derive(Debug)]
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
        println!(
            concat!(
                "use lattefm::instrument::{{Instrument, Waveform}};\n",
                "use lattefm::tune::Tune;\n",
                "use lattefm::player::Player;\n",
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
                (ins.amplitude * (u16::MAX as f64)).floor() as i16,
                (ins.attack * ((1<<12) as f64)) as i16,
                (ins.decay * ((1<<12) as f64)) as i16,
                (ins.sustain * (u16::MAX as f64)).floor() as i16,
                (ins.release * ((1<<12) as f64)) as i16,
                ins.modulator_waveform,
                (ins.modulator_amplitude * (u16::MAX as f64)).floor() as i16,
                ins.modulator_mul,
                ins.modulator_div,
                (ins.modulator_phase * (u16::MAX as f64)).floor() as i16
            );
        }
        println!("];\n");

        for (i, channel) in self.channels.iter().enumerate() {
            println!("const CHANNEL{}: [Command; {}] = [", i, channel.len());
            for command in channel.iter() {
                if let Command::Note(n) = command {
                    let note_name = String::from([
                        "C", "CH", "D", "DH", "E", "F",
                        "FH", "G", "GH", "A", "AH", "B"
                    ][(n%12) as usize]) + &(n/12).to_string();
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
