use crate::instrument::Instrument;
use crate::channel::Command;

pub struct Tune {
    pub samplerate: i32,
    pub tick_length: i32,
    pub instruments: &'static[Instrument],
    pub channels: &'static[&'static[Command]]
}

