pub type ParsingResult<T> = Result<T, ParsingError>;

#[derive(Debug)]
pub enum ParsingErrorType {
    Io(std::io::Error),
    Syntax(String),
}

#[derive(Debug)]
pub struct ParsingError {
    file: Option<String>,
    line_index: Option<usize>,
    char_index: Option<usize>,
    type_: ParsingErrorType,

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
                ".{}:{}",
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
    pub fn io(err: std::io::Error) -> Self {
        Self {
            char_index: None,
            line_index: None,
            file: None,

            type_: ParsingErrorType::Io(err),

            causes: Vec::new()
        }
    }

    pub fn syntax(err: impl ToString) -> Self {
        Self {
            char_index: None,
            line_index: None,
            file: None,

            type_: ParsingErrorType::Syntax(err.to_string()),

            causes: Vec::new()
        }
    }

    pub fn file(mut self, file: String) -> Self {
        self.file = Some(file);
        
        self
    }
    
    pub fn line(mut self, line_index: usize) -> Self {
        self.line_index = Some(line_index);
        
        self
    }
    
    pub fn char(mut self, char_index: usize) -> Self {
        self.char_index = Some(char_index);
        
        self
    }

    pub fn because(mut self, msg: impl ToString) -> Self {
        let msg = msg.to_string();

        self.causes.push(msg.to_string());

        self
    }

}

impl ParsingError {

    pub fn unexpected_token(token: impl ToString) -> Self {
        let err = format!("unexpected token `{}`", token.to_string());

        Self::syntax(err)
    }

    pub fn end_of_file(line_index: usize) -> ParsingError {
        let err = "unexpected end of file";

        Self::syntax(err).line(line_index)
    }

    pub fn end_of_line(line_index: usize) -> ParsingError {
        let err = "unexpected end of line";

        ParsingError::syntax(err).line(line_index)
    }

    pub fn invalid_number(number: impl ToString) -> Self {
        let err = format!("invalid number: {}", number.to_string());
        
        ParsingError::syntax(err)
    }

}
