use binread::prelude::*; // BinReadExt is in the prelude
use binread::{BinResult, io::{Read, Seek, SeekFrom}};
use std::fs;
use std::collections::HashMap;
use crate::intermediate;

// https://github.com/milkytracker/MilkyTracker/blob/master/resources/reference/xm-form.txt
#[derive(BinRead)]
#[br(little, magic = b"Extended Module: ")]
struct Header {
    name: [u8; 20],
    _1a: u8, // Basically ignored, always 0x1a
    tracker_name: [u8; 20],
    version: u16,
    header_size: u32,
    song_length: u16,
    song_restart_pos: u16,
    num_channels: u16,
    num_patterns: u16,
    num_instruments: u16,
    flags: u16,
    speed: u16,
    bpm: u16,
    #[br(count = header_size-20)]
    pattern_order_table: Vec<u8>,
}

struct Note {
    note: Option<u8>,
    instrument: Option<u8>,
    volume: Option<u8>,
    effect_type: Option<u8>,
    effect_parameter: Option<u8>,
}

// If Note::note is KEY_OFF, the release period of that note occurs.
const KEY_OFF: u8 =  97;

impl Default for Note {
    fn default() -> Note {
        Note{
            note: None,
            instrument: None,
            volume: None,
            effect_type: None,
            effect_parameter: None
        }
    }
}

impl Note {
    fn parse<R: Read + Seek>(reader: &mut R, _ro: &binread::ReadOptions, args: (u16,)) -> BinResult<Vec<Note>> {
        let mut notes: Vec<Note> = Default::default();
        let mut pattern_data_size: i32 = args.0 as i32;
        while pattern_data_size > 0 {
            let note_or_packing: u8 = reader.read_le()?;
            let mut n: Note = Default::default();
            pattern_data_size -= 1;
            if (note_or_packing & 0x80) != 0 {
                // Packed
                if (note_or_packing & 0x01) != 0 {
                    n.note = Some(reader.read_le()?);
                    pattern_data_size -= 1;
                }
                if (note_or_packing & 0x02) != 0 {
                    n.instrument = Some(reader.read_le()?);
                    pattern_data_size -= 1;
                }
                if (note_or_packing & 0x04) != 0 {
                    n.volume = Some(reader.read_le()?);
                    pattern_data_size -= 1;
                }
                if (note_or_packing & 0x08) != 0 {
                    n.effect_type = Some(reader.read_le()?);
                    pattern_data_size -= 1;
                }
                if (note_or_packing & 0x10) != 0 {
                    n.effect_parameter = Some(reader.read_le()?);
                    pattern_data_size -= 1;
                }
            } else {
                // Not packed
                n.note = Some(note_or_packing);
                n.instrument = Some(reader.read_le()?);
                n.volume = Some(reader.read_le()?);
                n.effect_type = Some(reader.read_le()?);
                n.effect_parameter = Some(reader.read_le()?);
                pattern_data_size -= 4;
            }

            notes.push(n);
        }
        assert!(pattern_data_size == 0);
        Ok(notes)
    }
}

#[derive(BinRead)]
struct Pattern {
    length: u32,
    _packing_type: u8,
    num_rows: u16,
    #[br(pad_size_to(length-7))]
    pattern_data_size: u16,
    #[br(parse_with = Note::parse, args(pattern_data_size))]
    pattern_data: Vec<Note>
}

#[derive(BinRead)]
struct InstrumentExtraHeader {
    sample_header_length: u32,
    #[br(count = 96)]
    sample_number: Vec<u8>,
    #[br(count = 12)]
    volume_envelope_points: Vec<(u16, u16)>,
    #[br(count = 12)]
    panning_envelope_points: Vec<(u16, u16)>,
    num_volume_points: u8,
    num_panning_points: u8,
    volume_sustain_point: u8,
    volume_loop_start_point: u8,
    volume_loop_end_point: u8,
    panning_sustain_point: u8,
    panning_loop_start_point: u8,
    panning_loop_end_point: u8,
    volume_type: u8,
    panning_type: u8,
    vibrato_type: u8,
    vibrato_sweep: u8,
    vibrato_depth: u8,
    vibrato_rate: u8,
    volume_fadeout: u16,
    reserved: u16,
}

