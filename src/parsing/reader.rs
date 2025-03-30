use super::*;


/// A reader that provides line-by-line access to a file or stdin with position tracking.
pub struct Reader<R: Read> {
    /// The path to the file being read, or "<stdin>" for stdin
    pub(super) path: PathBuf,

    /// Iterator over the lines of the input
    reader: BufReader<R>,

    /// The current line being processed
    pub(super) line: Option<String>,

    /// The current line number (0-based)
    pub(super) index: usize,
}

impl<R: Read> Reader<R> {
    /// Creates a new reader from an input source and path.
    fn new(reader: R, path: PathBuf) -> Reader<R> {
        let reader = BufReader::new(reader);

        Reader {
            path,
            reader,
            line: None,
            index: 0,
        }
    }

    /// Read one byte, and convert it to a char.
    /// 
    /// Returns `None` when the end of input is reached.
    pub fn next(&mut self) -> io::Result<Option<char>> {
        let mut buf = [0u8; 1];

        // reading one byte into buf
        if let Err(err) = self.reader.read_exact(&mut buf) {
            if err.kind() == io::ErrorKind::UnexpectedEof {
                return Ok(None)
            }

            return Err(err);
        };

        // converting buf into char
        Ok(Some(char::from(buf[0])))
    }

    /// Read until '\n' and returns a reference to the string readed.
    /// 
    /// Returns `None` when the end of input is reached.
    /// 
    /// If reader.next() has been call on the beginning on the line, this method will returns the remaining part of the line
    pub fn line(&mut self) -> io::Result<Option<&String>> {
        let mut line = String::new();

        self.reader.read_line(&mut line)?;
        if line.ends_with('\n') {
            self.index += 1;
            line.pop();
        }
        self.line = Some(line);

        Ok(Some(self.line.as_ref().unwrap()))
    }

    /// Returns the last readed line
    pub fn last_line(&self) -> Option<&String> {
        self.line.as_ref()
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
