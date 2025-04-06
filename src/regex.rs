pub mod parsing;
pub use parsing::*;

pub mod re2post;
pub use re2post::*;

pub mod post2nfa;
pub use post2nfa::*;

pub mod nfa_simulation;
pub use nfa_simulation::*;

use std::{collections::VecDeque, fmt, ops, str::Chars};

use super::*;

// 1. BASIC TYPE DEFINITIONS
// =========================

pub struct Regex {
	// Needed for the conversion to postfix
    operator_stack: Vec<RegexType>,
    output_stack: Vec<RegexType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegexType {
    Char(char),
    LineStart,
    LineEnd,
    OpenParenthesis,
    CloseParenthesis,
    Any,
    Or,
    Concatenation,
    Class(CharacterClass),
    Quant(Quantifier),
}

// 3. DISPLAY IMPLEMENTATIONS
// =========================

impl fmt::Display for RegexType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegexType::Char(c) => write!(f, "{}", c),
            RegexType::LineStart => write!(f, "^"),
            RegexType::LineEnd => write!(f, "$"),
            RegexType::OpenParenthesis => write!(f, "("),
            RegexType::CloseParenthesis => write!(f, ")"),
            RegexType::Any => write!(f, "."),
            RegexType::Or => write!(f, "|"),
            RegexType::Concatenation => write!(f, "&"),
            RegexType::Class(c) => write!(f, "{}", c),
            RegexType::Quant(q) => write!(f, "{}", q),
        }
    }
}

// 5. TYPE-SPECIFIC METHODS
// =======================

impl RegexType {
    pub fn precedence(&self) -> usize {
        match self {
            RegexType::Quant(_) => 3,

            RegexType::Concatenation => 2,

            RegexType::Or => 1,

            _ => 0,
        }
    }

    pub fn type_(&self) -> parsing::TokenType {
        match self {
            // Opening group
            RegexType::OpenParenthesis => parsing::TokenType::OpenParenthesis(self.clone()),

            // Closing group
            RegexType::CloseParenthesis => parsing::TokenType::CloseParenthesis(self.clone()),

            // One element operator
            RegexType::Quant(_) => parsing::TokenType::UnaryOperator(self.clone()),

            // Two element operator
            RegexType::Or | RegexType::Concatenation => parsing::TokenType::BinaryOperator(self.clone()),

            // start or end of line conditions
            RegexType::LineStart | RegexType::LineEnd => {
                parsing::TokenType::StartOrEndCondition(self.clone())
            }

            _ => parsing::TokenType::Literal(self.clone()),
        }
    }

    pub fn match_(&self, c: &char) -> bool {
        match self {
            RegexType::Char(char) => char == c,

            RegexType::Class(class) => class.matches(c),

			RegexType::Any => true,

            _ => todo!(),
        }
    }
}

// 6. REGEX PARSING IMPLEMENTATION
// ==============================

impl Regex {
    pub fn new(expr: String) -> ParsingResult<Nfa> {
        let tokens = Self::tokens(&expr)?;

        let tokens_with_concatenation = Self::add_concatenation(tokens);

        let postfix = re2post(tokens_with_concatenation)?;

		let nfa = post2nfa(postfix)?;

        Ok(nfa)
    }
}
