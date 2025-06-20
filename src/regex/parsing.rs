use std::collections::{HashSet, VecDeque};
use std::iter::Peekable;
use std::str::Chars;
use std::fmt;

use crate::*;

// ==============================================
// 1. TYPE DEFINITIONS
// ==============================================

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RegexType {
    Char(char),
	CharacterClass(CharacterClass),
    LineStart,
    LineEnd,
    OpenParenthesis,
    CloseParenthesis,
    Or,
    Concatenation,
    Quant(Quantifier),
}

// Wrapper for RegexType to be used in the conversion to postfix
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum TokenType {
    Literal(RegexType),
    OpenParenthesis(RegexType),
    CloseParenthesis(RegexType),
    UnaryOperator(RegexType),
    BinaryOperator(RegexType),
    StartOrEndCondition(RegexType),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CharacterClass {
    // Individual characters in the class
    pub chars: Vec<char>,
    pub negated: bool
}

impl fmt::Display for CharacterClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let chars = self.chars
			.iter()
			.map(|c| if c == &'\n' { "\\n".to_string() } else { c.to_string() })
			.collect::<String>();

        if self.negated {
            write!(f, "[^{}]", chars)
        } else {
            write!(f, "[{}]", chars)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Quantifier {
    // {n}
    Exact(usize),
    // {n,}
    AtLeast(usize),
    // {n,m}
    Range(usize, usize),
}

// ==============================================
// 2. REGEXTYPE IMPLEMENTATIONS
// ==============================================

// RegexType implementations
impl RegexType {
    pub fn precedence(&self) -> usize {
        match self {
            RegexType::Quant(_) => 3,
            RegexType::Concatenation => 2,
            RegexType::Or => 1,
            _ => 0,
        }
    }

    pub fn type_(&self) -> TokenType {
        match self {
            // Opening group
            RegexType::OpenParenthesis => TokenType::OpenParenthesis(self.clone()),

            // Closing group
            RegexType::CloseParenthesis => TokenType::CloseParenthesis(self.clone()),

            // One element operator
            RegexType::Quant(_) => TokenType::UnaryOperator(self.clone()),

            // Two element operator
            RegexType::Or | RegexType::Concatenation => TokenType::BinaryOperator(self.clone()),

            // start or end of line conditions
            RegexType::LineStart | RegexType::LineEnd => {
                TokenType::StartOrEndCondition(self.clone())
            }

            _ => TokenType::Literal(self.clone()),
        }
    }

    pub fn match_(&self, c: &char) -> bool {
        match self {
            RegexType::Char(char) => char == c,
            _ => todo!(),
        }
    }
    
    pub fn char(&self) -> Option<char> {
        match self {
            RegexType::Char(c) => Some(*c),
            _ => None,
        }
    }

	pub fn class(&self) -> Option<&CharacterClass> {
		match self {
			RegexType::CharacterClass(cc) => Some(cc),
			_ => None,
		}
	}
}

impl fmt::Display for RegexType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegexType::Char(c) => write!(f, "{}", c),
            RegexType::CharacterClass(cc) => write!(f, "{}", cc),
            RegexType::LineStart => write!(f, "^"),
            RegexType::LineEnd => write!(f, "$"),
            RegexType::OpenParenthesis => write!(f, "("),
            RegexType::CloseParenthesis => write!(f, ")"),
            RegexType::Or => write!(f, "|"),
            RegexType::Concatenation => write!(f, "&"),
            RegexType::Quant(q) => write!(f, "{}", q),
        }
    }
}

// ==============================================
// 3. TOKENTYPE IMPLEMENTATIONS
// ==============================================

// TokenType implementations
impl TokenType {
    /// Converts a TokenType back to its inner RegexType
    pub fn into_inner(&self) -> &RegexType {
        match self {
            TokenType::Literal(rt) => rt,
            TokenType::OpenParenthesis(rt) => rt,
            TokenType::CloseParenthesis(rt) => rt,
            TokenType::UnaryOperator(rt) => rt,
            TokenType::BinaryOperator(rt) => rt,
            TokenType::StartOrEndCondition(rt) => rt,
        }
    }

    /// Converts a TokenType back to its inner RegexType
    pub fn into_owned_inner(self) -> RegexType {
        match self {
            TokenType::Literal(rt) => rt,
            TokenType::OpenParenthesis(rt) => rt,
            TokenType::CloseParenthesis(rt) => rt,
            TokenType::UnaryOperator(rt) => rt,
            TokenType::BinaryOperator(rt) => rt,
            TokenType::StartOrEndCondition(rt) => rt,
        }
    }
    
