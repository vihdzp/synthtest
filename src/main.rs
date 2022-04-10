#![allow(dead_code)]

use std::path;

use basic::Instrument;

pub mod basic;
pub mod scales;
pub mod wav;

const DEFAULT_SAMPLE_RATE: u32 = 44100;
const SAMPLE_RATE: u32 = DEFAULT_SAMPLE_RATE;

fn main() {
    let mut data = wav::AudioData::<i16, 1>::new(SAMPLE_RATE);

    data.add_data_at(0, basic::Square::default().iter(SAMPLE_RATE).take(44100));
    data.add_data_at(0, basic::Saw::default().iter(SAMPLE_RATE).take(44100));
    data.save_to(path::Path::new("D:/Violeta/synthtest.wav"))
        .unwrap();
}
