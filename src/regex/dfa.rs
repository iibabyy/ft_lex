// use std::{cell::RefCell, collections::{BTreeMap, HashMap}, rc::Rc};
// use super::*;

// pub type DStatePtr = Box<DState>;

// pub struct Dfa {
// 	memo: Vec<Box<Memo>>,

// 	start: DState
// }

// impl Dfa {
// 	pub fn from_nfa(nfa: &Nfa) {
// 		// get next nfa states
// 		// get dstate from next nfa states
// 		//
// 	}


// }

// pub struct DState {
// 	nfa_states: HashMap<char, StateList>
// }

// impl DState {
// 	pub fn new(state: StatePtr) -> Self {



// 		todo!()
// 	}

// }

// pub struct Memo {
// 	id: usize,
// 	states: StateList,
// 	dstate: DStatePtr
// }