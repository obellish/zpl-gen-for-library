mod tracing_setup;

#[cfg(not(feature = "proof"))]
use std::ops::RangeInclusive;
use std::{
	borrow::{Borrow, Cow},
	io::Result as IoResult,
	sync::atomic::{AtomicUsize, Ordering::SeqCst},
};

use anyhow::Result;
use futures::TryFutureExt;
use tokio::{fs, runtime::Builder, time::Instant};
use tracing::{event, Level};

use self::tracing_setup::setup_tracing;

#[cfg(not(any(feature = "proof", debug_assertions)))]
const RANGE: RangeInclusive<u32> = 5_000_020..=5_050_019;
// const RANGE: RangeInclusive<u32> = 5_000_000..=5_000_019;
#[cfg(all(not(feature = "proof"), debug_assertions))]
const RANGE: RangeInclusive<u32> = 5_000_000..=5_000_019;
#[cfg(feature = "proof")]
const RANGE: [u32; 2] = [5_000_020, 5_050_019];
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
	Builder::new_multi_thread()
		.thread_name_fn(|| {
			let id = THREAD_ID.fetch_add(1, SeqCst) + 1;
			let output = String::from("zpl-generator-pool-");
			output + &id.to_string()
		})
		.on_thread_stop(|| {
			THREAD_ID.fetch_sub(1, SeqCst);
		})
		.build()?
		.block_on(run())
}

async fn run() -> Result<()> {
	setup_tracing()?;
	let timer = Instant::now();
	event!(Level::DEBUG, "removing outputs dir");
	_ = fs::remove_dir_all("./outputs").await;
	fs::create_dir_all("./outputs").await?;
	let mut futures = Vec::new();
	let mut output = Vec::<TagData>::with_capacity(1000);
	for (index, i) in RANGE.into_iter().enumerate() {
		if output.len() >= 1000 {
			let final_output = std::mem::replace(&mut output, Vec::with_capacity(1000));
			futures.push(tokio::spawn(
				async move { paste_to_file(final_output).await },
			));
		}

		output.push(TagData {
			index,
			data: generate_zpl(i),
		});
	}

	futures.push(tokio::spawn(async move { paste_to_file(output).await }));

	futures::future::try_join_all(futures)
		.map_ok(|values| values.into_iter().collect::<IoResult<()>>())
		.await??;

	let time = Instant::now().duration_since(timer);

	event!(Level::INFO, "took {time:?}");

	Ok(())
}

#[derive(Debug, Clone)]
struct TagData {
	index: usize,
	data: String,
}

impl Borrow<str> for TagData {
	fn borrow(&self) -> &str {
		&self.data
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

fn generate_zpl(num: u32) -> String {
	event!(Level::TRACE, %num, "generating zpl");
	let raw_num_string = num.to_string();
	let mut num_bytes = raw_num_string.as_bytes().to_vec();

	let check_digit_byte = check_digit::digit_checksum(&num_bytes);
	num_bytes.extend(check_digit_byte);
	let num = String::from_utf8(num_bytes).unwrap();
	let mut output = DEFAULT_DATA.to_vec();
	output.extend_from_slice(&[
		Cow::Borrowed("^XA"),
		Cow::Owned(format!("^FX{num}")),
		Cow::Borrowed("^PMN"),
		Cow::Borrowed("^LRN"),
		Cow::Borrowed("^BY3,2.0"),
		Cow::Owned(format!("^FO190,542^BKN,N,60,N,N,A,B^FD{num:0>8}^FS")),
		Cow::Borrowed("^FO164,690^XGET0^FS"),
		Cow::Owned(format!("^FO241,790^A0N,37,53^FD{num:0>8}^FS")),
		Cow::Borrowed("^BY3,2.0"),
		Cow::Owned(format!("^FO190,723^BKN,N,60,N,N,A,B^FD{num:0>8}^FS")),
		Cow::Owned(format!("^FO241,879^A0N,37,53^FD{num:0>8}^FS")),
		Cow::Owned(format!("^FO241,610^A0N,37,53^FD{num:0>8}^FS")),
		Cow::Borrowed("^RS8,,,3,N"),
		Cow::Owned(format!(
			"^RFW,H,4,8^FD{:0>8X}^FS",
			flip_endian(num.parse().unwrap())
		)),
		Cow::Borrowed("^PQ1,0,1,Y"),
		Cow::Borrowed("^XZ"),
	]);
	output.join("\n")
}

#[allow(clippy::unused_async)]
#[tracing::instrument(skip_all)]
async fn paste_to_file<I>(v: I) -> IoResult<()>
where
	I: IntoIterator<Item = TagData> + Send,
{
	let v = v.into_iter().collect::<Vec<_>>();
	let file_name = {
		let Some(first) = v.first() else {
			return Ok(());
		};

		let Some(last) = v.last() else {
			return Ok(());
		};

		format!(
			"./outputs/Output file {}-{}.zpl",
			first.index + 1,
			last.index + 1
		)
	};
	event!(Level::INFO, %file_name, "writing output");
	let output = v.join("\n\n");

	fs::write(file_name, output).await?;

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

mod check_digit {
	fn fold10_swar(mask1: u64, mask2: u64, raw: &[u8]) -> Option<u64> {
		let mut sum = 0;

		for c in raw.rchunks(8) {
			let mut buf = [b'0'; 8];
			copy_from_small_slice(&mut buf, c);

			let mut v = u64::from_le_bytes(buf);

			let a = v.wrapping_add(0x4646_4646_4646_4646);

			v = v.wrapping_sub(0x3030_3030_3030_3030);

			if (a | v) & 0x8080_8080_8080_8080 == 0 {
				sum += u64::from((mask2.wrapping_sub(v) & 0x8080_8080_8080_8080).count_ones());
				sum += v.wrapping_mul(mask1) >> 56;
			} else {
				return None;
			}
		}

		Some(sum)
	}

	fn copy_from_small_slice(buf: &mut [u8; 8], c: &[u8]) {
		match c.len() {
			8 => *buf = <[u8; 8]>::try_from(c).unwrap(),
			7 => buf[1..].copy_from_slice(&<[u8; 7]>::try_from(c).unwrap()),
			6 => buf[2..].copy_from_slice(&<[u8; 6]>::try_from(c).unwrap()),
			5 => buf[3..].copy_from_slice(&<[u8; 5]>::try_from(c).unwrap()),
			4 => buf[4..].copy_from_slice(&<[u8; 4]>::try_from(c).unwrap()),
			3 => buf[5..].copy_from_slice(&<[u8; 3]>::try_from(c).unwrap()),
			2 => buf[6..].copy_from_slice(&<[u8; 2]>::try_from(c).unwrap()),
			1 => buf[7..].copy_from_slice(&<[u8; 1]>::try_from(c).unwrap()),
			_ => unreachable!(),
		}
	}

	pub fn digit_checksum(digits: &[u8]) -> Option<u8> {
		let sum = fold10_swar(0x0102_0102_0102_0102, 0x047f_047f_047f_047f, digits)?;
		Some(b'0' + ((10 - (sum % 10)) % 10) as u8)
	}
}
