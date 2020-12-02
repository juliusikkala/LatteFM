// This was supposed to be an enum, but they have crippling limitations:
// * They don't allow for overlapping values (DH0/Eb0, yes, music notation is
//   stupid legacy trash)
// * The workaround for that (associated constants) doesn't work with 'use', so
//   you would constantly have to repeat the enum name in the file where the
//   song is defined :/
#[allow(non_upper_case_globals)]
pub mod note_names {
    pub const C0: u8 = 0;
    pub const CH0: u8 = 1;
    pub const Db0: u8 = 1;
    pub const D0: u8 = 2;
    pub const DH0: u8 = 3;
    pub const Eb0: u8 = 3;
    pub const E0: u8 = 4;
    pub const F0: u8 = 5;
    pub const FH0: u8 = 6;
    pub const Gb0: u8 = 6;
    pub const G0: u8 = 7;
    pub const GH0: u8 = 8;
    pub const Ab0: u8 = 8;
    pub const A0: u8 = 9;
    pub const AH0: u8 = 10;
    pub const Bb0: u8 = 10;
    pub const B0: u8 = 11;
    pub const C1: u8 = 12;
    pub const CH1: u8 = 13;
    pub const Db1: u8 = 13;
    pub const D1: u8 = 14;
    pub const DH1: u8 = 15;
    pub const Eb1: u8 = 15;
    pub const E1: u8 = 16;
    pub const F1: u8 = 17;
    pub const FH1: u8 = 18;
    pub const Gb1: u8 = 18;
    pub const G1: u8 = 19;
    pub const GH1: u8 = 20;
    pub const Ab1: u8 = 20;
    pub const A1: u8 = 21;
    pub const AH1: u8 = 22;
    pub const Bb1: u8 = 22;
    pub const B1: u8 = 23;
    pub const C2: u8 = 24;
    pub const CH2: u8 = 25;
    pub const Db2: u8 = 25;
    pub const D2: u8 = 26;
    pub const DH2: u8 = 27;
    pub const Eb2: u8 = 27;
    pub const E2: u8 = 28;
    pub const F2: u8 = 29;
    pub const FH2: u8 = 30;
    pub const Gb2: u8 = 30;
    pub const G2: u8 = 31;
    pub const GH2: u8 = 32;
    pub const Ab2: u8 = 32;
    pub const A2: u8 = 33;
    pub const AH2: u8 = 34;
    pub const Bb2: u8 = 34;
    pub const B2: u8 = 35;
    pub const C3: u8 = 36;
    pub const CH3: u8 = 37;
    pub const Db3: u8 = 37;
    pub const D3: u8 = 38;
    pub const DH3: u8 = 39;
    pub const Eb3: u8 = 39;
    pub const E3: u8 = 40;
    pub const F3: u8 = 41;
    pub const FH3: u8 = 42;
    pub const Gb3: u8 = 42;
    pub const G3: u8 = 43;
    pub const GH3: u8 = 44;
    pub const Ab3: u8 = 44;
    pub const A3: u8 = 45;
    pub const AH3: u8 = 46;
    pub const Bb3: u8 = 46;
    pub const B3: u8 = 47;
    pub const C4: u8 = 48;
    pub const CH4: u8 = 49;
    pub const Db4: u8 = 49;
    pub const D4: u8 = 50;
    pub const DH4: u8 = 51;
    pub const Eb4: u8 = 51;
    pub const E4: u8 = 52;
    pub const F4: u8 = 53;
    pub const FH4: u8 = 54;
    pub const Gb4: u8 = 54;
    pub const G4: u8 = 55;
    pub const GH4: u8 = 56;
    pub const Ab4: u8 = 56;
    pub const A4: u8 = 57;
    pub const AH4: u8 = 58;
    pub const Bb4: u8 = 58;
    pub const B4: u8 = 59;
    pub const C5: u8 = 60;
    pub const CH5: u8 = 61;
    pub const Db5: u8 = 61;
    pub const D5: u8 = 62;
    pub const DH5: u8 = 63;
    pub const Eb5: u8 = 63;
    pub const E5: u8 = 64;
    pub const F5: u8 = 65;
    pub const FH5: u8 = 66;
    pub const Gb5: u8 = 66;
    pub const G5: u8 = 67;
    pub const GH5: u8 = 68;
    pub const Ab5: u8 = 68;
    pub const A5: u8 = 69;
    pub const AH5: u8 = 70;
    pub const Bb5: u8 = 70;
    pub const B5: u8 = 71;
    pub const C6: u8 = 72;
    pub const CH6: u8 = 73;
    pub const Db6: u8 = 73;
    pub const D6: u8 = 74;
    pub const DH6: u8 = 75;
    pub const Eb6: u8 = 75;
    pub const E6: u8 = 76;
    pub const F6: u8 = 77;
    pub const FH6: u8 = 78;
    pub const Gb6: u8 = 78;
    pub const G6: u8 = 79;
    pub const GH6: u8 = 80;
    pub const Ab6: u8 = 80;
    pub const A6: u8 = 81;
    pub const AH6: u8 = 82;
    pub const Bb6: u8 = 82;
    pub const B6: u8 = 83;
    pub const C7: u8 = 84;
    pub const CH7: u8 = 85;
    pub const Db7: u8 = 85;
    pub const D7: u8 = 86;
    pub const DH7: u8 = 87;
    pub const Eb7: u8 = 87;
    pub const E7: u8 = 88;
    pub const F7: u8 = 89;
    pub const FH7: u8 = 90;
    pub const Gb7: u8 = 90;
    pub const G7: u8 = 91;
    pub const GH7: u8 = 92;
    pub const Ab7: u8 = 92;
    pub const A7: u8 = 93;
    pub const AH7: u8 = 94;
    pub const Bb7: u8 = 94;
    pub const B7: u8 = 95;
    pub const C8: u8 = 96;
    pub const CH8: u8 = 97;
    pub const Db8: u8 = 97;
    pub const D8: u8 = 98;
    pub const DH8: u8 = 99;
    pub const Eb8: u8 = 99;
    pub const E8: u8 = 100;
    pub const F8: u8 = 101;
    pub const FH8: u8 = 102;
    pub const Gb8: u8 = 102;
    pub const G8: u8 = 103;
    pub const GH8: u8 = 104;
    pub const Ab8: u8 = 104;
    pub const A8: u8 = 105;
    pub const AH8: u8 = 106;
    pub const Bb8: u8 = 106;
    pub const B8: u8 = 107;
    pub const PAUSE: u8 = 255;
}

pub enum Command {
    Note(u8), // Pitch, 0 is release. In semitones from C0 upwards. B8 is the highest allowed note.
    SetInstrument(u8), // Changes instrument to the given index. Always set the instrument before starting the note it should play.
    Beat(u8), // Steps given number of beats ahead
    Jump(u8), // Jump to the given command index (used for looping)
    Repeat(u8), // Repeats the following command only N times, with the Nth time skipping the command. Useful with Jump.
}
