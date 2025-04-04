use std::{iter::Peekable, ops::Deref, rc::Rc};

use super::*;

#[derive(Debug)]
pub struct List {
	pub states: Vec<StatePtr>,
}

impl List {
	pub fn contains(&self, to_find: &StatePtr) -> bool {
		self.states.iter()
			.any(|state| Rc::ptr_eq(&to_find, &state))
	}

	pub fn push(&mut self, value: &StatePtr) {
		self.states.push(Rc::clone(value))
	}

	pub fn clear(&mut self) {
		self.states.clear()
	}
	
	pub fn len(&self) -> usize {
		self.states.len()
	}
}

impl List {
	pub fn new() -> Self {
		Self {
			states: vec![]
		}
	}

	pub fn from(state: &StatePtr) -> Self {
		let mut list = Self::new();

		add_state(state, &mut list);
		
		list
	}
	
	pub fn iter(&self) -> std::slice::Iter<'_, StatePtr> {
		self.states.iter()
	}

	pub fn is_matched(&self) -> bool {
		self.states.iter()
			.any(|state| State::is_match_ptr(state))
	}
}

pub struct NfaSimulation<'a> {
	current_states: List,
	next_states: List,

	on_start_of_line: bool,
	on_end_of_line: bool,

	input: &'a str,
	chars: Peekable<Chars<'a>>,

	nfa: &'a Nfa
}

impl<'a> NfaSimulation<'a> {
	pub fn new(input: &'a str, nfa: &'a Nfa) -> Self {
		let current_states = List::from(&nfa.start);
		let next_states = List::new();

		let on_start_of_line = true;
		let on_end_of_line = true;

		let chars = input.chars().peekable();

		NfaSimulation {
			current_states,
			next_states,
			on_start_of_line,
			on_end_of_line,
			input,
			chars,
			nfa
		}
	}

	pub fn peek(&mut self) -> Option<&char> {
		self.chars.peek()
	}

	pub fn has_input_left(&mut self) -> bool {
		self.peek().is_some()
	}

	pub fn next(&mut self) -> Option<char> {
		self.chars.next()
	}

	pub fn is_end_of_line(&mut self) -> bool {
		self.peek() == Some(&'\n')
	}

	pub fn step(&mut self) {
		if self.has_input_left() == false {
			return ;
		}

		let c = self.next().unwrap();

		self.next_states.clear();


		if self.nfa.start.borrow().matche_with(&c) {
			if self.nfa.start_of_line == false || self.on_start_of_line == true {
				add_state(&self.nfa.start, &mut self.next_states);
			}
		}


		for state in self.current_states.iter() {
			if state.borrow().is_basic() == false {
				return ;
			}

			if  state.borrow().into_basic().unwrap().c.match_(&c) {
				add_state(&State::deref_var_ptr(&state.borrow().basic_out().unwrap()), &mut self.next_states);
			}
		}

		std::mem::swap(&mut self.current_states, &mut self.next_states);

		self.on_start_of_line = false;
	}
}

pub fn input_match(nfa: &Nfa, input: &str) -> bool {
	let mut simulation = NfaSimulation::new(input, nfa);

	if State::is_none_ptr(&nfa.start) {
		return false
	}

	if State::is_match_ptr(&nfa.start) {
		return true
	}

	while simulation.has_input_left() {
		simulation.step();
	}

	if nfa.end_of_line == true && simulation.chars.peek().is_some() {
		return false;
	}

	simulation.current_states.is_matched()
}

pub fn add_state(state: &StatePtr, list: &mut List) {
	if State::is_none_ptr(&state) {
		panic!("None ptr");
	}

	// Already added to list
	if list.contains(&state) {
		return ;
	}

	if State::is_split_ptr(&state) {
		let borrowed_state = state.borrow();
		let split = borrowed_state.into_split().unwrap();

		// out1
		add_state(&split.out1.borrow(), list);
		// out2
		add_state(&split.out2.borrow(), list);
	} else {
		list.push(state);
	}
}