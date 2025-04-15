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
		states: Vec<State>,
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

				LineType::Rule { regex, states } => {
					todo!()
				},

				LineType::Empty => {},

				LineType::EndOfSection => return Ok(self)
			}
		}
	}

	pub fn line_type<R: Read>(reader: &mut Reader<R>) -> ParsingResult<LineType> {


		todo!()
	}
}