use std::{io::Result as IoResult, path::Path};

use tokio::fs;
use tracing::{event, Level};

use super::TagData;

#[tracing::instrument(skip_all)]
pub async fn paste_to_file<P, I>(output_dir: P, input_data: I) -> IoResult<()>
where
	I: DoubleEndedIterator<Item = TagData> + Send,
	P: AsRef<Path> + Send,
{
	let v = input_data.into_iter().rev().collect::<Vec<_>>();
	let file_name = {
		let Some(first) = v.first() else {
			return Ok(());
		};

		let Some(last) = v.last() else { return Ok(()) };

		output_dir.as_ref().join(format!(
			"Output file {}-{}.zpl",
			last.index + 1,
			first.index + 1
		))
	};
	event!(Level::INFO, ?file_name, "writing output");
	let output = v
		.into_iter()
		.map(|v| v.data)
		.collect::<Vec<_>>()
		.join("\n\n");

	fs::write(file_name, output).await
}
