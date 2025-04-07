pub mod parsing;
pub use parsing::*;

pub mod re2post;
pub use re2post::*;

pub mod post2nfa;
pub use post2nfa::*;

pub mod nfa_simulation;
pub use nfa_simulation::*;

pub mod dfa;
pub use dfa::*;

use std::{collections::VecDeque, fmt, ops, str::Chars};

use super::*;

// 1. BASIC TYPE DEFINITIONS
// =========================

pub struct Regex {
	// Needed for the conversion to postfix
    operator_stack: Vec<parsing::RegexType>,
    output_stack: Vec<parsing::RegexType>,
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