    pub fn need_concatenation_with(&self, other: &RegexType) -> bool {
        match (self, other.type_()) {
            // Literal followed by literal or opening parenthesis
            (
				TokenType::Literal(_),
				TokenType::Literal(_) | TokenType::OpenParenthesis(_)
			) => true,

            // Closing parenthesis followed by literal/opening parenthesis
            (
                TokenType::CloseParenthesis(_),
                TokenType::Literal(_) | TokenType::OpenParenthesis(_),
            ) => true,

            // Unary operator followed by literal/opening parenthesis
            (
                TokenType::UnaryOperator(_),
                TokenType::Literal(_) | TokenType::OpenParenthesis(_),
            ) => true,

            _ => false,
        }
    }

    pub fn precedence(&self) -> usize {
        match self {
            Self::Literal(rt) => rt.precedence(),
            Self::OpenParenthesis(rt) => rt.precedence(),
            Self::CloseParenthesis(rt) => rt.precedence(),
            Self::UnaryOperator(rt) => rt.precedence(),
            Self::BinaryOperator(rt) => rt.precedence(),
            Self::StartOrEndCondition(rt) => rt.precedence(),
        }
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(rt) => write!(f, "{}", rt),
            Self::OpenParenthesis(rt) => write!(f, "{}", rt),
            Self::CloseParenthesis(rt) => write!(f, "{}", rt),
            Self::UnaryOperator(rt) => write!(f, "{}", rt),
            Self::BinaryOperator(rt) => write!(f, "{}", rt),
            Self::StartOrEndCondition(rt) => write!(f, "{}", rt),
        }
    }
}

impl From<RegexType> for TokenType {
    fn from(value: RegexType) -> Self {
        match value {
            // Opening group
            RegexType::OpenParenthesis => TokenType::OpenParenthesis(value),

            // Closing group
            RegexType::CloseParenthesis => TokenType::CloseParenthesis(value),

            // One element operator
            RegexType::Quant(_) => TokenType::UnaryOperator(value),

            // Two element operator
            RegexType::Or | RegexType::Concatenation => TokenType::BinaryOperator(value),

            // start or end of line conditions
            RegexType::LineStart | RegexType::LineEnd => TokenType::StartOrEndCondition(value),

            _ => TokenType::Literal(value),
        }
    }
}

// ==============================================
// 4. CHARACTER CLASS IMPLEMENTATIONS
// ==============================================

// CharacterClass implementations
impl CharacterClass {
    pub fn new() -> Self {
        Self {
            chars: Vec::new(),
            negated: false
        }
    }

    pub fn all() -> Self {
        let mut chars = Vec::with_capacity(127);

        for char in 0..=127_u8 {
            chars.push(char as char);
        }

        Self {
            chars,
            negated: false
        }
    }

    pub fn add_char(&mut self, c: char) {
        if self.chars.contains(&c) == false {
            self.chars.push(c);
        }
    }

    // Private method to remove a character from singles
    fn remove_char(&mut self, c: char) {
		let search = self.chars
			.iter()
			.enumerate()
			.find_map(|(index, char)|
				(char == &c).then_some(index)
			);

        if let Some(index) = search {
			self.chars.remove(index);
		}
    }

    pub fn add_range(&mut self, start: char, end: char) -> ParsingResult<()> {
        if start <= end {
            for c in start..=end {
                self.add_char(c);
            }

            Ok(())
        } else {
            ParsingError::unrecognized_rule().because("negative range in character class").into()
        }
    }

    // Parse a character class from a string
    pub fn parse(chars: &mut Peekable<Chars>) -> ParsingResult<Self> {
        let mut class = Self::new();
        let mut prev_char: Option<char> = None;

        // Check for negation
        if let Some('^') = chars.clone().next() {
            class.negated = true;
            chars.next(); // Consume the '^'
        }

        while let Some(c) = chars.next() {
            match c {
                ']' => {
                    return Ok(class);
                }
                '-' => {
                    // Range definition
                    if let Some(start) = prev_char {
                        if let Some(end) = chars.next() {
                            if end == ']' {
                                // '-' at the end is a literal character
                                class.add_char('-');
                                return Ok(class);
                            } else {
                                class.remove_char(start); // Remove the start char as it's now part of a range
                                class.add_range(start, end)?;
                                prev_char = None;
                            }
                        } else {
                            return ParsingError::unrecognized_rule()
                                .because("Unclosed character class")
                                .into();
                        }
                    } else {
                        // '-' at the beginning is a literal character
                        class.add_char('-');
                    }
                }
                '\\' => {
                    // Handle escape sequences
                    if let Some(next_c) = chars.next() {
                        let interpreted = Utils::backslashed(next_c);
                        class.add_char(interpreted);
                        prev_char = Some(interpreted);
                    } else {
                        return ParsingError::unrecognized_rule()
                            .because("Escape sequence at end of character class")
                        .into();                    }
                }

                c => {
                    class.add_char(c);
                    prev_char = Some(c);
                }
            }
        }

        ParsingError::unrecognized_rule()
            .because("Unclosed character class")
            .into()
    }

