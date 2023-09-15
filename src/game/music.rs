use crate::wasm4::*;

pub struct Song {
    pub name: &'static str,
    pub scale: [u16; 8],
    pub f1_pitchchange_timer: u8,
    pub f2_pitchchange_timer: u8,
    pub measure_length: u8,
    pub f1_note_duration: u8,
    pub f2_note_duration: u8,
    pub time_signature: (u8, u8),
}

pub const SONGS: [Song; 6] = [
    Song { // happy cat
        name: "happy",
        scale: [294, 330, 370, 392, 440, 494, 554, 587],
        f1_pitchchange_timer: 5,
        f2_pitchchange_timer: 3,
        measure_length: 12,
        f1_note_duration: 5,
        f2_note_duration: 3,
        time_signature: (3, 4),
    },
    Song { // sneak cat
        name: "sneak",
        scale: [196, 220, 247, 262, 294, 330, 370, 392],
        f1_pitchchange_timer: 17,
        f2_pitchchange_timer: 5,
        measure_length: 30,
        f1_note_duration: 10,
        f2_note_duration: 20,
        time_signature: (1, 3),
    },
    Song { // mosh cat
        name: "mosh ",
        scale: [196, 220, 247, 262, 294, 330, 370, 392],
        f1_pitchchange_timer: 1,
        f2_pitchchange_timer: 2,
        measure_length: 5,
        f1_note_duration: 1,
        f2_note_duration: 15,
        time_signature: (2, 5),
    },
    Song { // rando_cat
        name: "rando",
        scale: [247, 277, 311, 330, 370, 415, 466, 494],
        // scale: [196, 220, 247, 262, 294, 330, 370, 392],
        f1_pitchchange_timer: 19,
        f2_pitchchange_timer: 7,
        measure_length: 4,
        f1_note_duration: 1,
        f2_note_duration: 26,
        time_signature: (4, 4),
    },
    Song { // explore cat
        name: "explr",
        scale: [196, 220, 247, 262, 294, 330, 370, 392],
        // scale: [196, 220, 247, 262, 294, 330, 370, 392],
        f1_pitchchange_timer: 7,
        f2_pitchchange_timer: 11,
        measure_length: 4,
        f1_note_duration: 12,
        f2_note_duration: 18,
        time_signature: (12, 3),
    },
    Song { // eflat cat
        name: "eflat",
        scale: [156, 175, 196, 208, 233, 262, 294, 311],
        // scale: [196, 220, 247, 262, 294, 330, 370, 392],
        f1_pitchchange_timer: 36,
        f2_pitchchange_timer: 5,
        measure_length: 14,
        f1_note_duration: 18,
        f2_note_duration: 6,
        time_signature: (4, 2),
    }
];


pub fn play_bgm(timer: u32, song: &Song) {
        

    let freq1: usize = (timer as usize / song.f1_pitchchange_timer as usize) % song.scale.len();
    let freq2: usize = (timer as usize / song.f2_pitchchange_timer as usize) % song.scale.len();


    let time_signature_numerator: u32 = song.time_signature.0 as u32*song.measure_length as u32;
    let time_signature_denominator: u32 = song.time_signature.1 as u32*song.measure_length as u32;
    if timer % time_signature_numerator == 0 {
        tone(song.scale[freq1] as u32, song.f1_note_duration as u32, 20, TONE_PULSE1);
    }
    if timer % time_signature_denominator == 0 && (freq2 as i32).abs_diff(freq1 as i32) > 1 {
        tone(song.scale[freq2] as u32, song.f2_note_duration as u32, 20, TONE_PULSE2);
    }
}