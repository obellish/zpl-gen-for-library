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
