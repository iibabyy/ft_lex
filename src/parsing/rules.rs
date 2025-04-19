use crate::regex::normalizer::NormalizedDfa;

use super::*;

use std::io::Read;

pub const DEFAULT_STATE: &str = "INITIAL";

static mut RULE_ID: usize = 1;

pub fn rule_id() -> usize {
	unsafe { RULE_ID }
}

pub fn increment_rule_id() {
	unsafe { RULE_ID += 1 }
}

#[derive(Debug)]
pub enum LineType {
	Rule (Rule),

	Empty,
	EndOfSection,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RuleAction {
	Or,
	Statement(String)
}

#[derive(Debug)]
pub struct Rule {
	pub start_conditions: Vec<String>,

	pub regex_nfa: StatePtr,
	pub following_regex_nfa: Option<StatePtr>,

	pub action: RuleAction
}

pub struct Rules {}

impl Rules {
	pub fn parse_rules<'de, R: Read>(
        reader: &mut Reader<R>,
		definitions: &Definitions
    ) -> ParsingResult<Vec<Rule>> {
		let mut rules = vec![];

		loop {
			match Self::line_type(reader, definitions)? {

				LineType::Rule( rule ) => {
					dbg!(&rule);
					rules.push(rule);
				},

				LineType::Empty => {
					dbg!("empty line");
				},

				LineType::EndOfSection => {
					dbg!("end of section");
					return Ok(rules)
				}
			}
		}
	}

	pub fn line_type<R: Read>(
		reader: &mut Reader<R>,
		definitions: &Definitions
	) -> ParsingResult<LineType> {

		let mut first_char = if let Some(c) = reader.next()? {
			c as char
		} else {
			return Ok(LineType::EndOfSection)
		};

		if first_char == '\n' {
			return Ok(LineType::Empty)
		}

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

		let custom_conditions = first_char == '<';
		let start_conditions = Self::get_conditions(&mut first_char, reader, definitions)?;

		{	// Check if the line is empty
			if custom_conditions && first_char.is_ascii_whitespace() {
				return ParsingError::warning("empty line after start condition list").into()
			}

			if first_char == '\n' {
				return Ok(LineType::Empty)
			}
			
			if first_char.is_ascii_whitespace() {
				let line = reader.line()?;

				if line.is_none() {
					return Ok(LineType::EndOfSection)
				}

				if line.unwrap().chars().all(|c| c.is_ascii_whitespace()) {
					return Ok(LineType::Empty)
				} else {
					return ParsingError::warning("lines starting by spaces are ignored").into()
				}
			}
		}

		reader.push_front(first_char);

		let (regex, following_regex) = Self::get_regular_expression(reader)?;

		let action = Self::get_action(reader)?;

		let regex_nfa = Regex::new(regex, rule_id())?;

		let following_regex_nfa = if let Some(expr) = following_regex {
			Some(Regex::new(expr, rule_id())?)
		} else {
			None
		};

		Ok(
			LineType::Rule(Rule {
				start_conditions,
				regex_nfa,
				following_regex_nfa,
				action
			})
		)
	}

	pub fn get_action<R: Read>(
		reader: &mut Reader<R>
	) -> ParsingResult<RuleAction> {

		// skip first whitespaces
		loop {
			let peek = *reader.peek()
				.ok_or(ParsingError::end_of_file().because("missing action"))??
				as char;

			if peek.is_ascii_whitespace() {
				let _ = reader.next()?;
			} else {
				break;
			}
		}

		let c = *reader.peek()
			.ok_or(ParsingError::end_of_file().because("missing action"))??
			as char;

		let action = match c {

			'{' => {
				RuleAction::Statement(Self::read_entire_block(reader)?)
			},

			_ => {
				let line = reader.line()?
					.ok_or(ParsingError::end_of_file().because("missing action"))?;

				let trimmed = line.trim_ascii();

				if trimmed == "|" {
					RuleAction::Or
				} else {
					RuleAction::Statement(line)
				}
			}
		};

		Ok(action)
	}

