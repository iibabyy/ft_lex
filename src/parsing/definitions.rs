use super::*;
use std::collections::HashMap;

pub struct Definitions {
	substitute: HashMap<String, String>,
	fragment: Vec<String>,

	// Hashmap
	state: HashMap<String, bool>
}

pub enum DefinitionType {
	LineProgramFragment,
	BlockProgramFragment(String),
	NameSubstitute,
	TableSizeDeclaration(char, usize),

	// the boolean tells if it's an exclusive state (true) or not (false)
	State(bool),
}

pub enum LineType {
	Definition(String, String),
	Substitute(String, String),
	Line(String),
	BlockStart,
	BlockEnd,
	Empty
}

impl Definitions {
	pub(super) fn new() -> Self {
		Self {
			substitute: HashMap::new(),
			fragment: Vec::new(),
			state: HashMap::new(),
		}
	}

	pub(super) fn add(&mut self, line: String, lines: &mut Lines) -> ParsingResult<LineType> {

	}

	fn line_type(line: String, line_index: usize) -> ParsingResult<LineType> {

		if line.is_empty() {
			return Ok(LineType::Empty)
		}

		let mut chars = line.chars();

		let first_char = chars.next().unwrap();

		if first_char == ' ' {
			let res = line[1..].trim().split_ascii_whitespace();

			let name = res.next();
			if name.is_none() {
				return Err(ParsingError::end_of_line(line_index).because("Definition starting by a space, expected: {name} {substitution}"))
			}

			let substitution = res.next();
			if substitution.is_none() {
				return Err(ParsingError::end_of_line(line_index).because("Definition starting by a space, expected: {name} {substitution}"))
			}

			if let Some(invalid_token) = res.next() {
				return Err(ParsingError::unexpected_token(invalid_token, line_index, char_index))
			}

			return Ok(LineType::Substitute((), ()))
		}

		todo!()
	}

	/*
	
	fn definition_type(line: String, line_index: usize, lines: &mut Lines) -> ParsingResult<DefinitionType> {

		if line.is_empty() {
			return Err(ParsingError::unexpected_token("end of line", line_index, 0));
		}

		let mut chars = line.chars();

		let first_char = chars.next().unwrap();

		// ' ' + anything -> Program Fragment (1 line)
		if first_char == ' ' {
			return Ok(DefinitionType::LineProgramFragment)
		}

		if first_char == '%' {
			if line.len() < 2 {
				return Err(ParsingError::unexpected_token("end of line", line_index, 1));
			}

			let next_char = chars.next().unwrap();

			if !Self::is_valid_description_flag(next_char) {
				return Err(ParsingError::unexpected_token(next_char, line_index, 1));
			}

			// Definition Parsing
			let res = match next_char {
				// %{ -> Program Fragment until '%}' delimiter (can be multiple lines)
				'{' => {
					// Line not finished (block fragment are delimited by "%{\n" and "%}\n")
					if let Some(char) = chars.next() {
						return Err(ParsingError::
							unexpected_token(char, line_index, 2)
							.because("the block content should not be on the same line as the delimiter")
						)
					}

					let (content, found, index) = Utils::read_until_line("%}", lines)?;

					if found == false {
						return Err(ParsingError::end_of_file(index))
					}

					Ok(DefinitionType::BlockProgramFragment(content.join("\n")))
				},

				// % + one of (p, n, a, e, k, o) -> Table Size Declaration
				'p' | 'n' | 'a' | 'e' | 'k' | 'o' => {
					let letter = next_char;
					let next_char = chars.next();

					if let Some(char) = next_char {
						if !char.is_whitespace() {
							// char after the letter is invalid (not whitespace)
							return Err(ParsingError::
								unexpected_token(char, line_index, 2)
								.because("the block content should not be on the same line as the delimiter")
							)
						}
					} else {
						// end of line
						return Err(ParsingError::
							unexpected_token("end of line", line_index, 2)
							.because("positive number expected")
						)
					}

					let content = line[3..].trim();
					let number = content.parse::<usize>()
						.map_err(|err| ParsingError::
							invalid_number(content, line_index)
							.because(err.to_string())
					)?;

					Ok(DefinitionType::TableSizeDeclaration(letter, number))
				},

				// % + one of (s, S) -> State
				's' | 'S' => Ok(DefinitionType::State(false)),

				// % + one of (x, X) -> Exclusive State
				'x' | 'X' => Ok(DefinitionType::State(true)),

				// % + other chars (invalid)
				invalid_char => Err(ParsingError::unexpected_token(invalid_char, line_index, 1))
			};

			match chars.next() {
				Some(' ') => { /* OK */ },
				None => return Err(ParsingError::unexpected_token("end of line", line_index, 2)),
				Some(invalid_char) => return Err(ParsingError::unexpected_token(invalid_char, line_index, 2))
			}

			return res
		}

	 */
	/// check if char after % is a valid Description Section flag
	fn is_valid_description_flag(c: char) -> bool {
		// Program Fragment
		if c == '{' {
			return true;
		}

		// Table Size Declaration
		if c == 'p' ||	c == 'n' ||	c == 'a' ||	c == 'e' ||	c == 'k' ||	c == 'o' {
			return true;
		}

		// State
		if c == 's' || c == 'S' || c == 'x' || c == 'X' {
			return true;
		}

		false
	}
}