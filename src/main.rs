use std::{
	str::FromStr,
	sync::atomic::{AtomicUsize, Ordering::SeqCst},
};

use anyhow::Result;
use clap::Parser;
use futures::{stream::FuturesUnordered, TryFutureExt};
use library_tracing_setup::setup_tracing;
use tokio::{fs, runtime::Builder, time::Instant};
use tracing::{event, Level};
use zpl_gen_for_library::{generate_zpl, paste_to_file, Args, TagData};

static THREAD_ID: AtomicUsize = AtomicUsize::new(1);

fn main() -> Result<()> {
	let args = Args::parse();

	Builder::new_multi_thread()
		.enable_time()
		.thread_name_fn(|| {
			let id = THREAD_ID.fetch_add(1, SeqCst) + 1;
			let output = String::from("zpl-generator-pool-");
			output + &id.to_string()
		})
		.on_thread_stop(|| {
			THREAD_ID.fetch_sub(1, SeqCst);
		})
		.build()?
		.block_on(run(args))
}

#[allow(clippy::unused_async)]
async fn run(args: Args) -> Result<()> {
	setup_tracing()?;
	let timer = Instant::now();
	let output_dir = args.output_dir.unwrap_or_else(|| "./outputs".into());
	_ = tokio::fs::remove_dir_all(&output_dir).await;
	tokio::fs::create_dir(&output_dir).await?;

	match (
		args.first_number,
		args.amount_to_print.zip(args.chunk_size),
		args.last_number,
		args.reprint_file,
	) {
		(None, None, None, None) => {
			panic!("must specify either amount_to_print or last_number or reprint_file")
		}
		(Some(first_number), Some((amount_to_print, chunk_size)), _, _) => {
			let range = first_number..(first_number + amount_to_print);

			let futures = FuturesUnordered::new();
			let mut output = Vec::<TagData>::with_capacity(chunk_size);

			for (index, i) in range.enumerate() {
				if output.len() >= chunk_size {
					let final_output =
						std::mem::replace(&mut output, Vec::with_capacity(chunk_size));
					let final_dir = output_dir.clone();
					futures.push(tokio::spawn(async move {
						paste_to_file(final_dir, final_output.into_iter()).await
					}));
				}

				output.push(TagData::new(index, generate_zpl(i)));
			}

			futures.push(tokio::spawn(async move {
				paste_to_file(output_dir, output.into_iter()).await
			}));

			futures::future::try_join_all(futures)
				.map_ok(|values| values.into_iter().collect::<std::io::Result<()>>())
				.await??;
		}
		(Some(first_number), None, Some(last_number), _) => {
			let data = [
				TagData::new(0, generate_zpl(first_number)),
				TagData::new(1, generate_zpl(last_number)),
			];

			paste_to_file(output_dir, data.into_iter()).await?;
		}
		(None, None, None, Some(reprint_file)) => {
			#[allow(clippy::needless_collect)]
			let data = fs::read_to_string(reprint_file)
				.await?
				.lines()
				.map(<u32 as FromStr>::from_str)
				.enumerate()
				.filter_map(|(i, value)| {
					let value = value.ok()?;

					let data = TagData::new(i, generate_zpl(value));

					Some(data)
				})
				.collect::<Vec<_>>();

			paste_to_file(output_dir, data.into_iter()).await?;
		}
		_ => panic!("idk what you did."),
	}

	let time = Instant::now().duration_since(timer);

	event!(Level::INFO, "took {time:?}");

	Ok(())
}