	pub fn read_entire_block<R: Read>(
		reader: &mut Reader<R>
	) -> ParsingResult<String> {		
		let mut block = String::new();

		{	// check if the first char is a '{'
			let peek = *reader.peek()
				.ok_or(ParsingError::end_of_file().because("expected '{'"))??
				as char;

			if peek != '{' {
				return ParsingError::end_of_file().because("expected '{'").into()
			}
		}

		{	// read and add the '{'
			let c = reader.next()?
				.ok_or(ParsingError::end_of_file().because("expected '{'"))?
				as char;

			block.push(c);
		}

		let mut depth = 1;

		while depth > 0 {
			let c = reader.next()?
				.ok_or(ParsingError::end_of_file().because("unclosed block"))?
				as char;
			
			match c {

				'"' => {
					block.push(c);
					if let Some(content) = reader.read_until(&['"'], true)? {
						block.push_str(&content);
					} else {
						return ParsingError::end_of_file().because("unclosed quote in block").into()
					}
				},

				'\\' => {
					block.push(c);

					let c = reader.next()?
						.ok_or(ParsingError::end_of_file().because("unclosed block"))?
						as char;

					block.push(c);
				},

				'{' => {
					depth += 1;
					block.push(c);
				},

				'}' => {
					depth -= 1;
					block.push(c);
				},

				_ => {
					block.push(c);
				}
			}
		}

		Ok(block)
	}

	pub fn get_regular_expression<R: Read>(
		reader: &mut Reader<R>
	) -> ParsingResult<(String, Option<String>)> {
		
		let regex = Self::read_one_regular_expression(reader)?;

		let peek = *reader.peek()
			.ok_or(ParsingError::end_of_file().because("unclosed regular expression"))??
			as char;

		// no following regex (e.g. 'a/b')
		if peek != '/' {
			return Ok((regex, None))
		}

		// skip the '/'
		let _ = reader.next()?;

		let following_regex = Self::read_one_regular_expression(reader)?;

		let peek = *reader.peek()
			.ok_or(ParsingError::end_of_file().because("unclosed regular expression"))??
			as char;

		// duplicate '/'
		if peek == '/' {
			return ParsingError::unrecognized_rule().because("duplicate '/'").into()
		}

		Ok((regex, Some(following_regex)))
	}

	pub fn read_one_regular_expression<R: Read>(
		reader: &mut Reader<R>
	) -> ParsingResult<String> {
		let read_until = |delim: char, reader: &mut Reader<R>| -> ParsingResult<String> {
			let mut str = String::new();

			loop {	// read until the closing quote
				let c = reader.next()?
					.ok_or(ParsingError::end_of_file().because(format!("unclosed '{delim}'")))?
					as char;

				if c == '\\' {
					str.push(c);

					let next = reader.next()?
						.ok_or(ParsingError::end_of_file().because(format!("unclosed '{delim}'")))?
						as char;

					str.push(next);
				} else if c == delim {
					str.push(c);
					break;
				} else {
					str.push(c);
				}
			}

			Ok(str)
		};

		let mut regex = String::new();
		
		loop {
			let c = reader.next()?
				.ok_or(ParsingError::end_of_file().because("unclosed regular expression"))?
				as char;

			match c {
				'\"' => {
					regex.push(c);
					regex.push_str(&read_until('\"', reader)?);
				},

				'[' => {
					regex.push(c);
					regex.push_str(&read_until(']', reader)?);
				},

				'\\' => {
					regex.push(c);

					let c = reader.next()?
						.ok_or(ParsingError::end_of_file().because("unclosed regular expression"))?
						as char;

					regex.push(c);
				},

				_ => {

					// delimiter
					if c.is_ascii_whitespace() || c == '/' {
						reader.push_char(c);
						dbg!(&regex);
						return Ok(regex);
					}

					regex.push(c);
				}
			}
		}
	}

	pub fn get_conditions<R: Read>(
		first_char: &mut char,
		reader: &mut Reader<R>,
		definitions: &Definitions
	) -> ParsingResult<Vec<String>> {
		let conditions = if *first_char == '<' {
			let conditions = Self::extract_start_conditions(reader)?;

			*first_char = reader.next()?
				.ok_or(ParsingError::end_of_file().because("unclosed start condition list"))? as char;

			conditions
		} else {
			vec![DEFAULT_STATE.to_string()]
		};

		for condition in &conditions {
			if definitions.states.contains_key(condition) == false {
				return ParsingError::undeclared_start_condition(condition).into()
			}
		}

		Ok(conditions)
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