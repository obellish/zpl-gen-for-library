mod fs;
mod sorting;

use std::path::PathBuf;

use clap::Parser;

pub use self::{fs::*, sorting::*};

#[derive(Debug, Clone, Parser)]
pub struct Args {
	#[arg(short, long, value_name = "DIRECTORY")]
	/// The directory where files are located.
	pub input_directory: PathBuf,
	#[arg(short, long, value_name = "DIRECTORY")]
	/// The directory to place all the files.
	pub output_directory: PathBuf,
	#[arg(short, long, value_name = "AMOUNT")]
	/// The amount of files per folder.
	pub amount_per_folder: usize,
}
