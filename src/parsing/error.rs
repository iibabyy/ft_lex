#[derive(Debug)]
pub enum ParsingError{
	Io(std::io::Error),
	Syntax(String)
}

impl std::fmt::Display for ParsingError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ParsingError::Io(err) => write!(f, "{}", err.to_string()),
			ParsingError::Syntax(err) => write!(f, "{}", err),
		}
	}
}

impl std::error::Error for ParsingError {}

pub type ParsingResult<T> = Result<T, ParsingError>;


impl From<std::io::Error> for ParsingError {
	fn from(error: std::io::Error) -> Self {
		ParsingError::Io(error)
	}
}

impl ParsingError {
	pub fn unexpected_token(token: impl ToString, line_index: usize, char_index: usize) -> Self {
		ParsingError::Syntax(format!(
			"lexer.{}:{} unexpected token '{}'" ,
			line_index + 1, char_index + 1,
			token.to_string()
		))
	}

	pub fn unexpected_token_in_line(token: impl ToString, line_index: usize) -> Self {
		ParsingError::Syntax(format!(
			"lexer.{} unexpected token '{}'" ,
			line_index + 1,
			token.to_string()
		))
	}

	pub fn end_of_file(line_index: usize) -> ParsingError {
		ParsingError::Syntax(format!(
			"lexer.{} unexpected end of file",
			line_index
		))
	}

	pub fn end_of_line(line_index: usize) -> ParsingError {
		ParsingError::Syntax(format!(
			"lexer.{} unexpected newline",
			line_index
		))
	}

	pub fn invalid_number(number: impl ToString, line_index: usize) -> Self {
		ParsingError::Syntax(format!(
			"lexer.{} invalid number: {}",
			line_index, number.to_string()
		))
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