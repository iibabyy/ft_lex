use crate::regex::normalizer::NormalizedDfa;

use super::*;

use std::io::Read;

pub enum LineType {
	Rule (Rule),

	Empty,
	EndOfSection,
}

pub enum RuleAction {
	Echo,
	Begin(String),
	Reject,
	Or,
	Statement(String)
}

pub struct Rule {
	start_conditions: Vec<String>,

	regex: NormalizedDfa,
	followed_by: Option<NormalizedDfa>,

	action: RuleAction
}

pub struct Rules {
	rules: Vec<Rule>
}

impl Rules {
	pub fn parse<'de, R: Read>(
		&mut self,
        reader: &mut Reader<R>,
		definitions: &Definitions
    ) -> ParsingResult<Vec<Rule>> {
		let rules = vec![];

		loop {
			match Self::line_type(reader, definitions)? {

				LineType::Rule( rule ) => {
					todo!()
				},

				LineType::Empty => {},

				LineType::EndOfSection => return Ok(rules)
			}
		}
	}

	pub fn line_type<R: Read>(
		reader: &mut Reader<R>,
		definitions: &Definitions
	) -> ParsingResult<LineType> {

		let first_char = if let Some(c) = reader.next()? {
			c as char
		} else {
			return Ok(LineType::EndOfSection)
		};

		let second_char = reader.peek()
			.ok_or(ParsingError::end_of_file())??;

		if first_char == '%' && second_char == &b'%' {
			let _ = reader.next();

			loop {
				match reader.next()? {

					// end of line (e.g. '%%   \n')
					Some(b'\n') => return Ok(LineType::EndOfSection),

					// whitespace
					Some(c) if c.is_ascii_whitespace() => { continue; },

					// not a whitespace (e.g. '%%   a')
					Some(c) => return ParsingError::unexpected_token(c).into(),

					// eof
					None => return Ok(LineType::EndOfSection)
				}
			}
		}

		if first_char == '\n' {
			return Ok(LineType::Empty)
		} else if first_char == '\n' {
			return Ok(LineType::Empty)
		} else if first_char.is_ascii_whitespace() {
			return ParsingError::warning("lines starting by spaces are ignored").into()
		}

		let start_conditions = if first_char == '<' {
			Self::extract_start_conditions(reader)?
		} else {
			// default state
			vec!["INITIAL".to_string()]
		};

		for condition in start_conditions {
			if definitions.states.contains_key(&condition) == false {
				return ParsingError::undeclared_start_condition(condition).into()
			}
		}

		match first_char {

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
					'>' | ',' => {
						if condition.is_empty() {
							return ParsingError::bad_start_condition()
							.because("empty condition")
							.into()
						}

						if !start_conditions.contains(&condition) {
							start_conditions.push(condition);
						}

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
								.because(format!("'{c}': invalid char in start condition"))
								.because("start conditions have to be iso-C normed")
							)
						}

						if !(c.is_ascii_alphanumeric() || c == '_') {
							return Err(ParsingError::bad_start_condition()
								.because(format!("'{c}': invalid char in start condition"))
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