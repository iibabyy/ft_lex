// use std::{cell::RefCell, collections::{BTreeMap, HashMap}, rc::Rc};
// use super::*;

// type DStatePtr = MutPtr<DState>;

// static mut state_number: usize = 0;

// pub struct Dfa {
// 	start: DStatePtr
// }

// #[derive(Debug, PartialEq, Eq, Hash)]
// pub enum Literal {
// 	Char(char),
// 	Class(CharacterClass),
// 	Any
// }

// pub struct DState {
// 	id: usize,
// 	nfa_states: StateList,

// 	next: HashMap<char, StateList>,
// }

// impl DState {
// 	pub fn new(states: StateList, id: usize) -> DStatePtr {
// 		Rc::new(RefCell::new(
// 			DState {
// 				id,
// 				nfa_states: states,

// 				next: HashMap::new()
// 			}
// 		))
// 	}

// 	pub fn from_nfa(nfa: Nfa) -> DStatePtr {

// 		let dfa_start = DState::new(StateList::new(), 0);

// 		let nfa_start = nfa.start;

// 		dfa_start.borrow_mut().nfa_states.add_state(&nfa_start);

// 		todo!()
// 	}

// 	pub fn add_next(&mut self) {
// 		let mut paths: HashMap<Literal, StateList> = HashMap::new();

// 		for state in self.nfa_states.iter() {
// 			if State::is_basic_ptr(state) {
// 				let borrowed_state = state.borrow();
// 				let basic = borrowed_state.into_basic().unwrap();

// 				let literal = match &basic.c {
// 					RegexType::Any => Literal::Any,
// 					RegexType::Char(c) => Literal::Char(c.clone()),
// 					RegexType::Class(class) => Literal::Class(class.clone()),

// 					_ => panic!("Invalid basic state character")
// 				};

// 				if let Some(existing_list) = paths.get_mut(&literal) {
// 					existing_list.add_state(state);
// 				}
// 			}
// 		}
// 	}
// }

// pub struct StateMemo {
// 	states: Vec<(StateList, DStatePtr)>
// }

// impl StateMemo {
// 	pub fn new() -> Self {
// 		StateMemo {
// 			states: vec![]
// 		}
// 	}

// 	pub fn get(&mut self, to_find: StateList) -> DStatePtr {
// 		for (list, dstate) in &self.states {
// 			if &to_find == list {
// 				return Rc::clone(dstate)
// 			}
// 		}

// 		// Not found
// 		let dstate = DState::new(to_find.clone());

// 		self.states.push((to_find, Rc::clone(&dstate)));

// 		return dstate;
// 	}
// }
