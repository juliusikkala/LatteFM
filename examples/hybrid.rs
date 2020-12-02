// This is a cover of the legendary hybrid song / Funky Stars by Quazar of
// Sanxion.
use pm_mod::instrument::{Instrument, Waveform};
use pm_mod::channel::Command;
use pm_mod::tune::Tune;
use pm_mod::player::Player;
use sdl2;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::time::Duration;

const INSTRUMENTS: [Instrument; 6] = [
    Instrument{ // Lead 1
        carrier_waveform: Waveform::Sine,
        amplitude: u16::MAX/6,
        attack: (1<<12)/64,
        decay: (1<<12)/16,
        sustain: u16::MAX/8,
        release: (1<<12)/12,
        pan: 0,
        modulator_waveform: Waveform::Sine,
        modulator_amplitude: u16::MAX/5,
        modulator_mul: 1,
        modulator_div: 1,
        modulator_phase: 0
    },
    Instrument{ // Tweet
        carrier_waveform: Waveform::Sine,
        amplitude: u16::MAX/6,
        attack: (1<<12)/32,
        decay: (1<<12)/32,
        sustain: u16::MAX/8,
        release: (1<<12)/32,
        pan: 0,
        modulator_waveform: Waveform::Sine,
        modulator_amplitude: u16::MAX/2,
        modulator_mul: 2,
        modulator_div: 1,
        modulator_phase: 0
    },
    Instrument{ // Bass
        carrier_waveform: Waveform::Triangle,
        amplitude: u16::MAX/4,
        attack: (1<<12)/32,
        decay: (1<<12)/32,
        sustain: u16::MAX/6,
        release: (1<<12)/32,
        pan: 0,
        modulator_waveform: Waveform::Triangle,
        modulator_amplitude: u16::MAX/2,
        modulator_mul: 1,
        modulator_div: 2,
        modulator_phase: 0
    },
    Instrument{ // Hihat
        carrier_waveform: Waveform::Noise,
        amplitude: u16::MAX/4,
        attack: (1<<12)/64,
        decay: (1<<12)/8,
        sustain: 0,
        release: 0,
        pan: 0,
        modulator_waveform: Waveform::Saw,
        modulator_amplitude: 0,
        modulator_mul: 1,
        modulator_div: 1,
        modulator_phase: 0
    },
    Instrument{ // Bass drum
        carrier_waveform: Waveform::Sine,
        amplitude: u16::MAX/4,
        attack: (1<<12)/64,
        decay: (1<<12)/8,
        sustain: 0,
        release: 0,
        pan: 0,
        modulator_waveform: Waveform::Sine,
        modulator_amplitude: u16::MAX/4,
        modulator_mul: 1,
        modulator_div: 4,
        modulator_phase: 0
    },
    Instrument{ // Lead 2
        carrier_waveform: Waveform::Sine,
        amplitude: u16::MAX/4,
        attack: (1<<12)/64,
        decay: (1<<12)/16,
        sustain: u16::MAX/5,
        release: (1<<12)/12,
        pan: 0,
        modulator_waveform: Waveform::Sine,
        modulator_amplitude: u16::MAX/3,
        modulator_mul: 2,
        modulator_div: 1,
        modulator_phase: 0
    },
];

use pm_mod::channel::note_names::*;
const CHANNEL0: [Command; 43] = [
    Command::SetInstrument(0),
    Command::Note(GH4), Command::Beat(2),
    Command::Note(DH5), Command::Beat(2),
    Command::Note(FH5), Command::Beat(1),
    Command::Note(GH5), Command::Beat(1),
    Command::Note(GH4), Command::Beat(2),
    Command::Note(DH5), Command::Beat(2),
    Command::Note(AH5), Command::Beat(2),
    Command::Note(B5), Command::Beat(2),
    Command::Note(E5), Command::Beat(2),
    Command::Note(GH4), Command::Beat(2),
    Command::Note(E5), Command::Beat(2),
    Command::Note(E5), Command::Beat(1),
    Command::Note(DH5), Command::Beat(1),
    Command::Note(B4), Command::Beat(2),
    Command::Note(CH5), Command::Beat(2),
    Command::Note(DH5), Command::Beat(2),
    Command::Note(FH5), Command::Beat(1),
    Command::Note(GH5), Command::Beat(1),
    Command::Note(GH4), Command::Beat(2),
    Command::Repeat(3),
    Command::Jump(1),
    Command::SetInstrument(5),
    Command::Jump(1),
];

const CHANNEL1: [Command; 24] = [
    Command::SetInstrument(1),
    Command::Note(PAUSE), Command::Beat(64),
    Command::Note(GH5), Command::Beat(1),
    Command::Note(GH5), Command::Beat(1),
    Command::Note(GH5), Command::Beat(2),
    Command::Note(GH5), Command::Beat(1),
    Command::Note(GH5), Command::Beat(2),
    Command::Note(GH5), Command::Beat(1),
    Command::Note(PAUSE), Command::Beat(1),
    Command::Note(GH5), Command::Beat(1),
    Command::Note(GH5), Command::Beat(3),
    Command::Note(GH5), Command::Beat(3),
    Command::Jump(3)
];

