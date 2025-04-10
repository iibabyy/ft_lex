use crate::regex::*;
use crate::regex::dfa::*;
use crate::regex::post2nfa::*;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

fn into_postfix(str: &str) -> VecDeque<TokenType> {
	re2post(Regex::add_concatenation(Regex::tokens(str).unwrap())).unwrap()
}
