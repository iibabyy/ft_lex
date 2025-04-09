use std::{cell::RefCell, collections::{BTreeMap, HashMap}, rc::Rc};
use super::*;

pub type ListPtr = Rc<StateList>;
pub type DfaStatePtr = Rc<DfaState>;
pub type DfaMemory = MutPtr<HashMap<StateList, DfaStatePtr>>;
pub fn ptr_list(list: StateList) -> ListPtr {
	Rc::new(list)
}

pub fn ptr_state(state: DfaState) -> DfaStatePtr {
	Rc::new(state)
}

/// Merges two HashMaps of InputCondition to StateList
/// 
/// For each key that exists in both maps, the corresponding StateLists are merged.
/// For keys that exist in only one map, they are copied to the result.
pub fn merge_input_maps(
    map1: &mut HashMap<InputCondition, StateList>,
    map2: HashMap<InputCondition, StateList>
) {
    let mut result = map1;

    for (input, state_list2) in map2 {
        if let Some(state_list1) = result.get_mut(&input) {
            // If the key exists in both maps, merge the StateLists
            let mut merged = state_list1.clone();
            merged.merge(state_list2);
            *state_list1 = merged;
        } else {
            // If the key only exists in map2, add it to the result
            result.insert(input.clone(), state_list2.clone());
        }
    }
}



#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputCondition {
	StartOfLine,
	EndOfLine,
	Char(char),
}

pub struct Dfa {
	start: DfaStatePtr,

	memory: HashMap<StateList, DfaStatePtr>,
}

impl Dfa {
	pub fn new(start: StatePtr) -> Self {
		Dfa {
			start: ptr_state(DfaState::new(0, &ptr_list(StateList::from(&start)))),
			memory: HashMap::new(),
		}
	}

	pub fn add_state(&mut self, state: &DfaStatePtr) {
		let list = state.states.clone();
		self.memory.insert(list, state.clone());
	}

	pub fn get_state(&self, list: &StateList) -> Option<&DfaStatePtr> {
		self.memory.get(list)
	}
}


pub struct DfaState {
	id: usize,

	states: StateList,

	matchs: StateList,

	next: HashMap<InputCondition, StateList>,
}

impl DfaState {
	pub fn new(id: usize, states: &StateList) -> Self {
		DfaState {
			id,
			states: states.clone(),
			matchs: StateList::new(),
			next: HashMap::new(),
		}
	}

	pub fn next(&self, input: &InputCondition) -> Option<&StateList> {
		self.next.get(input)
	}

	pub fn compute_next(&mut self) {
		for state in self.states.iter() {
			let (next_states, matchs) = DfaState::find_next(state, &mut self.memory);
			merge_input_maps(&mut self.next, next_states);
			self.matchs.merge(matchs);
		}
	}

	/// Computes the next possible states from a given state in the NFA.
	///
	/// This function analyzes a state and determines all possible transitions from it,
	/// categorized by input conditions. It also collects any match states encountered.
	///
	/// # Arguments
	///
	/// * `state` - A reference to the current state pointer to analyze
	///
	/// # Returns
	///
	/// A tuple containing:
	/// * A HashMap mapping input conditions to the states reachable under those conditions
	/// * A StateList containing any match states encountered
	pub fn find_next(state: &StatePtr, memory: &mut HashMap<StateList, DfaStatePtr>) -> (HashMap<InputCondition, StateList>, StateList) {
		let mut next_states: HashMap<InputCondition, StateList> = HashMap::new();
		let mut matchs: StateList = StateList::new();

		match &*state.borrow() {
			State::Basic(basic) => {
				let condition = InputCondition::Char(basic.c.char().expect("Basic state should have a char"));
				let list = next_states.entry(condition).or_insert_with(|| StateList::new());
				list.add_state(state);
			},

			State::Split(split) => {
				let (next_states_1, matchs_1) = DfaState::find_next(&*State::deref_var_ptr(&split.out1), memory);
				let (next_states_2, matchs_2) = DfaState::find_next(&*State::deref_var_ptr(&split.out2), memory);

				matchs.merge(matchs_1);
				matchs.merge(matchs_2);

				for (condition, next_list) in next_states_1 {
					let list = next_states.entry(condition).or_insert_with(|| StateList::new());
					
					list.merge(next_list);
				}

				for (condition, next_list) in next_states_2 {
					let list = next_states.entry(condition).or_insert_with(|| StateList::new());
					
					list.merge(next_list);
				}
				
			},

			State::StartOfLine { out } => {
				let list = next_states.entry(InputCondition::StartOfLine).or_insert_with(|| StateList::new());
				list.add_state(&*out.borrow());
			},

			State::EndOfLine { out } => {
				let list = next_states.entry(InputCondition::EndOfLine).or_insert_with(|| StateList::new());
				list.add_state(&*out.borrow());
			},

			State::Match => {
				matchs.add_state(state);
			},

			_ => { eprintln!("Unhandled state: {:?}", state); }
		}

		(next_states, matchs)
	}
}