#[derive(BinRead, Clone)]
struct SampleHeader {
    length: u32,
    loop_start: u32,
    loop_length: u32,
    volume: u8,
    finetune: u8,
    sample_type: u8,
    panning: u8,
    relative_note_number: u8,
    reserved: u8,
    name: [u8; 22]
}

enum Sample {
    Depth8(Vec<i8>),
    Depth16(Vec<i16>)
}

impl Sample {
    fn parse<R: Read + Seek>(reader: &mut R, _ro: &binread::ReadOptions, args: (Vec<SampleHeader>,)) -> BinResult<Vec<Sample>> {
        let headers = &args.0;
        let mut samples: Vec<Sample> = vec![];
        for header in headers {
            if (header.sample_type&0x10) == 1 {
                let mut data: Vec<i16> = vec![0; header.length as usize];
                // TODO: Remove code duplication with generics (but seems like
                // Rust doesn't make such a simple thing easy because who would
                // want to make integer size generic -.-)
                let mut acc: i16 = 0;
                for new in data.iter_mut() {
                    acc = acc.wrapping_add(reader.read_le::<i16>()?);
                    *new = acc;
                }
                samples.push(Sample::Depth16(data));
            } else {
                let mut data: Vec<i8> = vec![0; header.length as usize];
                let mut acc: i8 = 0;
                for new in data.iter_mut() {
                    acc = acc.wrapping_add(reader.read_le::<i8>()?);
                    *new = acc;
                }
                samples.push(Sample::Depth8(data));
            }
        }
        Ok(samples)
    }
}

#[derive(BinRead)]
struct Instrument {
    header_length: u32,
    name: [u8; 22],
    instrument_type: u8,
    num_samples: u16,
    #[br(if(num_samples > 0))]
    extra_header: Option<InstrumentExtraHeader>,
    #[br(if(header_length > 29), pad_size_to(if num_samples == 0 {header_length-29} else {header_length-243}))]
    _pad: Option<u8>,
    #[br(count = num_samples)]
    sample_headers: Vec<SampleHeader>,
    #[br(parse_with = Sample::parse, args(sample_headers.clone()))]
    samples: Vec<Sample>
}

#[derive(BinRead)]
pub struct File {
    header: Header,
    #[br(count = header.num_patterns)]
    patterns: Vec<Pattern>,
    #[br(count = header.num_instruments)]
    instruments: Vec<Instrument>,
}

impl File {
    pub fn load(path: &str) -> BinResult<File> {
        let mut file = fs::File::open(path)?;
        let xm: File = file.read_le()?;
        Ok(xm)
    }

    pub fn print_preamble(&self) {
        println!("\
            // Module name: {}\n\
            // Tracker name: {}\n\
            // XM version: {:04X}\n\
            // Channels: {}\n\
            // Patterns: {}\n\
            // Instruments: {}\n\
            // Frequency table: {}\n\
            // Speed: {}\n\
            // BPM: {}\n\
            // Author's comments / instrument list:",
            std::str::from_utf8(&self.header.name).unwrap(),
            std::str::from_utf8(&self.header.tracker_name).unwrap(),
            self.header.version,
            self.header.num_channels,
            self.header.num_patterns,
            self.header.num_instruments,
            if (self.header.flags&1) == 1 {"linear"} else {"amiga"},
            self.header.speed,
            self.header.bpm
        );

        for (i, line) in self.instruments.iter().map(|i| std::str::from_utf8(&i.name).unwrap()).enumerate() {
            println!("// {:02X} |{}", i+1, line.trim_matches(char::from(0)));
        }
    }
}

