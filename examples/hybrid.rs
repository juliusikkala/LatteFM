// This is a cover of the legendary hybrid song / Funky Stars by Quazar of
// Sanxion.
use lattefm::instrument::{Instrument, Waveform};
use lattefm::channel::Command;
use lattefm::tune::Tune;
use lattefm::player::Player;
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
        modulator_waveform: Waveform::Sine,
        modulator_amplitude: u16::MAX/5,
        modulator_mul: 4,
        modulator_div: 1,
        modulator_phase: 0
    },
    Instrument{ // Tweet
        carrier_waveform: Waveform::Sine,
        amplitude: u16::MAX/8,
        attack: (1<<12)/32,
        decay: (1<<12)/32,
        sustain: u16::MAX/12,
        release: (1<<12)/32,
        modulator_waveform: Waveform::Sine,
        modulator_amplitude: u16::MAX/2,
        modulator_mul: 4,
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
        modulator_waveform: Waveform::Sine,
        modulator_amplitude: u16::MAX/3,
        modulator_mul: 2,
        modulator_div: 1,
        modulator_phase: 0
    },
];

use lattefm::channel::{note_names::*, Command::*};
const CHANNEL0: [Command; 45] = [
    SetInstrument(0), Pan(-80),
    Note(GH4), Play(2),
    Note(DH5), Play(2),
    Note(FH5), Play(1),
    Note(GH5), Play(1),
    Note(GH4), Play(2),
    Note(DH5), Play(2),
    Note(AH5), Play(2),
    Note(B5), Play(2),
    Note(E5), Play(2),
    Note(GH4), Play(2),
    Note(E5), Play(2),
    Note(E5), Play(1),
    Note(DH5), Play(1),
    Note(B4), Play(2),
    Note(CH5), Play(2),
    Note(DH5), Play(2),
    Note(FH5), Play(1),
    Note(GH5), Play(1),
    Note(GH4), Play(2),
    Repeat(3),
    Jump(2),
    SetInstrument(5), Pan(-20),
    Jump(2),
];

const CHANNEL1: [Command; 25] = [
    SetInstrument(1), Pan(127),
    Note(PAUSE), Play(64),
    Note(GH5), Play(1),
    Note(GH5), Play(1),
    Note(GH5), Play(2),
    Note(GH5), Play(1),
    Note(GH5), Play(2),
    Note(GH5), Play(1),
    Note(PAUSE), Play(1),
    Note(GH5), Play(1),
    Note(GH5), Play(3),
    Note(GH5), Play(3),
    Jump(4)
];

const CHANNEL2: [Command; 49] = [
    SetInstrument(2), Pan(-10),
    Note(PAUSE), Play(64),
    Note(GH3), Play(32),
    Note(GH4), Play(32),

    Note(GH3), Play(6),
    Note(GH4), Play(2),
    Note(GH3), Play(3),
    Note(GH3), Play(3),
    Note(DH4), Play(2),
    Note(E4), Play(4),
    Note(E4), Play(2),
    Note(CH4), Play(4),
    Note(CH4), Play(4),
    Note(FH4), Play(2),

    Note(GH3), Play(6),
    Note(GH4), Play(2),
    Note(GH3), Play(3),
    Note(GH3), Play(3),
    Note(DH4), Play(2),
    Note(E4), Play(4),
    Note(E4), Play(2),
    Note(FH4), Play(4),
    Note(FH4), Play(4),
    Note(DH4), Play(2),
    Jump(8)
];

const CHANNEL3: [Command; 50] = [
    SetInstrument(2),
    Note(PAUSE), Play(112),
    Note(GH5), Play(16),
    Note(PAUSE), Play(64),

    Note(AH5), Play(1),
    Note(B5), Play(2),
    Note(AH5), Play(5),
    Note(PAUSE), Play(2),
    Note(CH6), Play(4),
    Note(E5), Play(2),
    Note(DH6), Play(4),
    Note(GH5), Play(2),
    Note(B5), Play(4),
    Note(AH5), Play(6),

    Note(AH5), Play(1),
    Note(B5), Play(2),
    Note(AH5), Play(5),
    Note(PAUSE), Play(2),
    Note(CH6), Play(4),
    Note(E5), Play(2),
    Note(GH5), Play(4),
    Note(AH5), Play(2),
    Note(B5), Play(4),
    Note(AH5), Play(2),
    Note(FH5), Play(4),

    Jump(7)
];

const CHANNEL4: [Command; 43] = [
    Note(PAUSE), Play(124), Pan(30),
    SetInstrument(3),
    Note(C5), Play(1),
    Note(C5), Play(3),
    SetInstrument(4),
    Note(C3), Play(4),
    SetInstrument(3),
    Note(C5), Play(2),
    SetInstrument(4),
    Note(C3), Play(2),
    Note(C3), Play(4),
    SetInstrument(3),
    Note(C5), Play(3),
    Note(C5), Play(1),
    SetInstrument(4),
    Note(C3), Play(4),
    SetInstrument(3),
    Note(C5), Play(4),
    SetInstrument(4),
    Note(C3), Play(1),
    Note(C3), Play(2),
    Note(C3), Play(1),
    SetInstrument(3),
    Note(C5), Play(3),
    Note(C5), Play(1),
    Jump(8)
];

const CHANNELS: [&'static[Command]; 5]= [&CHANNEL0, &CHANNEL1, &CHANNEL2, &CHANNEL3, &CHANNEL4];

const EXAMPLE: Tune = Tune{
    samplerate: 8192,
    tick_length: 1024,
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
        channels: Some(2),
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
