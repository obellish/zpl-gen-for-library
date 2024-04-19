use std::cmp::Ordering;

use self::natural::HumanString;

mod natural;

pub trait NaturalSort {
	fn natural_sort(&mut self);
}

impl<'a> NaturalSort for [&'a str] {
	fn natural_sort(&mut self) {
		self.sort_by(|a, b| sorter(a, b).unwrap_or(Ordering::Equal));
	}
}

impl NaturalSort for [String] {
	fn natural_sort(&mut self) {
		self.sort_by(|a, b| sorter(a, b).unwrap_or(Ordering::Equal));
	}
}

fn sorter(a: &str, b: &str) -> Option<Ordering> {
	let a = HumanString::new(a)?;
	let b = HumanString::new(b)?;

	a.partial_cmp(&b)
}
