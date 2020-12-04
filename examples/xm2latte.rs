use binread::prelude::*; // BinReadExt is in the prelude
use binread::{BinResult, io::{Read, Seek, SeekFrom}};
use std::fs::File;
use std::env;

// https://github.com/milkytracker/MilkyTracker/blob/master/resources/reference/xm-form.txt
#[derive(BinRead)]
#[br(little, magic = b"Extended Module: ")]
struct XmHeader {
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
    tempo: u16,
    bpm: u16,
    #[br(count = header_size-20)]
    pattern_order_table: Vec<u8>,
}

struct XmNote {
    note: Option<u8>,
    instrument: Option<u8>,
    volume: Option<u8>,
    effect_type: Option<u8>,
    effect_parameter: Option<u8>,
}

impl Default for XmNote {
    fn default() -> XmNote {
        XmNote{
            note: None,
            instrument: None,
            volume: None,
            effect_type: None,
            effect_parameter: None
        }
    }
}

impl XmNote {
    fn parse<R: Read + Seek>(reader: &mut R, _ro: &binread::ReadOptions, args: (u16,)) -> BinResult<Vec<XmNote>> {
        let mut notes: Vec<XmNote> = Default::default();
        let mut pattern_data_size: i32 = args.0 as i32;
        while pattern_data_size > 0 {
            let note_or_packing: u8 = reader.read_le()?;
            let mut n: XmNote = Default::default();
            pattern_data_size -= 1;
            if (note_or_packing & 0x80) == 1 {
                // Packed
                if (note_or_packing & 0x01) == 1 {
                    n.note = Some(reader.read_le()?);
                    pattern_data_size -= 1;
                }
                if (note_or_packing & 0x02) == 1 {
                    n.instrument = Some(reader.read_le()?);
                    pattern_data_size -= 1;
                }
                if (note_or_packing & 0x04) == 1 {
                    n.volume = Some(reader.read_le()?);
                    pattern_data_size -= 1;
                }
                if (note_or_packing & 0x08) == 1 {
                    n.effect_type = Some(reader.read_le()?);
                    pattern_data_size -= 1;
                }
                if (note_or_packing & 0x10) == 1 {
                    n.effect_parameter = Some(reader.read_le()?);
                    pattern_data_size -= 1;
                }
            } else {
                // Not packed
                n.note = Some(note_or_packing);
            }
            notes.push(n);
        }
        assert!(pattern_data_size == 0);
        Ok(notes)
    }
}

#[derive(BinRead)]
struct XmPattern {
    length: u32,
    _packing_type: u8,
    num_rows: u16,
    #[br(pad_size_to(length-7))]
    pattern_data_size: u16,
    #[br(parse_with = XmNote::parse, args(pattern_data_size))]
    pattern_data: Vec<XmNote>
}

#[derive(BinRead)]
struct XmInstrumentExtraHeader {
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
struct XmSampleHeader {
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

enum XmSample {
    Depth8(Vec<i8>),
    Depth16(Vec<i16>)
}

impl XmSample {
    fn parse<R: Read + Seek>(reader: &mut R, _ro: &binread::ReadOptions, args: (Vec<XmSampleHeader>,)) -> BinResult<Vec<XmSample>> {
        let headers = &args.0;
        let mut samples: Vec<XmSample> = vec![];
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
                samples.push(XmSample::Depth16(data));
            } else {
                let mut data: Vec<i8> = vec![0; header.length as usize];
                let mut acc: i8 = 0;
                for new in data.iter_mut() {
                    acc = acc.wrapping_add(reader.read_le::<i8>()?);
                    *new = acc;
                }
                samples.push(XmSample::Depth8(data));
            }
        }
        Ok(samples)
    }
}

#[derive(BinRead)]
struct XmInstrument {
    header_length: u32,
    name: [u8; 22],
    instrument_type: u8,
    num_samples: u16,
    #[br(if(num_samples > 0))]
    extra_header: Option<XmInstrumentExtraHeader>,
    #[br(pad_size_to(if num_samples == 0 {header_length-29} else {header_length-243}))]
    pad: u8,
    #[br(count = num_samples)]
    sample_headers: Vec<XmSampleHeader>,
    #[br(parse_with = XmSample::parse, args(sample_headers.clone()))]
    samples: Vec<XmSample>
}

#[derive(BinRead)]
struct XmFile {
    header: XmHeader,
    #[br(count = header.num_patterns)]
    patterns: Vec<XmPattern>,
    #[br(count = header.num_instruments)]
    instruments: Vec<XmInstrument>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} file.xm", args[0]);
        return;
    }

    let filename = &args[1];
    let mut file = File::open(filename).expect("Failed to open specified file");
    let xm: XmFile = file.read_le().expect("Failed to parse XM file");

    println!("\
        // Automatically generated LatteFM music file.\n\
        //\n\
        // Original XM filename: {}\n\
        // Module name: {}\n\
        // Tracker name: {}\n\
        // XM version: {:04X}\n\
        // Channels: {}\n\
        // Patterns: {}\n\
        // Instruments: {}\n\
        // Frequency table: {}\n\
        // Tempo: {}\n\
        // BPM: {}\n\
        // Author's comments / instrument list:",
        filename,
        std::str::from_utf8(&xm.header.name).unwrap(),
        std::str::from_utf8(&xm.header.tracker_name).unwrap(),
        xm.header.version,
        xm.header.num_channels,
        xm.header.num_patterns,
        xm.header.num_instruments,
        if (xm.header.flags&1) == 1 {"linear"} else {"amiga"},
        xm.header.tempo,
        xm.header.bpm
    );

    for (i, line) in xm.instruments.iter().map(|i| std::str::from_utf8(&i.name).unwrap()).enumerate() {
        println!("// {:02X} |{}", i+1, line);
    }
}
