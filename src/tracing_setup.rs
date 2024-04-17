use anyhow::Result;
use tracing::Subscriber;
use tracing_subscriber::{
	fmt::{
		self,
		format::{Format, Pretty},
	},
	prelude::*,
	EnvFilter,
};

pub fn setup_tracing() -> Result<()> {
	let log_filter_layer = EnvFilter::try_from_default_env().or_else(|_| {
		EnvFilter::try_new(if cfg!(debug_assertions) {
			"trace"
		} else {
			"debug"
		})
	})?;

	let log_fmt_layer = setup_console();

	tracing_subscriber::registry()
		.with(log_filter_layer)
		.with(log_fmt_layer)
		.try_init()?;

	Ok(())
}

fn setup_console<S>() -> fmt::Layer<S, Pretty, Format<Pretty>>
where
	S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
	fmt::layer()
		.pretty()
		.with_ansi(true)
		.with_thread_ids(false)
		.with_file(false)
		.with_thread_names(true)
}
