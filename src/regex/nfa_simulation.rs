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

	pub fn push(&mut self, state: &StatePtr) {
		self.states.push(Rc::clone(state));
	}
	
	pub fn clear(&mut self) {
		self.states.clear()
	}
	
	pub fn is_empty(&self) -> bool {
		self.states.is_empty()
	}
}

pub enum NfaStatus {
	Match(usize),
	NoMatch,
	Pending
}

/// Main simulation controller for NFA-based regex matching
#[derive(Debug)]
pub struct NfaSimulation<'a> {
    // Input string to match against
    input: &'a str,
	chars: Peekable<Chars<'a>>,
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
	pub fn new(nfa: &'a Nfa, input: &'a str) -> Self {

		let chars = input.chars().peekable();

		let readed = 0;
		let longest_match = None;

		let current_states = StateList::from(&nfa.start);
		let next_states = StateList::new();

		NfaSimulation {
			input,
			chars,
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

	pub fn status(&self) -> NfaStatus {
		if self.current_states.is_empty() == false {
			return NfaStatus::Pending
		}

		if self.longest_match.is_none() {
			return NfaStatus::NoMatch
		}

		NfaStatus::Match(self.longest_match.unwrap())
	}

	pub fn step(&mut self) -> NfaStatus {

		if self.current_states.is_empty() {
			return self.status()
		}

		

		self.switch_to_next_states();
		todo!()
	}
}

pub fn input_match(nfa: &Nfa, input: &str) -> bool {
    // 	let mut simulation = NfaSimulation::new(input, nfa);

    // 	if State::is_none_ptr(&nfa.start) {
    // 		return false
    // 	}

    // 	if State::is_match_ptr(&nfa.start) {
    // 		return true
    // 	}

    // 	while simulation.has_input_left() {
    // 		simulation.step();
    // 	}

    // 	if nfa.end_of_line == true && simulation.chars.peek().is_some() {
    // 		return false;
    // 	}

    // 	simulation.current_states.is_matched()
    todo!()
}
