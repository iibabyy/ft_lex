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

pub enum NfaStatus {
	Match(usize),
	NoMatch,
	Pending
}

/// Main simulation controller for NFA-based regex matching
#[derive(Debug)]
// Input string to match against
pub struct NfaSimulation<'a> {
	start_of_line: bool,
	readed: usize,
	longest_match: Option<usize>,

    // NFA to use for matching
    nfa: &'a Nfa,

    // All active validation paths
    current_states: StateList,
    // Validation paths that have successfully matched
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

	pub fn switch_to_next_states(&mut self) {
		std::mem::swap(&mut self.current_states, &mut self.next_states);

		self.next_states.clear();
	}

	pub fn check_start_of_line(&self) -> bool {
		self.start_of_line == self.nfa.start_of_line
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

	pub fn step(&mut self, c: &char, end_of_line: bool) -> NfaStatus {

		if self.check_start_of_line() == false || self.current_states.is_empty() {
			return self.status()
		}

		self.readed += 1;

		for state in self.current_states.iter() {
			if State::is_basic_ptr(state) == false {
				continue;
			}

			let borrowed_state = state.borrow();

			if borrowed_state.matche_with(c) {
				let out = &borrowed_state.basic_out().unwrap();
				let next_state = out.borrow();

				self.next_states.add_state(&next_state);
			}
		}

		if self.next_states.is_matched() {
			if self.nfa.end_of_line == end_of_line {
				self.longest_match = Some(self.readed);
			}
			self.next_states.remove_matchs();
		}

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

pub fn input_match(nfa: &Nfa, input: &str) -> bool {
    let mut simulation = NfaSimulation::new(nfa);

	let mut chars = input.chars().peekable();

	let start_of_line = true;

	simulation.start(start_of_line);

	while let Some(c) = chars.next() {
		let peek = chars.peek();
		let end_of_line = peek == None || peek == Some(&'\n');

		match simulation.step(&c, end_of_line) {
			NfaStatus::Pending => continue,

			// finished
			_ => break,
		}
	}

	simulation.longest_match.is_some()
}
