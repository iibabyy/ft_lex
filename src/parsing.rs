/// A module for parsing lexer definitions and configurations.
mod definitions;
mod error;
pub mod utils;

use super::*;
use definitions::*;
use error::*;
use utils::*;

use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read, Stdin},
    os::unix::fs::FileExt,
    path::PathBuf,
};

/// Type alias for an iterator over file lines with line numbers
type Lines<R> = std::iter::Enumerate<io::Lines<BufReader<R>>>;

/// A reader that provides line-by-line access to a file or stdin with position tracking.
pub struct Reader<R: Read> {
    /// The path to the file being read, or "<stdin>" for stdin
    path: PathBuf,

    /// Iterator over the lines of the input
    lines: Lines<R>,

    /// The current line being processed
    line: Option<String>,

    /// The current line number (0-based)
    index: usize,
}

/// Trait defining the interface for file readers in the lexer.
pub trait FReader {
    /// Returns the path of the file being read
    fn path(&self) -> &PathBuf;
    /// Returns a mutable reference to the path
    fn path_mut(&mut self) -> &mut PathBuf;
    /// Returns the next line from the input
    fn next(&self) -> io::Result<Option<&String>>;
    /// Returns the current line
    fn line(&self) -> Option<&String>;
    /// Returns a mutable reference to the current line
    fn line_mut(&mut self) -> Option<&mut String>;
    /// Returns the current line number
    fn index(&self) -> usize;
}

impl<R: Read> Reader<R> {
    /// Creates a new reader from an input source and path.
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

    /// Advances the reader to the next line and returns a reference to it.
    /// Returns `None` when the end of input is reached.
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

/// Creates a reader from a file path.
pub fn reader_from_file(file_path: impl Into<PathBuf>) -> io::Result<Reader<File>> {
    let path = file_path.into();
    let file = File::open(&path)?;
    Ok(Reader::new(file, path))
}

/// Creates a reader from stdin.
pub fn reader_from_stdin() -> Reader<io::Stdin> {
    let stdin = io::stdin();
    Reader::new(stdin, PathBuf::from("<stdin>"))
}

/// The main parsing structure that handles the lexer definition parsing process.
pub struct Parsing {
    /// Collection of lexer definitions (substitutions, fragments, etc.)
    definitions: Definitions,

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
        loop {
            match self.section {
                Section::Definitions => {
                    // Parse the definitions section (substitutions, fragments, etc.)
                    self.definitions.parse(reader)?;
                    
                    // Move to the rules section after definitions are parsed
                    self.next_section();
                }
                Section::Rules => {
                    // TODO: Implement rules section parsing
                    eprintln!("TODO: Rules Section");
                    return Ok(());
                }
                Section::Subroutines => {
                    // TODO: Implement subroutines section parsing
                    eprintln!("TODO: Subroutines Section");
                    return Ok(());
                }
            }
        }
        // Note: This loop only exits through the return statements in the Rules and Subroutines branches
        // Once the Definitions branch has completed, it loops back and enters the Rules branch
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
            let char_index = 2;
            let char = line.chars().nth(char_index).unwrap();
            return Err(ParsingError::unexpected_token(char).line(line_index).char(char_index));
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
