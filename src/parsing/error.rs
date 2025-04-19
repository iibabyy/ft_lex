use std::{fmt::Display, io::Read};

use super::Reader;

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

    /// The type of error that occurred
    pub type_: ParsingErrorType,

    /// Additional error messages that provide context about the error
    pub causes: Vec<String>,
}

impl<T> Into<ParsingResult<T>> for ParsingError {
    fn into(self) -> ParsingResult<T> {
        Err(self)
    }
}

impl std::error::Error for ParsingError {}

impl Eq for ParsingError {}
impl PartialEq for ParsingError {
    fn eq(&self, other: &Self) -> bool {
        self.line_index == other.line_index
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
        Some(self.line_index.cmp(&other.line_index))
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

            &format!("{}:", self.line_index.as_ref().unwrap() + 1)
        } else {
            ""
        };

        let file = if self.file.is_some() {
            &format!("{}:{} ", self.file.as_ref().unwrap(), line_and_char_index)
        } else {
            ""
        };

        let mut causes = String::new();

        self.causes.iter().for_each(|cause| {
            causes.push_str(": ");
            causes.push_str(cause);
        });

        if matches!(self, ParsingError{ type_: ParsingErrorType::Warning(_), .. }) {
            write!(f, "WARNING: {}{}{}", file, message, causes)
        } else {
            write!(f, "{}: {}{}", file, message, causes)
        }
    }
}

impl From<std::io::Error> for ParsingError {
    fn from(error: std::io::Error) -> Self {
        ParsingError::io(error)
    }
}

impl From<&std::io::Error> for ParsingError {
    fn from(error: &std::io::Error) -> Self {
        let err = std::io::Error::from(error.kind().clone());
        ParsingError::io(err)
    }
}

impl ParsingError {
    /// Creates a new parsing error from an I/O error.
    pub fn io(err: std::io::Error) -> Self {
        Self {
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
            line_index: None,
            file: None,
            type_: ParsingErrorType::Syntax(err.to_string()),
            causes: Vec::new(),
        }
    }

    /// Creates a new syntax error with the given message.
    pub fn warning(err: impl ToString) -> Self {
        Self {
            line_index: None,
            file: None,
            type_: ParsingErrorType::Warning(err.to_string()),
            causes: Vec::new(),
        }
    }

    /// Creates a new syntax error with the given message.
    fn eof(err: impl ToString) -> Self {
        Self {
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

    pub fn bad_start_condition() -> ParsingError {
        let cause = "bad start condition";

        ParsingError::syntax(cause)
    }

    pub fn undeclared_start_condition(condition: impl ToString) -> ParsingError {
        ParsingError::syntax(format!("undeclared start condition: `{}`", condition.to_string()))
    }

    /// Creates an error for an invalid number format.
    pub fn invalid_number(number: impl ToString) -> Self {
        let err = format!("invalid number: `{}`", number.to_string());
        ParsingError::syntax(err)
    }

	pub fn actual_line_number<R: Read>(mut self, reader: &Reader<R>) -> Self {
		self.line_index = Some(reader.index());
		self
	}

    pub fn undefined_definition(definition: impl Display) -> Self {
        let message = format!("undefined definition: {{{definition}}}");

        ParsingError::syntax(message)
    }
}
