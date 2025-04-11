use std::{collections::HashMap, rc::Rc};

use super::*;

pub struct NormalizedMatch {}

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
