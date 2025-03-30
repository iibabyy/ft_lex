use std::{
    io::{stdin, Bytes, Lines},
    iter::Enumerate,
    str::Chars,
};

use super::*;

#[derive(Debug)]
pub struct Reader<R: Read> {
    chars: Bytes<BufReader<R>>,

    filename: String,

    line_index: usize,

    end_of_line: bool,
}

impl<R: Read> Reader<R> {
    pub fn new(file: R, path: impl ToString) -> io::Result<Self> {
        let reader = BufReader::new(file);

        let chars = reader.bytes();

        Ok(Self {
            chars,
            filename: path.to_string(),
            line_index: 0,
            end_of_line: false,
        })
    }

    pub fn next(&mut self) -> io::Result<Option<u8>> {
        if self.end_of_line == true {
            self.line_index += 1;
            self.end_of_line = false;
        }

        if let Some(char) = self.chars.next() {
            let char = char?;

            if char == '\n' as u8 {
                self.end_of_line = true;
            }

            Ok(Some(char))
        } else {
            Ok(None)
        }
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
                    return Ok(Some(line))
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
}

pub fn reader_from_file(path: &str) -> io::Result<Reader<File>> {
    let file = File::open(&path)?;

    Reader::new(file, &path)
}

pub fn reader_from_stdin() -> io::Result<Reader<Stdin>> {
    Reader::new(stdin(), "<stdin>")
}
