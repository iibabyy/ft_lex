use super::*;

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

#[derive(PartialEq, Eq, Hash)]
pub enum TableSizeDeclaration {
	P,
	N,
	A,
	E,
	K,
	O
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
	pub fn is_iso_C_normed(str: String) -> bool {

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

	pub fn read_until(delimiter: impl ToString, mut actual_line: String, lines: &mut Lines) -> io::Result<(Option<String>, String)> {
		let delimiter = delimiter.to_string();
		
		if let Some((res, rest)) = actual_line.split_once(&delimiter) {
			let res = Some(res.to_string());
			let rest = rest.to_string();

			return Ok((res, rest))
		}

		while let Some((_, line)) = lines.next() {
			let line = line?;
			let delimitor_len = delimiter.len();
			let saved_line_len = actual_line.len();
			
			actual_line.push_str(&line);
			
			if saved_line_len < delimitor_len {
				continue;
			}

			// to not search from the beginning at each loop
			let search_start_index = saved_line_len - delimitor_len;

			if let Some((res, rest)) = actual_line[search_start_index..].split_once(&delimiter) {
				let res = Some(res.to_string());
				let rest = rest.to_string();
	
				return Ok((res, rest))
			}
		}

		// Not found, returning the line readed
		Ok((None, actual_line))
	}

	/// Return true if found, false if not. The strings vec is all the lines readed, excluding the delimiter line if found
	/// The usize returned is the last readed line indexe, including the delimiter line if found (0 if no line readed)
	pub fn read_until_line(delimiter_line: impl ToString, lines: &mut Lines) -> io::Result<(Vec<String>, bool, usize)> {
		let delimiter_line = delimiter_line.to_string();

		let mut res = vec![];

		let mut line_index: usize = match lines.next() {
			
			// line matching delimiter
			Some((line_index, Ok(line))) if line == delimiter_line => return Ok((vec![], true, line_index)),

			// other line
			Some((line_index, Ok(line))) => {
				res.push(line);
				line_index
			},

			// i/o error while trying to read
			Some((_, Err(err))) => return Err(err),

			// end of the file (no remaining lines)
			None => return Ok((res, false, 0))
		};


		while let Some((index, line)) = lines.next() {
			let line = line?;

			if line == delimiter_line {
				return Ok((res, true, index))
			}

			line_index = index;
		}

		// Not found, returning the line readed
		Ok((res, false, line_index))
	}

}