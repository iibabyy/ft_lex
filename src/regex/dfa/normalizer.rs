use std::{collections::HashMap, rc::Rc};

use super::*;

pub struct NormalizedMatch {}

#[derive(Debug)]
pub struct NormalizedState {
	pub id: usize,
	pub matchs: HashSet<usize>,

	pub next: HashMap<InputCondition, usize>,
}

impl NormalizedState {
	pub fn new(id: usize, matchs: HashSet<usize>, next: HashMap<InputCondition, usize>) -> Self {
		Self { id, matchs, next }
	}
}

pub struct NormalizedDfa {
	pub start_id: usize,

	pub states: HashMap<usize, NormalizedState>,

	pub matchs: HashMap<usize, StatePtr>,
}

impl NormalizedDfa {
	pub fn from(dfa: &mut Dfa) -> Self {
		let mut match_memory = HashMap::new();

		let mut normalized_states = HashMap::new();

		for state in dfa.memory.values() {
			let normalized = Self::normalize_state(state, &dfa.memory, &mut match_memory);
			normalized_states.insert(state.borrow().id, normalized);
		}

		let start_id = dfa.start.borrow().id;

		Self { start_id, states: normalized_states, matchs: match_memory }
	}

	pub fn normalize_state(state: &DfaStatePtr, memory: &HashMap<StateList, DfaStatePtr>, match_memory: &mut HashMap<usize, StatePtr>) -> NormalizedState {
		let next = Self::normalize_hashmap(&state.borrow().next, memory);

		let mut matchs = HashSet::new();

		for state in &state.borrow().matchs {
			if let State::Match { id } = &*state.borrow() {
				matchs.insert(*id);
				if !match_memory.contains_key(id) {
					match_memory.insert(*id, Rc::clone(state));
				}
			}
		}

		NormalizedState::new(state.borrow().id, matchs, next)
	}

	pub fn normalize_hashmap(map: &HashMap<InputCondition, StateList>, memory: &HashMap<StateList, DfaStatePtr>) -> HashMap<InputCondition, usize> {
		let mut normalized = HashMap::new();

		for (condition, list) in map {
			let id = Self::normalize_statelist(list, memory).expect("Failed to normalize state list");
			normalized.insert(condition.clone(), id);
		}

		normalized
	}

	pub fn normalize_statelist(list: &StateList, memory: &HashMap<StateList, DfaStatePtr>) -> Option<usize> {
		if let Some(ptr) = memory.get(list) {
			return Some(ptr.borrow().id);
		}

		None
	}
}

pub struct Match {
	state: StatePtr,
	length: usize
}

impl Match {
	pub fn length(&self) -> usize {
		self.length
	}
}

pub fn simulate(str: &str, dfa: &NormalizedDfa) -> Option<Match> {

	let mut matchs: HashMap<usize, usize> = HashMap::new();

	let mut current = match dfa.states.get(&dfa.start_id) {
		Some(state) => state,
		None => return None,
	};

	add_matchs(0, current, &mut matchs);

	let mut readed = 0;
	let mut chars = str.chars().peekable();

	current = if_start_of_line(readed, current, &dfa.states, &mut matchs);

	while let Some(c) = chars.next() {
		readed += 1;
		let end_of_line = chars.peek() == Some(&'\n');

		current = match step(c, readed, current, &dfa.states, &mut matchs) {
			Some(state) => state,
			None => break
		};

		if end_of_line {
			current = if_end_of_line(readed, current, &dfa.states, &mut matchs)
		}
	}

	if matchs.is_empty() {
		return None;
	}

	let mut match_id = usize::MAX;
	let mut match_length = 0;

	matchs.iter().for_each(|(id, length)| {
		if match_length < *length {
			match_id = *id;
			match_length = *length;
		} else if match_length == *length && match_id > *id {
			// same length -> first match added (lower id)
			match_id = *id;
		}
	});

	let match_state = match dfa.matchs.get(&match_id) {
		Some(state) => state,
		None => return None
	};

	dbg!(&match_state);

	Some(Match {
		state: Rc::clone(match_state),
		length: match_length
	})
}

fn add_matchs(
	readed: usize,
	current: &NormalizedState,
	matchs: &mut HashMap<usize, usize>
) {
	current.matchs.iter().for_each(|match_id| {
		// if already added
		if let Some(previous_readed) = matchs.insert(*match_id, readed) {
			// if previous match was longer
			if previous_readed > readed {
				matchs.insert(*match_id, previous_readed);
			}
		}
	});
}

fn step<'a>(
	c: char,
	readed: usize,
	mut current: &'a NormalizedState,
	states: &'a HashMap<usize, NormalizedState>,
	matchs: &mut HashMap<usize, usize>
) -> Option<&'a NormalizedState> {

	current = match get_next(current, InputCondition::Char(c)) {
		Some(next) => {
			match states.get(&next) {
				Some(state) => state,
				None => return None,
			}
		},
		None => return None,
	};

	add_matchs(readed, current, matchs);

	Some(current)
}

fn if_start_of_line<'a>(
	readed: usize,
	mut current: &'a NormalizedState,
	states: &'a HashMap<usize, NormalizedState>,
	matchs: &mut HashMap<usize, usize>
) -> &'a NormalizedState {

	current = match get_next(current, InputCondition::StartOfLine) {
		Some(next) => states.get(&next).unwrap_or(current),
		None => return current,
	};

	add_matchs(readed, current, matchs);

	current
}

fn if_end_of_line<'a>(
	readed: usize,
	mut current: &'a NormalizedState,
	states: &'a HashMap<usize, NormalizedState>,
	matchs: &mut HashMap<usize, usize>
) -> &'a NormalizedState {

	current = match get_next(current, InputCondition::EndOfLine) {
		Some(next) => states.get(&next).unwrap_or(current),
		None => return current,
	};

	add_matchs(readed, current, matchs);

	current
}

fn get_next(current: &NormalizedState, input: InputCondition) -> Option<usize> {
	if let Some(next) = current.next.get(&input) {
		return Some(*next);
	}

	None
}