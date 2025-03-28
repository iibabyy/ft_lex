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

	pub fn parse(&mut self, path: String) -> ParsingResult<()> {

		let file = std::fs::File::open(path)?;

		let reader = BufReader::new(file);

		let mut lines: Lines = reader.lines().enumerate();

		while let Some((line_index, line)) = lines.next() {
			let line = line?;

			if self.is_section_delimiter(line, line_index)? {
				self.next_section();
				continue;
			}

			match self.section {
				Section::Definitions => {
					
				},

				Section::Rules => {

				},

				Section::Subroutines => {

				}
			}

		}

		todo!()
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
}