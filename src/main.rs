#![allow(clippy::unnecessary_wraps)]

#[cfg(not(debug_assertions))]
use std::ops::RangeInclusive;
use std::{
	borrow::Cow,
	sync::atomic::{AtomicUsize, Ordering},
};

use miette::{IntoDiagnostic as _, Result};
use tokio::runtime::Builder;

#[cfg(not(debug_assertions))]
const RANGE: RangeInclusive<u32> = 50_000_000..=50_029_999;
#[cfg(debug_assertions)]
// const RANGE: RangeInclusive<u32> = 50_000_000..=50_000_001;
const RANGE: [u32; 2] = [50_000_000, 50_029_999];
const DEFAULT_DATA: &[Cow<'static, str>] = &[
	Cow::Borrowed("^XA"),
	Cow::Borrowed("^IDR:*.*"),
	Cow::Borrowed("^XZ"),
	Cow::Borrowed("^XA"),
	Cow::Borrowed("^SZ2"),
	Cow::Borrowed("^XZ"),
	Cow::Borrowed("^XA"),
	Cow::Borrowed("^PON"),
	Cow::Borrowed("^PW635"),
	Cow::Borrowed("^LL929"),
	Cow::Borrowed("^LH0,006"),
	Cow::Borrowed("^MNY"),
	Cow::Borrowed("^XZ"),
	Cow::Borrowed("~DGET0,1800,45,z0m03FFF8F00F007F807FFF1F01E1FFFC007FF81E01E1FFF01E000F007F0000F000787FFC07FFC007F007FFC1F00F83FFF8F00F01FFE07FFF1F01E1FFFC007FFC1E01E1FFFC1E000F01FFC000F000787FFF07FFF007F007FFF0F00F03FFF8F00F03FFF07FFF1F81E1FFFC007FFE1E01E1FFFC1E000F03FFF000F000787FFF07FFF00FF807FFF0F81F03C000F00F07C1F078001FC1E1EJ00781F1E01E1E03E1E000F07C1F000F00078780F8780F80F780780F87C3E03C000F00F0F00F878001FC1E1EJ00780F1E01E1E01E1E000F0F00F800F0007878078780780F780780783C3C03C000F00F0F006078001FE1E1EJ00780F1E01E1E01E1E000F0F006000F0007878078780781F7C0780783E7C03C000F00F1E000078001FF1E1EJ00780F1E01E1E03C1E000F1E0J00F00078780F0780F81E3C0780F81E7803FFF0F00F1E00007FFE1FF1E1FFF800781F1E01E1FFFC1E000F1E0J00F000787FFF07FFF01E3C07FFF00FF003FFF0F00F1E07F87FFE1EF9E1FFF8007FFE1E01E1FFF81E000F1E0J00F000787FFE07FFE03E3E07FFE007E003FFF0F00F1E07F87FFE1E7DE1FFF8007FFC1E01E1FFFE1E000F1E0J00F000787FFF87FF803C1E07FF8007E003C000F00F1E07F878001E3DE1EJ007FF81E01E1E01E1E000F1E0J00F0007878078787C03C1E0787C003C003C000F00F1E007878001E3FE1EJ0078001E01E1E00F1E000F1E006000F000787803C783E07FFF0783E003C003C000F00F0F007878001E1FE1EJ0078001E01E1E00F1E000F0F00F800F000787803C783E07FFF0783E003C003C000F80F0F807878001E0FE1EJ0078001F01E1E00F1E000F0F00F000F000787803C781F07FFF0781F003C003C0007C1E07C1F878001E0FE1EJ0078000F83C1E01F1E000F07C3F000F000787807C780F8F0078780F803C003FFF87FFE03FFF87FFF1E07E1FFFC0078000FFFC1FFFE1FFF8F03FFE000FFFC787FFF8780F8F0078780F803C003FFF83FFC01FFE07FFF1E03E1FFFC00780007FF81FFFC1FFF8F01FFC000FFFC787FFF07807CF00787807C03C003FFF80FF0007F007FFF1E03E1FFFC00780001FE01FFF81FFF8F007F0000FFFC787FFE07803FE003C7803E03Cz0z0z0rH0")
	];

static THREAD_ID: AtomicUsize = AtomicUsize::new(1);

fn main() -> Result<()> {
	Builder::new_current_thread()
		.enable_all()
		.thread_name_fn(|| {
			let id = THREAD_ID.fetch_add(1, Ordering::SeqCst) + 1;
			let output = String::from("zpl-generator-pool-");
			output + &id.to_string()
		})
		.on_thread_stop(|| {
			THREAD_ID.fetch_sub(1, Ordering::SeqCst);
		})
		.build()
		.into_diagnostic()?
		.block_on(run())
}

async fn run() -> Result<()> {
	let mut output = Vec::<TagData>::new();
	for (index, i) in RANGE.into_iter().enumerate() {
		if output.len() >= 1000 {
			paste_to_file(&mut output).await?;
		}

		output.push(TagData {
			index,
			data: generate_zpl(i),
		});
	}

	paste_to_file(&mut output).await?;

	Ok(())
}

#[derive(Debug, Clone)]
struct TagData {
	index: usize,
	data: String,
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

fn generate_zpl(num: u32) -> String {
	let mut output = DEFAULT_DATA.to_vec();
	output.extend_from_slice(&[
		Cow::Borrowed("^XA"),
		Cow::Owned(format!("^FX{num}")),
		Cow::Borrowed("^PMN"),
		Cow::Borrowed("^LRN"),
		Cow::Borrowed("^BY3,2.0"),
		Cow::Owned(format!("^FO165,542^BKN,N,60,N,N,A,B^FD{num:8}^FS")),
		Cow::Borrowed("^FO139,690^XGET0^FS"),
		Cow::Owned(format!("^FO216,790^A0N,37,53^FD{num:8}^FS")),
		Cow::Borrowed("^BY3,2.0"),
		Cow::Owned(format!("^FO165,723^BKN,N,60,N,N,A,B^FD{num:8}^FS")),
		Cow::Owned(format!("^FO216,879^A0N,37,53^FD{num:8}^FS")),
		Cow::Owned(format!("^FO216,610^A0N,37,53^FD{num:8}^FS")),
		Cow::Borrowed("^RS8,,,3,N"),
		Cow::Owned(format!("^RFW,H,4,8^FD{:8X}^FS", flip_endian(num))),
		Cow::Borrowed("^PQ1,0,1,Y"),
		Cow::Borrowed("^XZ"),
	]);
	output.join("\n")
}

async fn paste_to_file(v: &mut Vec<TagData>) -> Result<()> {
	tokio::fs::create_dir_all("./outputs")
		.await
		.into_diagnostic()?;
	let file_name = {
		let Some(first) = v.first() else {
			return Ok(());
		};

		let Some(last) = v.last() else { return Ok(()) };

		format!(
			"./outputs/Output file {}-{}.zpl",
			first.index + 1,
			last.index + 1
		)
	};
	let output = v.drain(..).map(|v| v.data).collect::<Vec<_>>().join("\n\n");
	v.clear();

	tokio::fs::write(file_name, output)
		.await
		.into_diagnostic()?;

	Ok(())
}

#[cfg(target_endian = "big")]
const fn flip_endian(s: u32) -> u32 {
	s.to_le()
}

#[cfg(target_endian = "little")]
const fn flip_endian(s: u32) -> u32 {
	s.to_be()
}
