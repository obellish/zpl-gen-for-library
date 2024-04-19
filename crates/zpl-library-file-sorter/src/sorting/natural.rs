use std::cmp::Ordering;

use num::BigInt;
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
enum StringElement<'a> {
	Letters(&'a str),
	Number(BigInt),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct HumanString<'a>(Vec<StringElement<'a>>);

static NUMBERS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d+").unwrap());
static LETTERS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[^\d]+").unwrap());

impl<'a> HumanString<'a> {
	pub fn new(s: &'a str) -> Option<Self> {
		let mut elements = Vec::new();
		let mut to_parse = s;

		while !to_parse.is_empty() {
			let (next_element, next_to_parse) = Self::process_token(to_parse)?;

			elements.push(next_element);
			to_parse = next_to_parse;
		}

		Some(Self(elements))
	}

	fn process_token(to_parse: &'a str) -> Option<(StringElement<'a>, &'a str)> {
		if let Some(numbers_match) = NUMBERS_RE.find(to_parse).map(|m| m.end()) {
			Self::process_number(numbers_match, to_parse)
		} else {
			let letters_match = LETTERS_RE.find(to_parse).map(|m| m.end())?;
			Some(Self::process_letters(letters_match, to_parse))
		}
	}

	fn process_number(end_index: usize, to_parse: &'a str) -> Option<(StringElement<'a>, &'a str)> {
		let prefix_to_num = to_parse[..end_index].parse::<BigInt>().ok()?;

		let next_token = StringElement::Number(prefix_to_num);
		let to_parse_suffix = &to_parse[end_index..];

		Some((next_token, to_parse_suffix))
	}

	fn process_letters(end_index: usize, to_parse: &'a str) -> (StringElement<'a>, &'a str) {
		let prefix = &to_parse[..end_index];

		let next_token = StringElement::Letters(prefix);
		let to_parse_suffix = &to_parse[end_index..];

		(next_token, to_parse_suffix)
	}
}

impl<'a> PartialOrd for HumanString<'a> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		let pairs = self.0.iter().zip(other.0.iter());
		let compares = pairs.map(|pair| match pair {
			(StringElement::Number(ref a), StringElement::Number(ref b)) => a.partial_cmp(b),
			(StringElement::Letters(ref a), StringElement::Letters(ref b)) => a.partial_cmp(b),
			_ => None,
		});

		for comparison in compares {
			if !matches!(comparison, Some(Ordering::Equal)) {
				return comparison;
			}
		}

		self.0.len().partial_cmp(&other.0.len())
	}
}
