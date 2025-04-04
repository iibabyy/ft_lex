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

pub fn input_match(state: &StatePtr, input: &str) -> bool {
	if State::is_none_ptr(state) {
		return false
	}

	if State::is_match_ptr(state) {
		return true
	}

	let mut current_states = List::from(state);
	let mut next_states = List::new();

	let mut chars = input.chars().peekable();

	while let Some(_) = chars.peek() {

		step(&mut chars, &current_states, &mut next_states);

		std::mem::swap(&mut current_states, &mut next_states);
	}

	current_states.is_matched()
}

pub fn step(chars: &mut Peekable<Chars>, current_states: &List, next_states: &mut List) {
	if chars.peek().is_none() {
		return ;
	}

	let c = chars.next().unwrap();

	next_states.clear();

	for state in current_states.iter() {
		if state.borrow().is_basic() == false {
			return ;
		}

		if state.borrow().into_basic().unwrap().c.match_(&c) {
			add_state(&State::deref_var_ptr(&state.borrow().basic_out().unwrap()), next_states);
		}
	}
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