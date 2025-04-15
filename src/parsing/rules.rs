use crate::regex::normalizer::NormalizedDfa;

use super::*;

use std::io::Read;

pub enum RuleAction {
	Echo,
	Begin(String),
	Reject,
	Or,
	Statement(String)
}

pub enum LineType {
	Rule {
		regex: NormalizedDfa,
		start_conditions: Vec<String>,
	},

	Empty,
	EndOfSection
}

pub struct Rules {}

impl Rules {
	pub fn parse<'de, R: Read>(
        &'de mut self,
        reader: &mut Reader<R>,
    ) -> ParsingResult<&'de mut Self> {

		loop {
			match Self::line_type(reader)? {

				LineType::Rule { regex, start_conditions } => {
					todo!()
				},

				LineType::Empty => {},

				LineType::EndOfSection => return Ok(self)
			}
		}
	}

	pub fn line_type<R: Read>(reader: &mut Reader<R>) -> ParsingResult<LineType> {

		let first_char = reader.next()?
			.ok_or(ParsingError::end_of_file())?
			as char;

		if first_char == '<' {
			
		}

		match first_char {

			'<' => todo!(),

			_ => todo!()
		}

		todo!()
	}

	pub fn extract_start_conditions<R: Read>(reader: &mut Reader<R>) -> ParsingResult<Vec<String>> {

		let mut start_conditions = vec![];

		'_big_loop: loop {
			let mut condition = String::new();

			'little_loop: loop {
				let c = reader.next()?
					.ok_or(ParsingError::end_of_file().because("unclosed start condition list"))?
					as char;

				match c {
					c if c.is_ascii_whitespace() => return ParsingError::end_of_line()
						.because("unclosed start condition list")
						.into(),

					'>' | ',' => {
						if condition.is_empty() {
							return ParsingError::bad_start_condition()
							.because("empty condition")
							.into()
						}

						start_conditions.push(condition);

						if c == '>' {
							return Ok(start_conditions);
						} else {
							break 'little_loop;
						}
					},

					_ => {
						// valid first char (alphabetic or '_')
						if condition.is_empty() && !(c.is_ascii_alphabetic() || c == '_') {
							return Err(ParsingError::bad_start_condition()
								.because(format!("'{c}': invalid first char'"))
								.because("start conditions have to be iso-C normed")
							)
						}

						condition.push(c);
					}
				}
			}
		}

		if start_conditions.is_empty() {
			return ParsingError::bad_start_condition()
				.because("empty condition")
				.into()
		}

		Ok(start_conditions)
	}
}