    // Compatibility methods to create instances
    pub fn from_single(c: char) -> Self {
        let mut class = Self::new();
        class.add_char(c);
        class
    }

    pub fn from_range(start: char, end: char) -> ParsingResult<Self> {
        let mut class = Self::new();
        class.add_range(start, end)?;
        Ok(class)
    }

    pub fn from_negated(other: CharacterClass) -> Self {
        let mut class = other;
        class.negated = true;
        class
    }

    // Create a character class from a shorthand character class like \d, \w, etc.
    pub fn from_shorthand(c: char) -> ParsingResult<Self> {
        match c {
            'd' => Ok(Self::digit()),
            'D' => Ok(Self::non_digit()),
            'w' => Ok(Self::word_char()),
            'W' => Ok(Self::non_word_char()),
            's' => Ok(Self::whitespace()),
            'S' => Ok(Self::non_whitespace()),
            _ => Err(ParsingError::unrecognized_rule()
                .because(&format!("Unknown shorthand class '\\{}'", c))),
        }
    }

    // Predefined character classes
    pub fn digit() -> Self {
        let mut class = Self::new();
        let _ = class.add_range('0', '9');
        class
    }

    pub fn negated(mut self) -> Self {
        self.negated = true;
        self
    }

    pub fn non_digit() -> Self {
        Self::digit().negated()
    }

    pub fn word_char() -> Self {
        let mut class = Self::new();
        let _ = class.add_range('a', 'z');
        let _ = class.add_range('A', 'Z');
        let _ = class.add_range('0', '9');
        class.add_char('_');
        class
    }

    pub fn non_word_char() -> Self {
        Self::word_char().negated()
    }

    pub fn whitespace() -> Self {
        let mut class = Self::new();
        // Add all whitespace characters
        for c in [9_u8, 10, 11, 12, 13, 32] {
            class.add_char(c as char);
        }
        class
    }

    pub fn non_whitespace() -> Self {
        Self::whitespace().negated()
    }

    // Check if a character matches this character class
    pub fn contains(&self, c: &char) -> bool {
        if self.negated {
            !self.chars.contains(c)
        } else {
            self.chars.contains(c)
        }
    }

	pub fn len(&self) -> usize {
		if self.negated {
			128 - self.chars.len()
		} else {
			self.chars.len()
		}
	}

	pub fn chars(&self) -> Vec<char> {
		if self.negated {
			let mut chars = Vec::with_capacity(128);
			for c in 0..=127_u8 {
				if !self.chars.contains(&(c as char)) {
					chars.push(c as char);
				}
			}

			chars
		} else {
			self.chars.clone()
		}
	}
}

// ==============================================
// 5. QUANTIFIER IMPLEMENTATIONS
// ==============================================

// Quantifier implementations
impl fmt::Display for Quantifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Quantifier::Exact(n) => write!(f, "{{{}}}", n),
            Quantifier::AtLeast(n) => write!(f, "{{{},}}", n),
            Quantifier::Range(min, max) => write!(f, "{{{},{}}}", min, max),
        }
    }
}

// ==============================================
// 6. REGEX PARSING METHODS
// ==============================================

impl Regex {
    pub fn add_concatenation(tokens: VecDeque<RegexType>) -> VecDeque<TokenType> {
        if tokens.len() < 2 {
            return tokens.into_iter().map(TokenType::from).collect();
        }

        let mut result: VecDeque<TokenType> = VecDeque::with_capacity(tokens.len() * 2);
        let mut tokens_iter = tokens.into_iter().peekable();

        // Process first token
        if let Some(token) = tokens_iter.next() {
            result.push_back(TokenType::from(token));

            // Process remaining tokens
            while let Some(next_token) = tokens_iter.next() {
                let last = result.back().unwrap();
                let current = &next_token;

                // Check if concatenation is needed
                if last.need_concatenation_with(current) {
                    result.push_back(TokenType::from(RegexType::Concatenation));
                }

                result.push_back(TokenType::from(next_token));
            }
        }

        result
    }

