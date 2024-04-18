mod check_digit;
mod fs;
mod generator;
mod tracing_setup;

use std::path::PathBuf;

use clap::Parser;

pub use self::{fs::paste_to_file, generator::generate_zpl, tracing_setup::setup_tracing};

#[derive(Debug, Clone, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
	/// The first number to generate.
	#[arg(short, long, value_name = "NUMBER")]
	pub first_number: u32,
	/// The amount to print. Either this or [`last_number`](Self::last_number) is required.
	#[arg(short, long, value_name = "AMOUNT")]
	pub amount_to_print: Option<u32>,
	/// The size of each file. Needs to be used with [`amount_to_print`](Self::amount_to_print).
	#[arg(short, long, value_name = "SIZE")]
	pub chunk_size: Option<usize>,
	/// The last number to generate. Either this or [`amount_to_print`](Self::amount_to_print) is required.
	#[arg(short, long, value_name = "NUMBER")]
	pub last_number: Option<u32>,
	/// The directory to output files.
	#[arg(short, long, value_name = "DIRECTORY")]
	pub output_dir: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct TagData {
	index: usize,
	data: String,
}

impl TagData {
	#[must_use]
	pub const fn new(index: usize, data: String) -> Self {
		Self { index, data }
	}
}

impl PartialEq for TagData {
	fn eq(&self, other: &Self) -> bool {
		self.index == other.index
	}
}

impl Eq for TagData {}

impl PartialOrd for TagData {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for TagData {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.index.cmp(&other.index)
	}
}
