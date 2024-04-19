use std::{
	io::Result as IoResult,
	path::{Path, PathBuf},
};

use futures::{stream::FuturesUnordered, TryFutureExt as _};
use tokio::fs;
use tracing::{event, Level};

pub use self::flatten::visit as get_all_files;

pub async fn copy_files(
	input: &[PathBuf],
	output_dir: impl AsRef<Path> + Send,
	amount_per_file: usize,
) -> IoResult<()> {
	let futures = FuturesUnordered::new();
	let output_dir = output_dir.as_ref();
	for (i, file_pair) in input.chunks(amount_per_file).enumerate() {
		for (index, file_path) in file_pair.iter().cloned().rev().enumerate() {
			let output_dir = output_dir
				.join(format!("Roll {}", i + 1))
				.join(format!("File {}.zpl", index + 1));
			futures.push(tokio::spawn(async move {
				fs::copy(file_path, output_dir).await
			}));
		}
	}

	futures::future::try_join_all(futures)
		.map_ok(|values| values.into_iter().try_for_each(|r| r.map(|_| {})))
		.await??;

	Ok(())
}

pub async fn create_folders(
	output_dir: impl AsRef<Path> + Send,
	amount_of_folders: usize,
) -> IoResult<()> {
	event!(Level::INFO, "creating folders 1-{}", amount_of_folders);
	let output_dir = output_dir.as_ref();
	_ = fs::remove_dir_all(&output_dir).await;
	let futures = FuturesUnordered::new();

	for i in 0..amount_of_folders {
		let output_dir = output_dir.join(format!("Roll {}", i + 1));
		futures.push(tokio::spawn(
			async move { fs::create_dir_all(output_dir).await },
		));
	}

	futures::future::try_join_all(futures)
		.map_ok(|values| values.into_iter().collect::<IoResult<()>>())
		.await??;

	Ok(())
}

mod flatten {
	use std::{
		io::Result as IoResult,
		path::{Path, PathBuf},
	};

	use futures::{stream, Stream, StreamExt as _, TryStreamExt as _};
	use tokio::fs::{self, DirEntry};
	use tokio_stream::wrappers::ReadDirStream;

	async fn one_level<P>(path: P, to_visit: &mut Vec<PathBuf>) -> IoResult<Vec<DirEntry>>
	where
		P: AsRef<Path> + Send,
	{
		let mut dir = ReadDirStream::new(fs::read_dir(path).await?);

		let mut files = Vec::new();

		while let Some(child) = dir.try_next().await? {
			if child.metadata().await?.is_dir() {
				to_visit.push(child.path());
			} else {
				files.push(child);
			}
		}

		Ok(files)
	}

	pub fn visit<P>(path: P) -> impl Stream<Item = IoResult<DirEntry>>
	where
		P: AsRef<Path>,
	{
		stream::unfold(vec![path.as_ref().to_path_buf()], |mut to_visit| async {
			let path = to_visit.pop()?;
			let file_stream = match one_level(path, &mut to_visit).await {
				Ok(files) => stream::iter(files).map(Ok).left_stream(),
				Err(e) => stream::once(futures::future::err(e)).right_stream(),
			};

			Some((file_stream, to_visit))
		})
		.flatten()
	}
}
