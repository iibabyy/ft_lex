use std::{io::Read, str::Chars};

use crate::parsing::{error::{ParsingError, ParsingResult}, reader::Reader, utils::Utils};

use super::*;

pub struct Regex {
	content: String,

}

pub enum RegexType<'a> {
	Char(char),
	Number(char),
	Literal(&'a str),
	OpenBracket,
	CloseBracket,
	Hat,
	Dollar,
	Star,
	OpenBrace,
	CloseBrace,
	Comma,
	OpenParenthesis,
	CloseParenthesis,
	Dot,
	Dash,
}

impl Regex {
	pub fn new<R: Read>(expr: String) -> Self {

		

		todo!()
	}

	pub fn tokens(input: &String) -> ParsingResult<Vec<RegexType>> {

		let mut tokens = Vec::with_capacity(input.len());

		let mut chars = input.chars();

		while let Some(c) = chars.next() {

			let c_type = match c {
				c if c.is_numeric() => RegexType::Number(c),
				c if c.is_alphabetic() || c == '_' => RegexType::Char(c),
	
				'[' => RegexType::OpenBracket,
				']' => RegexType::CloseBracket,
	
				'{' => RegexType::OpenBrace,
				'}' => RegexType::CloseBrace,
	
				'(' => RegexType::OpenParenthesis,
				')' => RegexType::CloseParenthesis,
	
				'.' => RegexType::Dot,
	
				'-' => RegexType::Dash,
	
				',' => RegexType::Comma,
	
				'*' => RegexType::Star,
	
				'^' => RegexType::Hat,
	
				'$' => RegexType::Dollar,

				'\\' => {
					let c = chars.next()
						.unwrap_or('\\');

					RegexType::Char(Utils::backslashed(c as u8) as char)
				},

				'\"' => {
					let litteral = c.to_string();

					while let Some(c) = chars.next() {
						if c ==
					}
				},
	
				c => return Err(ParsingError::unexpected_token(c)),
			};

			tokens.push(c_type);
		}


		todo!()
	}

	pub fn into_type(c: char, chars: Chars<'_>) -> ParsingResult<RegexType> {


		let char_type = match c {
			c if c.is_numeric() => RegexType::Number(c),
			c if c.is_alphabetic() || c == '_' => RegexType::Char(c),

			'[' => RegexType::OpenBracket,
			']' => RegexType::CloseBracket,

			'{' => RegexType::OpenBrace,
			'}' => RegexType::CloseBrace,

			'(' => RegexType::OpenParenthesis,
			')' => RegexType::CloseParenthesis,

			':' => RegexType::Column,

			'.' => RegexType::Dot,

			'-' => RegexType::Dash,

			',' => RegexType::Comma,

			'*' => RegexType::Star,

			'^' => RegexType::Hat,

			'$' => RegexType::Dollar,

			'\\' => {
				let c = chars.next()
					.ok_or(ParsingError::end_of_file())?;

				let c = Utils::backslashed(c as u8);

				RegexType::Char(c)
			},

			c => return Err(ParsingError::unexpected_token(c)),
		};

		Ok(char_type)
	}
}