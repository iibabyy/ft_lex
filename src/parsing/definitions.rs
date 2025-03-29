use super::*;
use std::{collections::{HashMap, HashSet}, mem::{self, take}};

pub struct Definitions {
	substitutes: HashMap<String, String>,
	fragments: Vec<String>,

	type_declaration: Option<TypeDeclaration>,

	table_sizes: HashMap<TableSizeDeclaration, usize>,

	states: HashMap<String, bool>
}

pub enum DefinitionType {
	TableSize(TableSizeDeclaration, usize),
	Substitute(String, String),
	Fragment(String),
	TypeDeclaration(TypeDeclaration),
	Empty,
	EndOfSection
}

impl Definitions {
	pub(super) fn new() -> Self {
		Self {
			substitutes: HashMap::new(),
			fragments: Vec::new(),
			states: HashMap::new(),
			table_sizes: HashMap::new(),
			type_declaration: None
		}
	}

	pub(super) fn parse(&mut self, lines: &mut Lines) -> ParsingResult<&mut Self> {

		let mut saved_index = 0;

		while let Some((index, line)) = lines.next() {
			saved_index = index;
			let line = line?;

			match Self::line_type(line, index, lines)? {

				// Table Size ('%{letter} {size}')
				DefinitionType::TableSize(table, size) => { self.table_sizes.insert(table, size); },
				
				// Substitute ('{name} {substitute}')
				DefinitionType::Substitute(name, substitute) => { self.substitutes.insert(name, substitute); },
				
				// Fragment (' {Program fragment}' or '%{\n{Program fragment}\n%}')
				DefinitionType::Fragment(fragment) => { self.fragments.push(fragment); },
				
				// Type of yytext ('%array' or '%pointer')
				DefinitionType::TypeDeclaration(type_decla) => { self.type_declaration = Some(type_decla) },
				
				// Empty line
				DefinitionType::Empty => { },

				// End of Definition section
				DefinitionType::EndOfSection => return Ok(self),
			}
		}

		return Err(ParsingError::end_of_file(saved_index));
	}

	fn line_type(line: String, line_index: usize, lines: &mut Lines) -> ParsingResult<DefinitionType> {

		if line.is_empty() {
			return Ok(DefinitionType::Empty)
		}

		if line == "%%" {
			return Ok(DefinitionType::EndOfSection);
		}

		let mut chars = line.chars();

		let first_char = chars.next().unwrap();

		// Line Program Fragment
		if first_char == ' ' {
			return Ok(DefinitionType::Fragment(line[1..].to_string()))
		}

		// Substitution Chains
		if first_char.is_alphabetic() || first_char == '_' {
			let mut split = Self::split_definition(&line, 2, line_index, "{name} {substitute}")?;
			
			return Ok(DefinitionType::Substitute(
				take(&mut split[0]),	// name
				take(&mut split[1])	// substitute
			));
		}

		// Block Program Fragments
		if line == "%{" {
			let (content, found, last_index) = Utils::read_until_line("%}", lines)?;

			if !found {
				return Err(ParsingError::
					end_of_file(last_index)
					.because(format!("expected close matching delimiter for open delimiter at line {line_index}"))
				);
			}

			let content = content.join("\n");

			return Ok(DefinitionType::Fragment(content))
		}

		// Only possibility left is '%', Syntax error else
		if first_char != '%' {
			return Err(ParsingError::unexpected_token(first_char, line_index, 0));
		}

		let mut split: Vec<String> = line[1..].split_ascii_whitespace().map(|str| str.to_string()).collect();

		// empty
		if split.is_empty() {
			return Ok(DefinitionType::Empty)
		}

		let flag = take(&mut split[0]);

		match flag.as_str() {
			"p" | "n" | "a" | "e" |"k" | "o" => {
				Self::check_split_size(&split, 2, line_index, "{flag} {positive number}")?;

				let size = split[1].as_str().parse::<usize>()
					.map_err(|err|ParsingError::
						invalid_number(&split[1], line_index)
						.because(format!("{err}"))
					)?;
				
				let char = flag.chars().next().unwrap();

				return Ok(DefinitionType::TableSize(TableSizeDeclaration::try_from(char).unwrap(), size));
			},

			"array" | "pointer" => {
				Self::check_split_size(&split, 1, line_index, "{type}")?;

				return Ok(DefinitionType::TypeDeclaration(TypeDeclaration::try_from(flag).unwrap()))
			}

			&_ => return Err(ParsingError::unexpected_token(flag, line_index, 1))
		}
	}

	fn split_definition(line: &String, expected: usize, line_index: usize, expected_err_msg: impl ToString) -> ParsingResult<Vec<String>> {
		let split: Vec<String> = line.split_ascii_whitespace().map(|str| str.to_string()).collect();
		let expected_err_msg = expected_err_msg.to_string();

		Self::check_split_size(&split, expected, line_index, expected_err_msg)?;

		Ok(split)
	}

	pub(super) fn check_split_size(split: &Vec<String>, expected: usize, line_index: usize, expected_err_msg: impl ToString) -> ParsingResult<()> {
		let expected_err_msg = expected_err_msg.to_string();
		
		if split.len() < expected {
			return Err(ParsingError::end_of_line(line_index).because(format!("expected: {expected_err_msg}")))
		}

		if split.len() > expected {
			return Err(ParsingError::unexpected_token_in_line(&split[2], line_index).because(format!("expected: {expected_err_msg}")))
		}

		Ok(())
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