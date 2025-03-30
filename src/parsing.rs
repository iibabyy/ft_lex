/// A module for parsing lexer definitions and configurations.
mod definitions;
mod error;
mod reader;
pub mod utils;

use super::*;
use definitions::*;
use error::*;
use utils::*;
use reader::*;

use std::{
    fs::File, io::{self, BufRead, BufReader, Read, Stdin}, mem::take, os::unix::fs::FileExt, path::PathBuf
};

/// The main parsing structure that handles the lexer definition parsing process.
pub struct Parsing {
    /// Collection of lexer definitions (substitutions, fragments, etc.)
    pub definitions: Definitions,

    pub errors: Vec<ParsingError>,

    /// The current section being parsed
    section: Section,
}

/// Represents the different sections of a lexer definition file.
#[derive(PartialEq, Eq)]
pub enum Section {
    /// The definitions section containing substitutions, fragments, and declarations
    Definitions,
    /// The rules section containing pattern-action pairs
    Rules,
    /// The subroutines section containing C code
    Subroutines,
}

impl Section {
    /// Returns the next section in sequence.
    /// Note: Calling this on `Subroutines` will return `Subroutines` again.
    fn next(&self) -> Self {
        match self {
            Section::Definitions => Section::Rules,
            Section::Rules => Section::Subroutines,
            Section::Subroutines => Section::Subroutines,
        }
    }
}

impl Parsing {
    /// Creates a new parsing instance with empty definitions.
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            definitions: Definitions::new(),
            errors: Vec::new(),
            section: Section::Definitions,
        })
    }

    /// Parses the lexer definition files specified in the config.
    /// 
    /// This function handles both file inputs and stdin, processing each section
    /// (definitions, rules, subroutines) in sequence.
    pub fn parse(&mut self, config: &Config) -> ParsingResult<()> {
        // Create an iterator over the config arguments
        let mut iter = config.args.iter().map(|arg| arg.as_ref());

        while let Some(arg) = iter.next() {
            if let Some(path) = arg {
                // For file input, create a reader and parse with file context
                self.parse_section(&mut reader_from_file(path)?)
                    .map_err(|err| err.file(path))?;
            } else {
                // For stdin input, create a reader and parse with stdin context
                self.parse_section(&mut reader_from_stdin())
                    .map_err(|err| err.file("<stdin>"))?;
            }
        }

        Ok(())
    }

    /// Parses a single section of the lexer definition file.
    /// 
    /// This function handles the parsing of each section (definitions, rules, subroutines)
    /// and advances to the next section when appropriate.
    fn parse_section<R: Read>(&mut self, reader: &mut Reader<R>) -> ParsingResult<()> {
        'big_loop: loop {
            match self.section {
                Section::Definitions => {
                    // Parse the definitions section (substitutions, fragments, etc.)
                    while let Err(err) = self.definitions.parse(reader) {
                        self.errors.push(
                            err.file(reader.path.to_string_lossy().to_string())
                        );

                        match self.errors.last().unwrap().type_ {
                            ParsingErrorType::Io(_) => break 'big_loop,

                            // To parse all the file even if there is a syntax error
                            _ => {}
                        }
                    }

                    // Move to the rules section after definitions are parsed
                    self.next_section();
                }
                Section::Rules => {
                    // TODO: Implement rules section parsing
                    eprintln!("TODO: Rules Section");
                    break;
                }
                Section::Subroutines => {
                    // TODO: Implement subroutines section parsing
                    eprintln!("TODO: Subroutines Section");
                    break;
                }
            }
        }

        if self.errors.is_empty() == false {
            self.errors.sort();
            self.errors.iter().for_each(|err| eprintln!("{}", err.to_string()));
            return Err(
                std::mem::replace(&mut self.errors[0], ParsingError::syntax("foo"))
            )
        }

        Ok(())
    }

    /// Checks if a line is a section delimiter ("%%").
    /// 
    /// Returns an error if:
    /// - The line starts with "%%" but contains more than "%%"
    /// - The delimiter appears in the subroutines section
    fn is_section_delimiter(&self, line: String, line_index: usize) -> ParsingResult<bool> {
        // Check if the line starts with the section delimiter
        if line.starts_with("%%") == false {
            return Ok(false);
        }

        // Section delimiters are not allowed in the subroutines section (which is the last section)
        if self.section == Section::Subroutines {
            return Err(ParsingError::unexpected_token("%%").line(line_index).char(0));
        }

        // Ensure the delimiter is just "%%" with no extra characters
        if line.len() > 2 {
			let (unexpected_token, _) = Utils::split_whitespace_once(&line).unwrap_or((line.as_str(), ""));
            return Err(ParsingError::unexpected_token(unexpected_token).line(line_index));
        }

        // Valid section delimiter
        Ok(true)
    }

    /// Advances to the next section in the lexer definition file.
    fn next_section(&mut self) {
        // Move from Definitions -> Rules -> Subroutines (in that order)
        self.section = self.section.next();
    }
}
