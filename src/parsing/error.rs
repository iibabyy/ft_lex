/// A type alias for parsing results that can either succeed with a value of type `T` or fail with a `ParsingError`.
pub type ParsingResult<T> = Result<T, ParsingError>;

/// Represents the different types of parsing errors that can occur.
#[derive(Debug)]
pub enum ParsingErrorType {
    /// An I/O error occurred during parsing
    Io(std::io::Error),
    /// A syntax error occurred during parsing
    Syntax(String),

    UnexpectedEof(String),

    Warning(String),
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
    pub type_: ParsingErrorType,

    /// Additional error messages that provide context about the error
    pub causes: Vec<String>,
}

impl std::error::Error for ParsingError {}

impl Eq for ParsingError {}
impl PartialEq for ParsingError {
    fn eq(&self, other: &Self) -> bool {
        self.char_index == other.char_index
    }
}

impl Ord for ParsingError {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // partial_cmp always work for this struct
        self.partial_cmp(other).unwrap()
    }
}
impl PartialOrd for ParsingError {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.char_index.cmp(&other.char_index))
    }
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match &self.type_ {
            ParsingErrorType::Io(err) => err.to_string(),
            ParsingErrorType::Syntax(err) => err.to_string(),
            ParsingErrorType::UnexpectedEof(err) => err.to_string(),
            ParsingErrorType::Warning(err) => err.to_string(),
        };

        let line_and_char_index = if self.line_index.is_some() {
            let char_index = self
                .char_index
                .and_then(|index| Some(format!(":{index}")))
                .unwrap_or("".to_string());

            format!(":{}{}", self.line_index.as_ref().unwrap() + 1, char_index)
        } else {
            "".to_string()
        };

        let file = if self.file.is_some() {
            format!("{}{}", self.file.as_ref().unwrap(), line_and_char_index)
        } else {
            "".to_string()
        };

        let mut causes = String::new();

        self.causes.iter().for_each(|cause| {
            causes.push_str(": ");
            causes.push_str(cause);
        });

        write!(f, "{}: {}{}", file, message, causes)
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
            causes: Vec::new(),
        }
    }

    /// Returns the error message without file information or causes.
    pub fn message(&self) -> String {
        let base_message = match &self.type_ {
            ParsingErrorType::Io(err) => err.to_string(),
            ParsingErrorType::Syntax(err) => err.to_string(),
            ParsingErrorType::UnexpectedEof(err) => err.to_string(),
            ParsingErrorType::Warning(err) => err.to_string(),
        };
        
        if self.causes.is_empty() {
            base_message
        } else {
            let mut message = base_message;
            for cause in &self.causes {
                message.push_str(": ");
                message.push_str(cause);
            }
            message
        }
    }

    /// Creates a new syntax error with the given message.
    pub fn syntax(err: impl ToString) -> Self {
        Self {
            char_index: None,
            line_index: None,
            file: None,
            type_: ParsingErrorType::Syntax(err.to_string()),
            causes: Vec::new(),
        }
    }

    /// Creates a new syntax error with the given message.
    pub fn warning(err: impl ToString) -> Self {
        Self {
            char_index: None,
            line_index: None,
            file: None,
            type_: ParsingErrorType::Warning(err.to_string()),
            causes: Vec::new(),
        }
    }

    /// Creates a new syntax error with the given message.
    fn eof(err: impl ToString) -> Self {
        Self {
            char_index: None,
            line_index: None,
            file: None,
            type_: ParsingErrorType::UnexpectedEof(err.to_string()),
            causes: Vec::new(),
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

    /// Creates an error for an unexpected token.
    pub fn warning_unexpected_token(token: impl ToString) -> Self {
        let err = format!("unexpected token `{}`", token.to_string());
        Self::warning(err)
    }

    pub fn unrecognized_rule() -> Self {
        let err = format!("unrecognized rule");

        Self::syntax(err)
    }

    pub fn invalid_flag(token: impl ToString) -> Self {
        let err = format!("unrecognized '%' directive: `{}`", token.to_string());

        Self::syntax(err)
    }

    /// Creates an error for an unexpected end of file.
    pub fn end_of_file() -> ParsingError {
        let err: &str = "unexpected end of file";
        Self::eof(err)
    }

    /// Creates an error for an unexpected end of line.
    pub fn end_of_line() -> ParsingError {
        let err = "unexpected end of line";
        ParsingError::syntax(err)
    }

    /// Creates an error for an invalid number format.
    pub fn invalid_number(number: impl ToString) -> Self {
        let err = format!("invalid number: `{}`", number.to_string());
        ParsingError::syntax(err)
    }
}
