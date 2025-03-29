use super::*;
mod regex;
use regex::*;
mod definitions;
use definitions::*;
mod error;
use error::*;
pub mod utils;
use utils::*;

use std::{fs::File, io::{self, BufRead, BufReader}};

type Lines = std::iter::Enumerate<io::Lines<BufReader<File>>>;

pub struct Parsing {
	definitions: Definitions,

	arg: Vec<Option<String>>,

	lines: Lines,

	line: Option<String>,

	line_number: usize,
	
	// section actually being parsed
	section: Section
}

// Sections (in order)
#[derive(PartialEq, Eq)]
pub enum Section {
	Definitions,
	Rules,
	Subroutines
}

impl Section {
	fn next(&self) -> Self {
		match self {
			Section::Definitions => Section::Rules,
			Section::Rules => Section::Subroutines,

			// Last Section (normally don't call this function at this state)
			Section::Subroutines => Section::Subroutines
		}
	}
}

impl Parsing {
	pub fn new() -> io::Result<Self> {
		Ok(
			Self {
				definitions: Definitions::new(),
				section: Section::Definitions
			}
		)
	}

	fn next_section(&mut self) {
		self.section = self.section.next();
	}

	pub fn parse(&mut self, args: Vec<Option<String>>) -> ParsingResult<()> {

		for arg in args {
			let path = arg.unwrap();

			let file = std::fs::File::open(path)?;

			let reader = BufReader::new(file);

			let mut lines: Lines = reader.lines().enumerate();

			loop {
				match self.section {
					Section::Definitions => { self.definitions.parse(&mut lines)?; eprintln!("{:#?}", self.definitions) },

					Section::Rules => {
						eprintln!("TODO: Rules Section");
						return Ok(())
					},

					Section::Subroutines => {
						eprintln!("TODO: Subroutines Section");
						return Ok(())
					}
				}
			}
		}
	}

	fn parse_definition(line: String, lines: Lines) -> ParsingResult<()> {



		todo!()
	}

	fn is_section_delimiter(&self, line: String, line_index: usize) -> ParsingResult<bool> {
		if line.starts_with("%%") == false {
			return Ok(false)
		}
		
		if self.section == Section::Subroutines {
			return Err(ParsingError::unexpected_token("%%", line_index, 0));
		}

		if line.len() > 2 {
			let char_index = 2;
			let char = line.chars().nth(char_index).unwrap();

			return Err(ParsingError::unexpected_token(char, line_index, char_index))
		}

		Ok(true)
	}

	fn next_line(&mut self) -> io::Result<Option<String>> {
		if let Some((line_index, line)) = self.lines.next() {

		} else {

		}
	}
}