const CHANNEL2: [Command; 48] = [
    Command::SetInstrument(2),
    Command::Note(PAUSE), Command::Beat(64),
    Command::Note(GH3), Command::Beat(32),
    Command::Note(GH4), Command::Beat(32),

    Command::Note(GH3), Command::Beat(6),
    Command::Note(GH4), Command::Beat(2),
    Command::Note(GH3), Command::Beat(3),
    Command::Note(GH3), Command::Beat(3),
    Command::Note(DH4), Command::Beat(2),
    Command::Note(E4), Command::Beat(4),
    Command::Note(E4), Command::Beat(2),
    Command::Note(CH4), Command::Beat(4),
    Command::Note(CH4), Command::Beat(4),
    Command::Note(FH4), Command::Beat(2),

    Command::Note(GH3), Command::Beat(6),
    Command::Note(GH4), Command::Beat(2),
    Command::Note(GH3), Command::Beat(3),
    Command::Note(GH3), Command::Beat(3),
    Command::Note(DH4), Command::Beat(2),
    Command::Note(E4), Command::Beat(4),
    Command::Note(E4), Command::Beat(2),
    Command::Note(FH4), Command::Beat(4),
    Command::Note(FH4), Command::Beat(4),
    Command::Note(DH4), Command::Beat(2),
    Command::Jump(7)
];

const CHANNEL3: [Command; 50] = [
    Command::SetInstrument(2),
    Command::Note(PAUSE), Command::Beat(112),
    Command::Note(GH5), Command::Beat(16),
    Command::Note(PAUSE), Command::Beat(64),

    Command::Note(AH5), Command::Beat(1),
    Command::Note(B5), Command::Beat(2),
    Command::Note(AH5), Command::Beat(5),
    Command::Note(PAUSE), Command::Beat(2),
    Command::Note(CH6), Command::Beat(4),
    Command::Note(E5), Command::Beat(2),
    Command::Note(DH6), Command::Beat(4),
    Command::Note(GH5), Command::Beat(2),
    Command::Note(B5), Command::Beat(4),
    Command::Note(AH5), Command::Beat(6),

    Command::Note(AH5), Command::Beat(1),
    Command::Note(B5), Command::Beat(2),
    Command::Note(AH5), Command::Beat(5),
    Command::Note(PAUSE), Command::Beat(2),
    Command::Note(CH6), Command::Beat(4),
    Command::Note(E5), Command::Beat(2),
    Command::Note(GH5), Command::Beat(4),
    Command::Note(AH5), Command::Beat(2),
    Command::Note(B5), Command::Beat(4),
    Command::Note(AH5), Command::Beat(2),
    Command::Note(FH5), Command::Beat(4),

    Command::Jump(7)
];

const CHANNEL4: [Command; 42] = [
    Command::Note(PAUSE), Command::Beat(124),
    Command::SetInstrument(3),
    Command::Note(C5), Command::Beat(1),
    Command::Note(C5), Command::Beat(3),
    Command::SetInstrument(4),
    Command::Note(F3), Command::Beat(4),
    Command::SetInstrument(3),
    Command::Note(C5), Command::Beat(2),
    Command::SetInstrument(4),
    Command::Note(F3), Command::Beat(2),
    Command::Note(F3), Command::Beat(4),
    Command::SetInstrument(3),
    Command::Note(C5), Command::Beat(3),
    Command::Note(C5), Command::Beat(1),
    Command::SetInstrument(4),
    Command::Note(F3), Command::Beat(4),
    Command::SetInstrument(3),
    Command::Note(C5), Command::Beat(4),
    Command::SetInstrument(4),
    Command::Note(F3), Command::Beat(1),
    Command::Note(F3), Command::Beat(2),
    Command::Note(F3), Command::Beat(1),
    Command::SetInstrument(3),
    Command::Note(C5), Command::Beat(3),
    Command::Note(C5), Command::Beat(1),
    Command::Jump(7)
];

const CHANNELS: [&'static[Command]; 5]= [&CHANNEL0, &CHANNEL1, &CHANNEL2, &CHANNEL3, &CHANNEL4];

const EXAMPLE: Tune = Tune{
    samplerate: 8192,
    beat_length: 1024,
    instruments: &INSTRUMENTS,
    channels: &CHANNELS
};

struct MyPlayer<'a> {
    player: Player<'a>
}

impl<'a> AudioCallback for MyPlayer<'a> {
    type Channel = i8;

    fn callback(&mut self, out: &mut [i8]) {
        self.player.generate(out);
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let audio = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(EXAMPLE.samplerate),
        channels: Some(1),
        samples: None
    };
    let mut channels = [Default::default(); CHANNELS.len()];
    let player = MyPlayer{ player: Player::new(&EXAMPLE, &mut channels) };

    let device = audio.open_playback(
        None, &desired_spec, |_spec| {
            player
        }
    ).unwrap();

    device.resume();

    std::thread::sleep(Duration::from_millis(100000));
}
