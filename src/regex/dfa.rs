use std::{cell::RefCell, collections::{BTreeMap, HashMap}, rc::Rc, usize};
use super::*;

pub mod normalizer;

pub type DfaStatePtr = Rc<RefCell<DfaState>>;

/// Merges two HashMaps of InputCondition to StateList
/// 
/// For each key that exists in both maps, the corresponding StateLists are merged.
/// For keys that exist only in map2, they are moved to map1.
pub fn merge_input_maps(
    map1: &mut HashMap<InputCondition, StateList>,
    map2: HashMap<InputCondition, StateList>
) {
    for (input, state_list2) in map2 {
        if let Some(state_list1) = map1.get_mut(&input) {
            // If the key exists in both maps, merge the StateLists
            let mut merged = state_list1.clone();
            merged.merge(state_list2);
            *state_list1 = merged;
        } else {
            // If the key only exists in map2, add it to the map1
            map1.insert(input, state_list2);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputCondition {
	StartOfLine,
	EndOfLine,
	Char(char),
}

#[derive(Debug)]
pub struct Dfa {
	pub start: DfaStatePtr,

	pub memory: HashMap<StateList, DfaStatePtr>,
}

impl Dfa {
	pub fn new(starts: Vec<StatePtr>) -> Self {
		let mut list = StateList::new();

		for state in &starts {
			list.add_state(state);
		}

		let (start, memory) = DfaState::iterative_create(list);

		Dfa {
			start,
			memory
		}
	}

}

#[derive(Debug)]
pub struct DfaState {
	pub id: usize,

	pub states: StateList,

	pub matchs: StateList,

	pub next: HashMap<InputCondition, StateList>,
}

impl DfaState {
	pub fn new(id: usize, mut states: StateList) -> Self {
		let mut matchs = StateList::new();

		for match_ in states.remove_matchs() {
			matchs.add_state(&match_);
		}

		DfaState {
			id,
			states,
			matchs,

			next: HashMap::new(),
		}
	}

	#[deprecated(note="please use `iterative_create` instead")]
	pub fn recursive_create(states: StateList, memory: &mut HashMap<StateList, DfaStatePtr>) -> DfaStatePtr {
		if let Some(next) = memory.get(&states) {
			return Rc::clone(next)
		}

		let mut states = DfaState::new(memory.len(), states);

		states.compute_next();

		let states = Rc::new(RefCell::new(states));

		memory.insert(states.borrow().states.clone(), Rc::clone(&states));

		for (_condition, list) in &states.borrow().next {
			DfaState::recursive_create(list.clone(), memory);
		}

		states
	}

	pub fn iterative_create(start_states: StateList) -> (DfaStatePtr, HashMap<StateList, DfaStatePtr>) {
		let mut memory = HashMap::new();
		let mut work_queue = VecDeque::new();
		
		// Create and process the initial state
		let start = DfaState::new(0, start_states.clone());
		let start_ptr = Rc::new(RefCell::new(start));

		memory.insert(start_states.clone(), Rc::clone(&start_ptr));

		// Add initial transitions to work queue
		start_ptr.borrow_mut().compute_next();
		
		for (_, list) in &start_ptr.borrow().next {
			if !memory.contains_key(list) && !work_queue.contains(list) {
				work_queue.push_back(list.clone());
			}
		}
		
		// Process work queue iteratively
		while let Some(state_list) = work_queue.pop_front() {
			let dfa_state = DfaState::new(memory.len(), state_list.clone());
			let state_ptr = Rc::new(RefCell::new(dfa_state));
			
			memory.insert(state_list, Rc::clone(&state_ptr));
			state_ptr.borrow_mut().compute_next();
			
			// Add new states to work queue
			for (_, list) in &state_ptr.borrow().next {
				if !memory.contains_key(list) && !work_queue.contains(list) {
					work_queue.push_back(list.clone());
				}
			}
		}
		
		// Return the start state
		(Rc::clone(&memory[&start_states]), memory)
	}

	pub fn compute_next(&mut self) {
		for state in &self.states {
			let (next_states, matchs) = DfaState::find_next(state, &self.states);
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
	pub fn find_next(state: &StatePtr, current_states: &StateList) -> (HashMap<InputCondition, StateList>, StateList) {
		let mut next_states: HashMap<InputCondition, StateList> = HashMap::new();
		let mut matchs: StateList = StateList::new();

		match &*state.borrow() {
			State::Basic(basic) => {
				if !State::is_none_var_ptr(&basic.out) && !State::is_nomatch_var_ptr(&basic.out) {
					if let RegexType::CharacterClass(class) = &basic.c {
						let next_state = basic.out.borrow();
						for c in class.chars() {
							let condition = InputCondition::Char(c);
							let list = next_states.entry(condition).or_insert_with(|| StateList::new());
							list.add_state(&next_state);
						}
					} else {
						let condition = InputCondition::Char(basic.c.char().expect("Basic state should have a char"));
						let list = next_states.entry(condition).or_insert_with(|| StateList::new());
						list.add_state(&basic.out.borrow());
					}
				}
			},

			State::Split(split) => {
				let (next_states_1, matchs_1) = DfaState::find_next(&*State::deref_var_ptr(&split.out1), current_states);
				let (next_states_2, matchs_2) = DfaState::find_next(&*State::deref_var_ptr(&split.out2), current_states);

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

				current_states.iter().for_each(|state|
					if !State::is_end_of_line_ptr(state) {
						list.add_state(state)
					}
				);
			},

			State::EndOfLine { out } => {
				let list = next_states.entry(InputCondition::EndOfLine).or_insert_with(|| StateList::new());
				list.add_state(&*out.borrow());

				// if end of line, states
				current_states.iter().for_each(|state|
					if !State::is_start_of_line_ptr(state) {
						list.add_state(state)
					}
				);
			},

			State::Match {..} => {
				matchs.add_state(state);
			},

			_ => { eprintln!("Unhandled state: {:?}", state); }
		}

		(next_states, matchs)
	}

	pub fn is_match(&self) -> bool {
		self.matchs.is_empty() == false
	}

	pub fn match_id(&self) -> Option<usize> {

		if self.matchs.is_empty() {
			return  None;
		}

		let mut match_ = usize::MAX;

		for state in &self.matchs {
			match &*state.borrow() {

				State::Match { id } => {
					if *id < match_ {
						match_ = *id;
					}
				},

				_ => panic!("Invalid state")
			}

		}

		Some(match_)
	}
}