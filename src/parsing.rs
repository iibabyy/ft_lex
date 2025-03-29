use super::*;
mod regex;
use regex::*;
mod definitions;
use definitions::*;
mod error;
use error::*;
pub mod utils;
use utils::*;

use std::{fs::File, io::{self, BufRead, BufReader, Read, Stdin}, os::unix::fs::FileExt, path::PathBuf};

type Lines<R> = std::iter::Enumerate<io::Lines<BufReader<R>>>;

pub struct Reader<R: Read> {
	path: PathBuf,

	lines: Lines<R>,

	line: Option<String>,

	index: usize,
}

pub trait FReader {
	fn path(&self) -> &PathBuf;
	fn path_mut(&mut self) -> &mut PathBuf;
	fn next(&self) -> io::Result<Option<&String>>;
	fn line(&self) -> Option<&String>;
	fn line_mut(&mut self) -> Option<&mut String>;
	fn index(&self) -> usize;
}

impl<R: Read> Reader<R> {
	fn new(reader: R, path: PathBuf) -> Reader<R> {
        let buf_reader = BufReader::new(reader);
        let lines = io::BufRead::lines(buf_reader).enumerate();
        
        Reader {
            path,
            lines,
            line: None,
            index: 0,
        }
    }

	pub fn next(&mut self) -> io::Result<Option<&String>> {
		if let Some((index, line)) = self.lines.next() {
			self.line = Some(line?);
			self.index = index;

			Ok(self.line.as_ref())
		} else {
			self.line = None;
			Ok(None)
		}
	}
}

pub fn reader_from_file(file_path: impl Into<PathBuf>) -> io::Result<Reader<File>> {
	let path = file_path.into();
	let file = File::open(&path)?;

	Ok(Reader::new(file, path))
}

pub fn reader_from_stdin() -> Reader<io::Stdin> {
	let stdin = io::stdin();

	Reader::new(stdin, PathBuf::from("<stdin>"))
}

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

	pub fn parse(&mut self, config: &Config) -> ParsingResult<()> {

		let mut iter = config.args.iter().map(|arg| arg.as_ref());

		while let Some(arg) = iter.next() {
			if let Some(path) = arg {
				self.parse_section(&mut reader_from_file(path)?)?;
			} else {
				self.parse_section(&mut reader_from_stdin())?;
			}
		}

		Ok(())
	}

	fn parse_section<R: Read>(&mut self, reader: &mut Reader<R>) -> ParsingResult<()> {

		loop {
			match self.section {
				Section::Definitions => { self.definitions.parse(reader)?; dbg!(&self.definitions); },

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