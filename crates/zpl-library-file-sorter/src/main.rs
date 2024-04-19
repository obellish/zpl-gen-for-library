use std::{
	path::PathBuf,
	sync::atomic::{AtomicUsize, Ordering::SeqCst},
};

use anyhow::Result;
use clap::Parser;
use futures::TryStreamExt;
use library_tracing_setup::setup_tracing;
use tokio::runtime::Builder;
use zpl_library_file_sorter::{copy_files, create_folders, get_all_files, Args, NaturalSort};

static THREAD_ID: AtomicUsize = AtomicUsize::new(1);

fn main() -> Result<()> {
	let args = Args::parse();

	Builder::new_current_thread()
		.thread_name_fn(|| {
			let id = THREAD_ID.fetch_add(1, SeqCst) + 1;
			let output = String::from("zpl-library-file-sorter-pool-");
			output + &id.to_string()
		})
		.on_thread_stop(|| {
			THREAD_ID.fetch_sub(1, SeqCst);
		})
		.build()?
		.block_on(run(args))
}

#[allow(clippy::unused_async)]
async fn run(mut args: Args) -> Result<()> {
	setup_tracing()?;

	args.input_directory = args.input_directory.canonicalize()?;
	args.output_directory = args.output_directory.canonicalize()?;

	let files = get_all_files(&args.input_directory)
		.try_collect::<Vec<_>>()
		.await?;

	create_folders(&args.output_directory, files.len() / args.amount_per_folder).await?;

	let mut file_paths = files
		.iter()
		.map(|entry| entry.path().canonicalize().map(|s| s.display().to_string()))
		.collect::<std::io::Result<Vec<_>>>()?;

	file_paths.natural_sort();

	let file_paths = file_paths
		.into_iter()
		.map(PathBuf::from)
		.collect::<Vec<_>>();

	copy_files(&file_paths, &args.output_directory, args.amount_per_folder).await?;

	Ok(())
}
