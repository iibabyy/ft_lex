use std::{collections::HashSet, hash::Hash, iter::{Enumerate, Peekable}, ops::Deref, rc::Rc};

use super::*;

// ===================================
// 1. DATA STRUCTURES FOR NFA SIMULATION
// ===================================

/// Represents a list of NFA states during simulation
#[derive(Debug)]
pub struct StateList {
    states: Vec<StatePtr>,
}

impl<'a> IntoIterator for &'a StateList {
    type Item = &'a StatePtr;
    type IntoIter = std::slice::Iter<'a, StatePtr>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.states.iter()
    }
}

impl<'a> IntoIterator for &'a mut StateList {
    type Item = &'a mut StatePtr;
    type IntoIter = std::slice::IterMut<'a, StatePtr>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.states.iter_mut()
    }
}

impl IntoIterator for StateList {
    type Item = StatePtr;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.states.into_iter()
    }
}

impl std::fmt::Display for StateList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StateList[")?;
        for (i, state) in self.states.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            match &*state.borrow() {
                State::Basic(basic) => {
                    let char_repr = match basic.c.char() {
                        Some(c) => format!("'{}'", c),
                        None => format!("{:?}", basic.c),
                    };
                    write!(f, "Basic({})", char_repr)?;
                },
                State::Split(_) => write!(f, "Split")?,
                State::Match { id } => write!(f, "Match({})", id)?,
                State::StartOfLine { .. } => write!(f, "StartOfLine")?,
                State::EndOfLine { .. } => write!(f, "EndOfLine")?,
                State::NoMatch => write!(f, "NoMatch")?,
                State::None => write!(f, "None")?,
            }
        }
        write!(f, "]")
    }
}

impl std::ops::Index<usize> for StateList {
    type Output = StatePtr;

    fn index(&self, index: usize) -> &Self::Output {
        &self.states[index]
    }
}


impl Clone for StateList {
	fn clone(&self) -> Self {
		let cloned_states = self.states.iter().map(|state| Rc::clone(state)).collect();

		StateList {
			states: cloned_states
		}
	}
}

impl Eq for StateList {}
impl PartialEq for StateList {
	fn eq(&self, other: &Self) -> bool {
		
		if self.states.len() != other.states.len() {
			return false
		}

		for other_state in &other.states {
			let contained = self.states.iter().any(|state|
				Rc::ptr_eq(state, other_state)
			);

			if contained == false {
				return false
			}
		}

		true
	}
}

impl Hash for StateList {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        // Hash the number of states
        self.states.len().hash(hasher);

        for state_ptr in &self.states {
            let raw_ptr = state_ptr.borrow().deref() as *const State;
            raw_ptr.hash(hasher);
        }
    }
}


impl StateList {
	pub fn new() -> Self {
		StateList { states: Vec::with_capacity(1) }
	}

	pub fn from(state: &StatePtr) -> Self {
		let mut list = StateList::new();

		list.add_state(state);

		list
	}

    pub fn add_state(&mut self, state: &StatePtr) {
        self.add_state_with_memo(state, &mut HashSet::new());
    }

    pub fn add_state_with_memo(&mut self, state: &StatePtr, visited: &mut HashSet<*const State>) {
        let state_ptr = state.borrow().deref() as *const State;

        if visited.insert(state_ptr) == false {
            return;
        }

        if self.contains(state) {
            return;
        }

		if State::is_split_ptr(&state) {
            let borrowed_state = state.borrow();
            let split = borrowed_state.into_split().unwrap();

            // out1
            self.add_state_with_memo(&split.out1.borrow(), visited);
            // out2
            self.add_state_with_memo(&split.out2.borrow(), visited);
        } else {
            self.push(state);
        }
    }

	pub fn add_state_with_memo_iterative(&mut self, state: &StatePtr, visited: &mut HashSet<*const State>) {
		let mut work_stack = Vec::new();
		work_stack.push(Rc::clone(state));
		
		while let Some(current) = work_stack.pop() {
			let state_ptr = current.borrow().deref() as *const State;
			
			if visited.insert(state_ptr) == false {
				continue;
			}
			
			if self.contains(&current) {
				continue;
			}

			if State::is_split_ptr(&current) {
				let borrowed_state = current.borrow();
				let split = borrowed_state.into_split().unwrap();
				
				work_stack.push(Rc::clone(&split.out1.borrow()));
				work_stack.push(Rc::clone(&split.out2.borrow()));
			} else {
				self.push(&current);
			}
		}
	}

	pub fn contains(&self, to_find: &StatePtr) -> bool {
		self.states
			.iter()
			.any(|state|
				Rc::ptr_eq(to_find, state)
			)
	}

	pub fn is_matched(&self) -> bool {
		self.states
		.iter()
		.any(|state|
			State::is_match_ptr(state)
		)
	}

	pub fn remove_matchs(&mut self) -> Vec<StatePtr> {
		let mut indexes = vec![];

		self.states.iter().enumerate().for_each(|(index, state)|
			if State::is_match_ptr(state) {
				indexes.push(index);
			}
		);

		let mut removed = 0;
		let mut matchs = vec![];

		for index in indexes {
			let match_ = self.states.remove(index - removed);
			matchs.push(match_);

			removed += 1;
		}

		matchs
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

	pub fn merge(&mut self, other: StateList) {
		for state in other.states {
			if !self.contains(&state) {
				self.states.push(state);
			}
		}
	}
	
	pub fn len(&self) -> usize {
		self.states.len()
	}
	
	pub fn hash_code(&self) -> u64 {
		use std::hash::{Hash, Hasher};
		use std::collections::hash_map::DefaultHasher;
		
		let mut hasher = DefaultHasher::new();
		self.hash(&mut hasher);
		hasher.finish()
	}

	pub fn enumerate(&self) -> Enumerate<std::slice::Iter<'_, StatePtr>> {
		self.states.iter().enumerate()
	}

	pub fn match_(&self, c: char) -> bool {

		for state in self {
			if state.borrow().matche_with(&c) {
				return true
			}
		}

		false
	}
	
	pub fn iter(&self) -> std::slice::Iter<'_, StatePtr> {
		self.states.iter()
	}
	
	pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, StatePtr> {
		self.states.iter_mut()
	}
}