impl From<File> for intermediate::Module {
    fn from(xm: File) -> Self {
        let mut m = intermediate::Module {
            tick_length: 2.5/(xm.header.bpm as f64)*(xm.header.speed as f64),
            instruments: vec![],
            channels: vec![]
        };

        // Because an instrument can have multiple samples, they don't match 1:1
        // with LatteFM's instrument concept. Therefore, every sample of every
        // XM instrument is made into a LatteFM instrument.
        let mut sample_instrument_table = HashMap::new();

        for ins_index in 0..xm.instruments.len() {
            let ins = &xm.instruments[ins_index];
            let mut fit_ins: intermediate::Instrument = Default::default();

            if let Some(ref extra) = ins.extra_header {

                let envelope_points: Vec<(f64, f64)> = extra.volume_envelope_points[0..extra.num_volume_points as usize].iter().map(
                    |x| (2.5/(xm.header.bpm as f64) * x.0 as f64, x.1 as f64 / 64.0)
                ).collect();

                let looping = (extra.volume_type&4) != 0;
                let sustain_index = if looping {
                    extra.volume_loop_start_point as i64
                } else if (extra.volume_type&2) != 0 {
                    extra.volume_sustain_point as i64
                } else {
                    -1
                };

                fit_ins.fit_adsr(&envelope_points, sustain_index, looping);

                for sample_index in 0..(ins.num_samples as usize) {
                    let sample_data: Vec<f64>;
                    let header = &ins.sample_headers[sample_index];
                    let mut sample_ins = fit_ins.clone();
                    match &ins.samples[sample_index] {
                        Sample::Depth8(data) => sample_data = data.iter().map(|x| *x as f64/128.0).collect(),
                        Sample::Depth16(data) => sample_data = data.iter().map(|x| *x as f64/32768.0).collect(),
                    }
                    sample_ins.fit_to_sample(
                        &sample_data,
                        (header.relative_note_number as f64) +
                        (header.finetune as f64/128.0),
                    );
                    sample_instrument_table.insert(
                        (ins_index, sample_index),
                        m.instruments.len() as u32
                    );
                    m.instruments.push(sample_ins);
                }
            }
        }

        // While XM interlaces channels in pattern data, we load the channels
        // one-by-one since that better matches the LatteFM world.
        for channel_index in 0..(xm.header.num_channels as usize) {
            let mut channel = vec![];

            let mut cur_instrument = 0;
            let mut tick_counter = 0;

            for pattern_order_index in 0..(xm.header.song_length as usize) {
                let pattern_index = xm.header.pattern_order_table[pattern_order_index] as usize;
                let pattern = &xm.patterns[pattern_index];

                for row in 0..(pattern.num_rows as usize) {
                    let note = &pattern.pattern_data[
                        row * (xm.header.num_channels as usize) + channel_index
                    ];

                    if let Some(i) = note.instrument {
                        cur_instrument = (i-1) as usize;
                    }

                    if let Some(n) = note.note {
                        if tick_counter != 0 {
                            channel.push(intermediate::Command::Play(tick_counter));
                        }
                        let mut note_command = intermediate::PAUSE;
                        if let Some(ref extra) = xm.instruments[cur_instrument].extra_header {
                            if n < KEY_OFF {
                                let sample_index = extra.sample_number[n as usize-1] as usize;
                                if let Some(&index) = sample_instrument_table.get(&(cur_instrument, sample_index)) {
                                    channel.push(intermediate::Command::SetInstrument(index));
                                    note_command = n as u32 - 1;
                                }
                            } else if n == KEY_OFF {
                                note_command = intermediate::RELEASE;
                            }
                        }
                        channel.push(intermediate::Command::Note(note_command));
                        tick_counter = 1;
                        continue;
                    } else if let Some(volume) = note.volume {
                        // If volume is zeroed, we might as well put the note on pause.
                        // Volume also starts from 0x10 (great idea guys, this
                        // was real nice to debug)
                        if (volume as i32)-0x10 <= 1 {
                            if tick_counter != 0 {
                                channel.push(intermediate::Command::Play(tick_counter));
                            }
                            channel.push(intermediate::Command::Note(intermediate::PAUSE));
                            tick_counter = 1;
                            continue;
                        }
                    }

                    tick_counter += 1;

                }
            }
            if tick_counter != 0 {
                channel.push(intermediate::Command::Play(tick_counter));
            }

            let mut restart_tick = 0;
            for pattern_order_index in 0..(xm.header.song_restart_pos as usize) {
                let pattern_index = xm.header.pattern_order_table[pattern_order_index] as usize;
                let pattern = &xm.patterns[pattern_index];
                restart_tick += pattern.num_rows;
            }
            channel.push(intermediate::Command::JumpTick(restart_tick as u32));
            m.channels.push(channel);
        }

        m.optimize()
    }
}
