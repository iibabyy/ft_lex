use std::{iter::Peekable, ops::Deref, rc::Rc};

use super::*;
// ===================================
// 1. DATA STRUCTURES FOR NFA SIMULATION
// ===================================

/// Represents a list of NFA states during simulation
#[derive(Debug)]
pub struct StateList {
    states: Vec<StatePtr>,
}

impl StateList {
	pub fn new() -> Self {
		StateList { states: Vec::with_capacity(1) }
	}

	pub fn from(state: &StatePtr) -> Self {
		let state = Rc::clone(state);

		StateList {
			states: Vec::from([state])
		}
	}

    pub fn add_state(&mut self, state: &StatePtr) {
        if State::is_none_ptr(&state) {
            panic!("None ptr");
        }

        // Already added to list
        if self.contains(&state) {
            return;
        }

        // Don't directly add split states, add their out states instead
        if State::is_split_ptr(&state) {
            let borrowed_state = state.borrow();
            let split = borrowed_state.into_split().unwrap();

            // out1
            self.add_state(&split.out1.borrow());
            // out2
            self.add_state(&split.out2.borrow());
        } else {
            self.push(state);
        }
    }

	pub fn contains(&self, to_find: &StatePtr) -> bool {
		self.states
			.iter()
			.any(|state|
				Rc::ptr_eq(to_find, state)
			)
	}

	pub fn is_matched(&self) -> bool {
		self.states
		.iter()
		.any(|state|
			State::is_match_ptr(state)
		)
	}

	pub fn remove_matchs(&mut self) {
		let mut matchs = vec![];

		self.states.iter().enumerate().for_each(|(index, state)|
			if State::is_match_ptr(state) {
				matchs.push(index);
			}
		);

		let mut removed = 0;
		for index in matchs {
			self.states.remove(index - removed);
			removed += 1;
		}
	}

	pub fn push(&mut self, state: &StatePtr) {
		self.states.push(Rc::clone(state));
	}
	
	pub fn clear(&mut self) {
		self.states.clear()
	}
	
	pub fn is_empty(&self) -> bool {
		self.states.is_empty()
	}
	
	pub fn iter(&self) -> std::slice::Iter<'_, StatePtr> {
		self.states.iter()
	}
	
}

/// Represents the status of the NFA simulation
pub enum NfaStatus {
	Match(usize),
	NoMatch,
	Pending
}

/// Main simulation controller for NFA-based regex matching
#[derive(Debug)]
pub struct NfaSimulation<'a> {
	/// If the current input is at the start of a line
	start_of_line: bool,

	/// The current number of characters readed
	readed: usize,

	/// The number of characters read until match (if matched)
	longest_match: Option<usize>,

    /// NFA to use for matching
    nfa: &'a Nfa,

    /// All active validation paths
    current_states: StateList,

	/// Next validation paths that have successfully matched
    next_states: StateList,
}

impl<'a> NfaSimulation<'a> {
	pub fn new(nfa: &'a Nfa) -> Self {

		let readed = 0;
		let longest_match = None;

		let current_states = StateList::from(&nfa.start);
		let next_states = StateList::new();

		NfaSimulation {
			start_of_line: false,
			readed,
			longest_match,
			nfa,
			current_states,
			next_states
		}
	}

	/// Current states are now next states, and next states are cleared
	pub fn switch_to_next_states(&mut self) {
		std::mem::swap(&mut self.current_states, &mut self.next_states);

		self.next_states.clear();
	}

	/// Check if the start of line matches the NFA's start of line condition
	pub fn check_start_of_line(&self) -> bool {
		self.nfa.start_of_line == false || self.start_of_line == true
	}

	pub fn status(&self) -> NfaStatus {
		if self.check_start_of_line() == false {
			return NfaStatus::NoMatch
		}

		if self.current_states.is_empty() == false {
			return NfaStatus::Pending
		}

		if self.longest_match.is_none() {
			return NfaStatus::NoMatch
		}

		NfaStatus::Match(self.longest_match.unwrap())
	}

	/// Step the simulation forward by one character.
	/// 
	/// - c :  The current character
	/// 
	/// - end_of_line :  If the current character is at the end of a line
	pub fn step(&mut self, c: &char, end_of_line: bool) -> NfaStatus {

		if self.check_start_of_line() == false || self.current_states.is_empty() {
			return self.status()
		}

		self.readed += 1;

		for state in self.current_states.iter() {
			// The states should be basic states
			if State::is_basic_ptr(state) == false {
				continue;
			}

			let borrowed_state = state.borrow();

			// Check if the state matches the current character
			if borrowed_state.matche_with(c) {
				let out = &borrowed_state.basic_out().unwrap();
				let next_state = out.borrow();

				self.next_states.add_state(&next_state);
			}
		}

		// Check if the next states have a match
		if self.next_states.is_matched() {
			if self.nfa.end_of_line == end_of_line {
				self.longest_match = Some(self.readed);
			}
			self.next_states.remove_matchs();
		}
		
		// remove the matchs, to only keep active states in the next states
		self.switch_to_next_states();
		
		return self.status()
	}

	pub fn start(&mut self, start_of_line: bool) {
		self.readed = 0;
		self.longest_match = None;
		self.current_states.clear();
		self.next_states.clear();
		self.start_of_line = start_of_line;
	}
}

/// Implements a traditional NFA simulation where we track all possible states simultaneously. \
/// The algorithm maintains two sets of states (current_states and next_states) and follows all possible
/// paths through the NFA in parallel. This approach handles nondeterminism by exploring all possible
/// transitions for each input character, which is the defining characteristic of Thompson's NFA simulation.
pub fn input_match(nfa: &Nfa, input: &str) -> bool {
    let mut simulation = NfaSimulation::new(nfa);

	let mut chars = input.chars().peekable();

	let start_of_line = true;

	simulation.start(start_of_line);

	while let Some(c) = chars.next() {
		let peek = chars.peek();
		// check if the next character is the end of a line
		let end_of_line = peek == None || peek == Some(&'\n');

		match simulation.step(&c, end_of_line) {
			NfaStatus::Pending => continue,

			// finished
			_ => break,
		}
	}

	simulation.longest_match.is_some()
}
