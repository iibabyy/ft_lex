use std::{
    collections::VecDeque,
    io::{stdin, Bytes, Lines},
    iter::{Enumerate, Peekable},
    str::Chars,
};

use super::*;

#[derive(Debug)]
pub struct Reader<R: Read> {
    chars: Peekable<Bytes<BufReader<R>>>,

    filename: String,

    line_index: usize,

    end_of_line: bool,

    rest: VecDeque<char>,

	peek: Option<u8>,
}

impl<R: Read> Reader<R> {
    pub fn new(file: R, path: impl ToString) -> io::Result<Self> {
        let reader = BufReader::new(file);

        let chars = reader.bytes().peekable();

        Ok(Self {
            chars,
            filename: path.to_string(),
            line_index: 0,
            end_of_line: false,
            rest: VecDeque::new(),
			peek: None,
        })
    }

    pub fn next(&mut self) -> io::Result<Option<u8>> {
        if self.end_of_line == true {
            self.line_index += 1;
            self.end_of_line = false;
        }

        let c = if let Some(c) = self.rest.pop_front() {
            c
        } else if let Some(c) = self.chars.next() {
            c? as char
        } else {
            return Ok(None);
        };

        if c == '\n' {
            self.end_of_line = true;
        }

        Ok(Some(c as u8))
    }

    pub fn line(&mut self) -> io::Result<Option<String>> {
        let mut line = String::new();

        loop {
            if let Some(char) = self.next()? {
                if char == '\n' as u8 {
                    return Ok(Some(line));
                }
                line.push(char as char);

            } else {
                if line.is_empty() == false {
                    return Ok(Some(line));
                }

                return Ok(None);
            }
        }
    }

    // /// returns the last readed line
    // pub fn char(&self) -> Option<&u8> {
    //     self.readed_chars.last()
    // }

    // Returns the index of the line
    pub fn index(&self) -> usize {
        self.line_index
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn push_str(&mut self, s: &str) {
        self.rest.extend(s.chars());
    }

    pub fn push_char(&mut self, c: char) {
        self.rest.push_back(c);
    }

    pub fn peek(&mut self) -> Option<Result<&u8, &io::Error>> {
		if self.rest.is_empty() {
			self.chars.peek().and_then(|res| Some(res.as_ref()))
		} else {
			self.peek = Some(*self.rest.front().unwrap() as u8);

			Some(Ok(self.peek.as_ref().unwrap()))
		}
    }
}

pub fn reader_from_file(path: &str) -> io::Result<Reader<File>> {
    let file = File::open(&path)?;

    Reader::new(file, &path)
}

pub fn reader_from_stdin() -> io::Result<Reader<Stdin>> {
    Reader::new(stdin(), "<stdin>")
}
