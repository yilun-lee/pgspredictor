#![cfg_attr(test, feature(test))]

pub mod meta;
pub mod reader;
pub mod test;

pub use reader::{read_bed_nolib::BedReaderNoLib, ReadGenotype};
pub use reader::freq_reader::{BfileSet,FreqBedReader};
