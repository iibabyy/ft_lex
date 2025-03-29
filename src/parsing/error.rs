pub type ParsingResult<T> = Result<T, ParsingError>;

#[derive(Debug)]
pub enum ParsingError {
    Io(std::io::Error),
    Syntax(String),
}

impl std::error::Error for ParsingError {}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError::Io(err) => write!(f, "{}", err.to_string()),
            ParsingError::Syntax(err) => write!(f, "{}", err),
        }
    }
}

impl Into<String> for ParsingError {
    fn into(self) -> String {
        match self {
            ParsingError::Io(err) => err.to_string(),
            ParsingError::Syntax(err) => err.to_string()
        }
    }

    // fn to_string(&self) -> String {
    //     match self {
    //         ParsingError::Io(err) => err.to_string(),
    //         ParsingError::Syntax(err) => err.to_string()
    //     }
    // }
}

impl From<std::io::Error> for ParsingError {
    fn from(error: std::io::Error) -> Self {
        ParsingError::Io(error)
    }
}

impl ParsingError {
    pub fn unexpected_token(token: impl ToString, line_index: usize, char_index: usize) -> Self {
        ParsingError::Syntax(format!(
            "lexer.{}:{} unexpected token '{}'",
            line_index + 1,
            char_index + 1,
            token.to_string()
        ))
    }

    pub fn syntax(err: impl ToString, line_index: usize) -> Self {
        ParsingError::Syntax(format!("{}: {}", line_index + 1, err.to_string()))
    }

    pub fn unexpected_token_in_line(token: impl ToString, line_index: usize) -> Self {
        ParsingError::Syntax(format!(
            "{} unexpected token '{}'",
            line_index + 1,
            token.to_string()
        ))
    }

    pub fn end_of_file(line_index: usize) -> ParsingError {
        ParsingError::Syntax(format!("{} unexpected end of file", line_index + 1))
    }

    pub fn end_of_line(line_index: usize) -> ParsingError {
        ParsingError::Syntax(format!("{} unexpected newline", line_index + 1))
    }

    pub fn invalid_number(number: impl ToString, line_index: usize) -> Self {
        ParsingError::Syntax(format!(
            "{} invalid number: {}",
            line_index + 1,
            number.to_string()
        ))
    }
	
    pub fn from_file(self, file: impl ToString) -> Self {
        let file = file.to_string();

        match self {
            // adding a cause to the error message
            ParsingError::Syntax(err) => ParsingError::Syntax(format!("{}:{}", file, err)),

            // We don't change error message
            ParsingError::Io(err) => ParsingError::Io(err),
        }
    }

    pub fn because(self, msg: impl ToString) -> Self {
        let msg = msg.to_string();

        match self {
            // adding a cause to the error message
            ParsingError::Syntax(err) => ParsingError::Syntax(format!("{}: {}", err, msg)),

            // We don't change error message
            ParsingError::Io(err) => ParsingError::Io(err),
        }
    }
}