    pub fn tokens(input: &str) -> ParsingResult<VecDeque<RegexType>> {
        let mut tokens = VecDeque::with_capacity(input.len());
        let mut chars = input.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '"' => Self::add_string(&mut tokens, &mut chars)?,

				'[' => Self::add_character_class(&mut tokens, &mut chars)?,

				'{' => Self::add_quantifier(&mut tokens, &mut chars)?,


				'\\' => Self::add_backslash(&mut tokens, &mut chars),

				'.' => tokens.push_back(RegexType::CharacterClass(CharacterClass::from_single('\n').negated())),

				'^' => {
					if tokens.is_empty() {
						// if at the start of the string -> line start
						tokens.push_back(RegexType::LineStart);
					} else {
						tokens.push_back(RegexType::Char('^'));
					}
				},

				'$' => {
					if chars.peek().is_none() {
						// if at the end of the string -> line end
						tokens.push_back(RegexType::LineEnd);
					} else {
						tokens.push_back(RegexType::Char('$'));
					}
				},

                c => tokens.push_back(Self::into_type(c)),
            }
        }

        Ok(tokens)
    }

    pub fn add_backslash(
        tokens: &mut VecDeque<RegexType>,
        chars: &mut Peekable<Chars<'_>>,
    ) {
        let next_c = chars.next().unwrap_or('\\');

        // Check if it's a shorthand character class
        match next_c {
            'd' | 'D' | 'w' | 'W' | 's' | 'S' => {
                if let Ok(class) = CharacterClass::from_shorthand(next_c) {
                    tokens.push_back(RegexType::CharacterClass(class));
                } else {
                    tokens.push_back(RegexType::Char(Utils::backslashed(next_c)));
                }
            }
            // Handle other escape sequences
            _ => tokens.push_back(RegexType::Char(Utils::backslashed(next_c))),
        }
    }

    /// Handling litterals (trick: transform litterals into parenthesis of chars)
    pub fn add_string(
        tokens: &mut VecDeque<RegexType>,
        chars: &mut Peekable<Chars<'_>>,
    ) -> ParsingResult<()> {
        // Open parenthesis replace open '"'
        tokens.push_back(RegexType::OpenParenthesis);

        while let Some(c) = chars.next() {
            match c {
                '\\' => {
                    let c = chars.next().unwrap_or('\\');

                    tokens.push_back(RegexType::Char(Utils::backslashed(c)));
                }

                '\"' => {
                    // Close parenthesis replace close '"'
                    tokens.push_back(RegexType::CloseParenthesis);
                    return Ok(());
                }

                c => tokens.push_back(RegexType::Char(c)),
            }
        }

        return Err(ParsingError::unrecognized_rule().because("Unclosed string"));
    }

    /// Handle character classes ([...])
    pub fn add_character_class(
        tokens: &mut VecDeque<RegexType>,
        chars: &mut Peekable<Chars<'_>>,
    ) -> ParsingResult<()> {
        let class = CharacterClass::parse(chars)?;
        tokens.push_back(RegexType::CharacterClass(class));
        Ok(())
    }

    /// Handle quantifiers ({n}, {n,}, {n,m})
    pub fn add_quantifier(
        tokens: &mut VecDeque<RegexType>,
        chars: &mut Peekable<Chars<'_>>,
    ) -> ParsingResult<()> {
        let mut digits1 = String::new();
        let mut digits2 = String::new();
        let mut saw_comma = false;

        while let Some(c) = chars.next() {
            match c {
                '0'..='9' => {
                    if saw_comma {
                        digits2.push(c);
                    } else {
                        digits1.push(c);
                    }
                }
                ',' => {
                    saw_comma = true;
                }
                '}' => {
                    let quant = if !saw_comma {
                        // {n}
                        let n = digits1.parse::<usize>().map_err(|_| {
                            ParsingError::unrecognized_rule().because("Invalid quantifier number")
                        })?;
                        Quantifier::Exact(n)
                    } else if digits2.is_empty() {
                        // {n,}
                        let n = digits1.parse::<usize>().map_err(|_| {
                            ParsingError::unrecognized_rule().because("Invalid quantifier number")
                        })?;
                        Quantifier::AtLeast(n)
                    } else {
                        // {n,m}
                        let n = digits1.parse::<usize>().map_err(|_| {
                            ParsingError::unrecognized_rule().because("Invalid quantifier number")
                        })?;
                        let m = digits2.parse::<usize>().map_err(|_| {
                            ParsingError::unrecognized_rule().because("Invalid quantifier number")
                        })?;

                        if n > m {
                            return Err(ParsingError::unrecognized_rule()
                                .because("Invalid quantifier range: min > max"));
                        }

                        Quantifier::Range(n, m)
                    };

                    tokens.push_back(RegexType::Quant(quant));
                    return Ok(());
                }
                _ => {
                    return Err(ParsingError::unrecognized_rule()
                        .because("Invalid character in quantifier"));
                }
            }
        }

        Err(ParsingError::unrecognized_rule().because("Unclosed quantifier"))
    }

    pub fn into_type(c: char) -> RegexType {
        match c {
            '*' => RegexType::Quant(Quantifier::AtLeast(0)),

            '+' => RegexType::Quant(Quantifier::AtLeast(1)),

            '(' => RegexType::OpenParenthesis,

            ')' => RegexType::CloseParenthesis,

            '?' => RegexType::Quant(Quantifier::Range(0, 1)),

            '|' => RegexType::Or,

            c => RegexType::Char(c),
        }
    }
}
