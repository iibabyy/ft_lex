/// A type alias for parsing results that can either succeed with a value of type `T` or fail with a `ParsingError`.
pub type ParsingResult<T> = Result<T, ParsingError>;

/// Represents the different types of parsing errors that can occur.
#[derive(Debug)]
pub enum ParsingErrorType {
    /// An I/O error occurred during parsing
    Io(std::io::Error),
    /// A syntax error occurred during parsing
    Syntax(String),
}

/// A structured error type for parsing operations that includes context about where and why the error occurred.
#[derive(Debug)]
pub struct ParsingError {
    /// The file where the error occurred, if applicable
    file: Option<String>,

    /// The line number where the error occurred, if applicable
    line_index: Option<usize>,

    /// The character position where the error occurred, if applicable
    char_index: Option<usize>,

    /// The type of error that occurred
    type_: ParsingErrorType,

    /// Additional error messages that provide context about the error
    causes: Vec<String>
}

impl std::error::Error for ParsingError {}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match &self.type_ {
            ParsingErrorType::Io(err) => err.to_string(),
            ParsingErrorType::Syntax(err) => err.to_string(),
        };

        let line_and_char_index = if self.line_index.is_some() {
            let char_index = self.char_index
                .and_then(|index| Some(format!(":{index}")))
                .unwrap_or("".to_string());

            format!(
                ":{}{}",
                self.line_index.as_ref().unwrap() + 1,
                char_index
            )
        } else {
            "".to_string()
        };

        let file = if self.file.is_some() {
            format!(
                "{}{}",
                self.file.as_ref().unwrap(),
                line_and_char_index
            )
        } else {
            "".to_string()
        };

        let mut causes = String::new();

        self.causes.iter().for_each(|cause| {
            causes.push_str(": ");
            causes.push_str(cause);
        });

        write!(f,
            "{} : {}{}",
            file, message, causes
        )
    }
}

impl From<std::io::Error> for ParsingError {
    fn from(error: std::io::Error) -> Self {
        ParsingError::io(error)
    }
}

impl ParsingError {
    /// Creates a new parsing error from an I/O error.
    pub fn io(err: std::io::Error) -> Self {
        Self {
            char_index: None,
            line_index: None,
            file: None,
            type_: ParsingErrorType::Io(err),
            causes: Vec::new()
        }
    }

    /// Creates a new syntax error with the given message.
    pub fn syntax(err: impl ToString) -> Self {
        Self {
            char_index: None,
            line_index: None,
            file: None,
            type_: ParsingErrorType::Syntax(err.to_string()),
            causes: Vec::new()
        }
    }

    /// Adds file context to the error.
    pub fn file(mut self, file: impl ToString) -> Self {
        self.file = Some(file.to_string());
        self
    }
    
    /// Adds line number context to the error.
    pub fn line(mut self, line_index: usize) -> Self {
        self.line_index = Some(line_index);
        self
    }
    
    /// Adds character position context to the error.
    pub fn char(mut self, char_index: usize) -> Self {
        self.char_index = Some(char_index);
        self
    }

    /// Adds an additional error message to provide more context about the error.
    pub fn because(mut self, msg: impl ToString) -> Self {
        let msg = msg.to_string();
        self.causes.push(msg.to_string());
        self
    }

    /// Creates an error for an unexpected token.
    pub fn unexpected_token(token: impl ToString) -> Self {
        let err = format!("unexpected token `{}`", token.to_string());
        Self::syntax(err)
    }

    /// Creates an error for an unexpected end of file.
    pub fn end_of_file(line_index: usize) -> ParsingError {
        let err = "unexpected end of file";
        Self::syntax(err).line(line_index)
    }

    /// Creates an error for an unexpected end of line.
    pub fn end_of_line(line_index: usize) -> ParsingError {
        let err = "unexpected end of line";
        ParsingError::syntax(err).line(line_index)
    }

    /// Creates an error for an invalid number format.
    pub fn invalid_number(number: impl ToString) -> Self {
        let err = format!("invalid number: `{}`", number.to_string());
        ParsingError::syntax(err)
    }
}
