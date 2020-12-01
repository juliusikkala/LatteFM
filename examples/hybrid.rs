use pm_mod::instrument::{Instrument, Waveform};
use pm_mod::channel::Command;
use pm_mod::tune::Tune;
use pm_mod::player::Player;
use sdl2;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::time::Duration;

const INSTRUMENTS: [Instrument; 1] = [
    Instrument{
        carrier_waveform: Waveform::Sine,
        amplitude: u16::MAX,
        attack: 0,
        decay: 0,
        sustain: u16::MAX,
        release: 0,
        pan: 0,
        modulator_waveform: Waveform::Sine,
        modulator_amplitude: u16::MAX/2,
        modulator_mul: 1,
        modulator_div: 1,
        modulator_phase: 0
    }
];

use pm_mod::channel::note_names::*;
const CHANNEL0: [Command; 43] = [
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
    Command::Note(E5), Command::Beat(1),
    Command::Note(PAUSE), Command::Beat(1),
    Command::Note(E5), Command::Beat(1),
    Command::Note(DH5), Command::Beat(1),
    Command::Note(B4), Command::Beat(2),
    Command::Note(CH5), Command::Beat(2),
    Command::Note(DH5), Command::Beat(2),
    Command::Note(FH5), Command::Beat(1),
    Command::Note(GH5), Command::Beat(1),
    Command::Note(GH4), Command::Beat(1),
    Command::Note(PAUSE), Command::Beat(1),
    Command::Jump(0)
];

const CHANNELS: [&'static[Command]; 1]= [&CHANNEL0];

const EXAMPLE: Tune = Tune{
    samplerate: 44100,
    beat_length: 5088,
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

    std::thread::sleep(Duration::from_millis(10000));
}
