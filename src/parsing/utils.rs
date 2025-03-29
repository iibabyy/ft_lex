use std::mem::take;

use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TypeDeclaration {
	Array,
	Pointer
}

impl ToString for TypeDeclaration {
	fn to_string(&self) -> String {
		match self {
			TypeDeclaration::Array => "array",
			TypeDeclaration::Pointer => "pointer",
		}.to_string()
	}
}

impl TryFrom<String> for TypeDeclaration {
	type Error = ();

	fn try_from(value: String) -> Result<Self, Self::Error> {
		match value.as_str() {
			"array" => Ok(Self::Array),
			"pointer" => Ok(Self::Pointer),
			_ => Err(())
		}
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TableSizeDeclaration {
	P,
	N,
	A,
	E,
	K,
	O
}

impl ToString for TableSizeDeclaration {
	fn to_string(&self) -> String {
		match self {
			TableSizeDeclaration::P => "p",
			TableSizeDeclaration::N => "n",
			TableSizeDeclaration::A => "a",
			TableSizeDeclaration::E => "e",
			TableSizeDeclaration::K => "k",
			TableSizeDeclaration::O => "o",
		}.to_string()
	}
}

impl TableSizeDeclaration {
	pub fn try_from(letter: impl ToString) -> Result<Self, ()> {
		let letter = letter.to_string();
		if letter.is_empty() {
			return Err(())
		}

		let letter = letter.chars().next().unwrap();
		match letter {
			'p' => Ok(TableSizeDeclaration::P),
			'n' => Ok(TableSizeDeclaration::N),
			'a' => Ok(TableSizeDeclaration::A),
			'e' => Ok(TableSizeDeclaration::E),
			'k' => Ok(TableSizeDeclaration::K),
			'o' => Ok(TableSizeDeclaration::O),

			_ => Err(())
		}
	}
}

pub struct Utils {}

impl Utils {

	#[allow(non_snake_case)]
	pub fn is_iso_C_normed(str: &str) -> bool {

		if str.is_empty() {
			return false;
		}

		let mut chars = str.chars();

		let first_char = chars.next().unwrap();

		// check first char is alphabetic or '_'
		if !first_char.is_alphabetic() && first_char != '_' {
			return false;
		}

		// check str contains only alphanums and '_'
		if chars.any(|char| !char.is_alphanumeric() && char != '_') {
			return false;
		}

		true
	}

	pub fn read_until<R: Read>(delimiter: impl ToString, actual_line: Option<String>, reader: &mut Reader<R>) -> io::Result<(Option<String>, String)> {
		let delimiter = delimiter.to_string();

		if actual_line.is_some() {
			// Check if delimiter alreaady present
			if let Some((res, rest)) = actual_line.as_ref().unwrap().split_once(&delimiter) {
				let res = Some(res.to_string());
				let rest = rest.to_string();
	
				return Ok((res, rest))
			}
		}

		let mut save = actual_line.unwrap_or(String::new());

		while let Some(line) = reader.next()? {
			let delimitor_len = delimiter.len();
			let saved_line_len = save.len();

			save.push_str(&line);
			
			if saved_line_len < delimitor_len {
				continue;
			}

			// to not search from the beginning at each loop
			let search_start_index = saved_line_len - delimitor_len;

			if let Some((res, rest)) = save[search_start_index..].split_once(&delimiter) {
				let res = Some(res.to_string());
				let rest = rest.to_string();
	
				return Ok((res, rest))
			}
		}

		// Not found, returning the line readed
		Ok((None, save))
	}

	/// Return true if found, false if not. The strings vec is all the lines readed, excluding the delimiter line if found
	/// The usize returned is the last readed line indexe, including the delimiter line if found (0 if no line readed)
	pub fn read_until_line<R: Read>(delimiter_line: impl ToString, reader: &mut Reader<R>) -> io::Result<(Vec<String>, bool)> {
		let delimiter_line = delimiter_line.to_string();

		let mut res = vec![];

		match reader.next()? {

			// line matching delimiter
			Some(line) if line == &delimiter_line => return Ok((vec![], true)),

			// other line
			Some(_) => res.push(take(reader.line.as_mut().unwrap())),

			// end of the file (no remaining lines)
			None => return Ok((res, false))
		};

		while let Some(line) = reader.next()? {
			// line matching delimiter
			if line == &delimiter_line {
				return Ok((res, true))
			}

			// taking and replacing the reader's last line
			res.push(take(reader.line.as_mut().unwrap()));
		}

		// setting the reader's last line to the last readed line
		reader.line = res.last().cloned();

		// Not found, returning the lines readed
		Ok((res, false))
	}

	pub fn split_whitespace_once(str: &str) -> Option<(&str, &str)> {
		let index = str.find(|c: char| c.is_whitespace())?;

		let (str1, mut str2) = str.split_at(index);

		str2 = str2.trim_ascii_start();

		Some((str1, str2))
	}

}