use std::{iter::Peekable, rc::Rc};

use super::*;

static mut LIST_ID: usize = 0;

pub struct List {
	pub states: Vec<Rc<State>>,
}

impl List {
	pub fn contains(&self, to_find: &Rc<State>) -> bool {
		self.states.iter()
			.any(|state| Rc::ptr_eq(&to_find, &state))
	}

	pub fn push(&mut self, value: Rc<State>) {
			self.states.push(value)
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

	pub fn from(state: Rc<State>) -> Self {
		let mut list = Self::new();

		add_state(state, &mut list);
		
		list
	}
	
	pub fn iter(&self) -> std::slice::Iter<'_, Rc<State>> {
		self.states.iter()
	}

	pub fn is_matched(&self) -> bool {
		self.states.iter()
			.any(|state| matches!(state.as_ref(), State::Match))
	}
}

pub fn increment_list_id() {
	unsafe { LIST_ID += 1 }
}

pub fn set_list_id(id: usize) {
	unsafe { LIST_ID = id }
}

pub fn current_list_id() -> usize {
	unsafe { LIST_ID }
}

pub fn input_match(state: StatePtr, input: &str) -> bool {
	if State::is_none_ptr(&state) {
		return false
	}

	set_list_id(0);

	let mut current_states = List::from(State::from_ptr(&state));
	let mut next_states = List::new();

	let mut chars = input.chars().peekable();

	while let Some(_) = chars.peek() {

		step(&mut chars, &current_states, &mut next_states);

		std::mem::swap(&mut current_states, &mut next_states);
	}

	current_states.is_matched()
}

pub fn step(chars: &mut Peekable<Chars>, current_states: &List, next_states: &mut List) {
	let c = chars.next().unwrap();

	next_states.clear();

	for state in current_states.iter() {
		match state.as_ref() {
			State::Basic(basic) if basic.c.match_(&c) => {
				add_state(State::from_ptr(&basic.out), next_states);
			}

			_ => {},
		}
	}
}

pub fn add_state(state: Rc<State>, next_states: &mut List) {
	if State::is_none(&state) {
		return;
	}

	// Already added to next_states
	if next_states.contains(&state) {
		return ;
	}

	match state {
		split if matches!(split.as_ref(), State::Split(_)) => {
			let outs = split.split_out().unwrap();

			add_state(State::from_ptr(&outs.0), next_states);
			add_state(State::from_ptr(&outs.1), next_states);
		},

		state => {
			next_states.push(state);
		},
	}
}