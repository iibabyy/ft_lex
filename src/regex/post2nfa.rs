use std::{
    cell::{RefCell, UnsafeCell}, collections::HashMap, fmt, hash::Hash, ops::{Deref, DerefMut}, rc::{Rc, Weak}
};

use super::*;
use utils::*;

// 1. BASIC TYPE DEFINITIONS
// =========================

/// Represents a C-like pointer to a pointer to a State (e.g. State**)
pub type VarStatePtr = MutPtr<StatePtr>;
/// Represents a C-like pointer to a State (e.g. State*)
pub type StatePtr = MutPtr<State>;
/// Allow both shareability with Rc and mutability with RefCell
pub type MutPtr<T> = Rc<RefCell<T>>;

pub fn state_ptr(state: State) -> StatePtr {
    Rc::new(RefCell::new(state))
}

pub fn var_state_ptr(state: StatePtr) -> VarStatePtr {
    Rc::new(RefCell::new(state))
}

#[derive(Debug)]
pub enum State {
    Basic(BasicState),
    Split(SplitState),
    NoMatch,
    Match {
		// Allows to knows wich pattern have matched
		id: usize
	},
    None,
    StartOfLine{ out: VarStatePtr },
    EndOfLine{ out: VarStatePtr },
}

impl Hash for State {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        // Hash the discriminant to differentiate between variants
        std::mem::discriminant(self).hash(hasher);
        
        match self {
            State::Basic(basic) => {
                basic.hash(hasher);
            },
            State::Split(split) => {
                split.hash(hasher);
            },
            State::StartOfLine { out } => {
                let out_ptr = Rc::as_ptr(&*out.borrow());
                out_ptr.hash(hasher);
            },
            State::EndOfLine { out } => {
                let out_ptr = Rc::as_ptr(&*out.borrow());
                out_ptr.hash(hasher);
            },
            // NoMatch, Match, and None don't have additional data to hash
            // Use constant numbers to ensure consistent hashing
            State::NoMatch => { 1u8.hash(hasher); },
            State::Match { id } => {
				2u8.hash(hasher);
				id.hash(hasher);
			},
            State::None => { 3u8.hash(hasher); }
        }
    }
}


pub struct BasicState {
    pub c: RegexType,
    pub out: VarStatePtr,
}

impl Hash for BasicState {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        self.c.hash(hasher);
        let out_ptr = Rc::as_ptr(&*self.out.borrow());
        out_ptr.hash(hasher);
    }
}


pub struct SplitState {
    pub out1: VarStatePtr,
    pub out2: VarStatePtr,
}

impl Hash for SplitState {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        let out1_ptr = Rc::as_ptr(&*self.out1.borrow());
        out1_ptr.hash(hasher);

        let out2_ptr = Rc::as_ptr(&*self.out2.borrow());
        out2_ptr.hash(hasher);
    }
}

/// In the NFA, a Fragment is a list of states that can be matched
/// 
/// Any pattern can be represented by a Fragment:
/// 
/// Basic pattern (one char) -> Fragment with one state
/// 
/// For more complex patterns, the fragments can be combined using the `and`, `or`, `optional`, `optional_repeat`, `exact_repeat`, `at_least`, `range` methods
#[derive(Debug)]
pub struct Fragment {
    pub start: StatePtr,
    pub ptr_list: Vec<VarStatePtr>,
}

// 2. TYPE-SPECIFIC METHODS
// ========================

impl State {
    pub fn basic(litteral: RegexType) -> StatePtr {
        let state = Self::Basic(BasicState {
            c: litteral,
            out: var_state_ptr(State::none()),
        });

        state_ptr(state)
    }

    pub fn split(out1: StatePtr, out2: StatePtr) -> StatePtr {
        let state = Self::Split(SplitState {
            out1: var_state_ptr(out1),
            out2: var_state_ptr(out2),
        });

        state_ptr(state)
    }

    pub fn match_(id: usize) -> StatePtr {
        state_ptr(State::Match { id })
    }

    pub fn no_match() -> StatePtr {
        state_ptr(State::NoMatch)
    }

    pub fn none() -> StatePtr {
        state_ptr(State::None)
    }

    pub fn start_of_line() -> StatePtr {
        state_ptr(State::StartOfLine {
            out: var_state_ptr(State::none()),
        })
    }

    pub fn end_of_line() -> StatePtr {
        state_ptr(State::EndOfLine {
            out: var_state_ptr(State::none()),
        })
    }

    pub fn is_none(&self) -> bool {
        matches!(self, State::None)
    }

    pub fn is_basic(&self) -> bool {
        matches!(self, State::Basic(_))
    }

    pub fn is_split(&self) -> bool {
        matches!(self, State::Split(_))
    }

    pub fn is_match(&self) -> bool {
        matches!(self, State::Match { .. })
    }

    pub fn is_nomatch(&self) -> bool {
        matches!(self, State::NoMatch)
    }

    pub fn is_start_of_line(&self) -> bool {
        matches!(self, State::StartOfLine { .. })
    }

    pub fn is_end_of_line(&self) -> bool {
        matches!(self, State::EndOfLine { .. })
    }

    pub fn is_basic_ptr(ptr: &StatePtr) -> bool {
        ptr.borrow().is_basic()
    }

    pub fn is_split_ptr(ptr: &StatePtr) -> bool {
        ptr.borrow().is_split()
    }

    pub fn is_none_ptr(ptr: &StatePtr) -> bool {
        ptr.borrow().is_none()
    }

    pub fn is_match_ptr(ptr: &StatePtr) -> bool {
        ptr.borrow().is_match()
    }

    pub fn is_nomatch_ptr(ptr: &StatePtr) -> bool {
        ptr.borrow().is_nomatch()
    }

    pub fn is_start_of_line_ptr(ptr: &StatePtr) -> bool {
        ptr.borrow().is_start_of_line()
    }

    pub fn is_end_of_line_ptr(ptr: &StatePtr) -> bool {
        ptr.borrow().is_end_of_line()
    }

    pub fn is_basic_var_ptr(ptr: &VarStatePtr) -> bool {
        ptr.borrow().borrow().is_basic()
    }

    pub fn is_split_var_ptr(ptr: &VarStatePtr) -> bool {
        ptr.borrow().borrow().is_split()
    }

    pub fn is_none_var_ptr(ptr: &VarStatePtr) -> bool {
        ptr.borrow().borrow().is_none()
    }

    pub fn is_match_var_ptr(ptr: &VarStatePtr) -> bool {
        ptr.borrow().borrow().is_match()
    }

    pub fn is_nomatch_var_ptr(ptr: &VarStatePtr) -> bool {
        ptr.borrow().borrow().is_nomatch()
    }

    pub fn is_start_of_line_var_ptr(ptr: &VarStatePtr) -> bool {
        ptr.borrow().borrow().is_start_of_line()
    }

    pub fn is_end_of_line_var_ptr(ptr: &VarStatePtr) -> bool {
        ptr.borrow().borrow().is_end_of_line()
    }

    pub fn from_ptr(ptr: &StatePtr) -> std::cell::Ref<'_, Self> {
        ptr.borrow()
    }

    pub fn deref_var_ptr(ptr: &VarStatePtr) -> std::cell::Ref<'_, StatePtr> {
        ptr.borrow()
    }

    pub fn start_of_line_out(&self) -> Option<VarStatePtr> {
        match self {
            State::StartOfLine { out } => Some(Rc::clone(out)),
            
            _ => None,
        }
    }
    
    pub fn end_of_line_out(&self) -> Option<VarStatePtr> {
        match self {
            State::EndOfLine { out } => Some(Rc::clone(out)),
            
            _ => None,
        }
    }

    pub fn basic_out(&self) -> Option<VarStatePtr> {
        match self {
            State::Basic(state) => Some(Rc::clone(&state.out)),

            _ => None,
        }
    }

    pub fn split_out(&self) -> Option<(VarStatePtr, VarStatePtr)> {
        match self {
            State::Split(state) => {
                let ptr1 = Rc::clone(&state.out1);
                let ptr2 = Rc::clone(&state.out2);

                Some((ptr1, ptr2))
            }

            _ => None,
        }
    }

    pub fn into_split(&self) -> Option<&SplitState> {
        match self {
            Self::Split(split) => Some(split),

            _ => None,
        }
    }

    pub fn into_basic(&self) -> Option<&BasicState> {
        match self {
            Self::Basic(basic) => Some(basic),

            _ => None,
        }
    }

    /// Needed for reusing the same fragment (e.g repeting a fragment)
    fn self_ptr_deep_clone(&self) -> (StatePtr, Vec<VarStatePtr>) {
        Self::self_ptr_deep_clone_with_memo_iterative(self, &mut HashMap::new())
    }
    
	#[deprecated(note="please use `self_ptr_deep_clone_with_memo_iterative` instead")]
    pub fn self_ptr_deep_clone_with_memo_recursive(
        &self, 
        memo: &mut HashMap<*const State, StatePtr>
    ) -> (StatePtr, Vec<VarStatePtr>) {
        // Get raw pointer for use as HashMap key
        let self_ptr = self as *const State;
        
        // If we've already cloned this state, return the cached clone
        if let Some(cached_clone) = memo.get(&self_ptr) {
            return (Rc::clone(cached_clone), vec![]);
        }
        
        match self {
            State::Basic(basic) => {
                let cloned_regex = basic.c.clone();
                
                // Create empty state first so we can insert it into the memo table
                let state = state_ptr(State::Basic(BasicState {
                    c: cloned_regex.clone(),
                    out: var_state_ptr(State::none()),
                }));
                
                // Insert the new state into memo table before recursing
                memo.insert(self_ptr, Rc::clone(&state));
                
                // Now safely recursively clone the out state
                let out_ref = &basic.out.borrow();
                if !State::is_none_ptr(out_ref) {
                    let (cloned_out, cloned_ptr_list) = 
                        Self::deep_clone_with_memo(out_ref, memo);
                    
                    // Update the out pointer
                    state.borrow_mut().into_basic().unwrap().out.replace(cloned_out);
                    
                    return (state, cloned_ptr_list);
                } else {
                    let ptr = state.borrow().basic_out().unwrap();
                    return (state, vec![ptr]);
                }
            }

            State::Split(split) => {
                let (cloned_out1, cloned_ptr_list1) = Self::deep_clone_with_memo(&split.out1.borrow(), memo);
                let cloned_1_is_some = State::is_none_ptr(&cloned_out1) == false;

                let (cloned_out2, cloned_ptr_list2) = Self::deep_clone_with_memo(&split.out2.borrow(), memo);
                let cloned_2_is_some = State::is_none_ptr(&cloned_out2) == false;

                let state = State::split(cloned_out1, cloned_out2);

                let mut ptr_list1 = if cloned_1_is_some {
                    cloned_ptr_list1
                } else {
                    let ptr1 = state.borrow().split_out().unwrap().0;

                    vec![ptr1]
                };

                let prt_list_2 = if cloned_2_is_some {
                    cloned_ptr_list2
                } else {
                    let ptr2 = state.borrow().split_out().unwrap().1;

                    vec![ptr2]
                };

                ptr_list1.extend(prt_list_2);

                (state, ptr_list1)
            }

            State::Match { id } => (State::match_(*id), vec![]),

            State::NoMatch => (State::no_match(), vec![]),

            State::None => (State::none(), vec![]),

            State::EndOfLine { out } => {
                let (cloned_out, ptr_list) = Self::deep_clone_with_memo(&out.borrow(), memo);

                let cloned_state = state_ptr(State::EndOfLine { out: var_state_ptr(cloned_out) });

                (cloned_state, ptr_list)
            }
            
            State::StartOfLine { out } => {
                let (cloned_out, ptr_list) = Self::deep_clone_with_memo(&out.borrow(), memo);

                let cloned_state = state_ptr(State::StartOfLine { out: var_state_ptr(cloned_out) });

                (cloned_state, ptr_list)
            }
        }
    }

    pub fn deep_clone(state: &StatePtr) -> (StatePtr, Vec<VarStatePtr>) {
        Self::deep_clone_with_memo(state, &mut HashMap::new())
    }
    
    fn deep_clone_with_memo(
        state: &StatePtr, 
        memo: &mut HashMap<*const State, StatePtr>
    ) -> (StatePtr, Vec<VarStatePtr>) {
        if State::is_none_ptr(state) {
            return (State::none(), vec![]);
        }
        
        let state_ref = state.borrow();
        state_ref.self_ptr_deep_clone_with_memo_iterative(memo)
    }

    pub fn matche_with(&self, c: &char) -> bool {
        match self {
            Self::Basic(basic) => basic.c.match_(&c),

            _ => false,
        }
    }

	pub fn match_id(&self) -> Option<usize> {
		match self {
			Self::Match { id } => Some(*id),

			_ => None,
		}
	}

}

// Stack for pending work (state to clone, parent reference, field to update)
struct WorkItem {
	state: StatePtr,
	cloned: Option<StatePtr>,
	parent: Option<StatePtr>,
	is_first_out: bool, // true = out1/out, false = out2
}

impl State {
	pub fn self_ptr_deep_clone_with_memo_iterative(
		&self,
		memo: &mut HashMap<*const State, StatePtr>
	) -> (StatePtr, Vec<VarStatePtr>) {

		let mut work_stack = Vec::new();
		let mut result_ptr_list = Vec::new();
		
		// Start with the current state
		let self_ptr = self as *const State;
		
		// If we've already cloned this state, return the cached clone
		if let Some(cached_clone) = memo.get(&self_ptr) {
			return (Rc::clone(cached_clone), vec![]);
		}
		
		// Initialize result based on the type of current state
		let (initial_result, add_to_stack) = self.initialize_clone(memo, &mut result_ptr_list);
		
		// Add our initial state's outgoing states to the work stack
		if add_to_stack {
			self.add_outgoing_states_to_stack(&initial_result, &mut work_stack);
		}
		
		// Process the work stack
		self.process_work_stack(&mut work_stack, memo, &mut result_ptr_list);
		
		(initial_result, result_ptr_list)
	}
	
	fn initialize_clone(
		&self,
		memo: &mut HashMap<*const State, StatePtr>,
		result_ptr_list: &mut Vec<VarStatePtr>
	) -> (StatePtr, bool) {
		let self_ptr = self as *const State;
		
		match self {
			State::Basic(basic) => {
				let cloned_regex = basic.c.clone();
				
				// Create empty state with placeholder out pointer
				let state = state_ptr(State::Basic(BasicState {
					c: cloned_regex.clone(),
					out: var_state_ptr(State::none()),
				}));
				
				// Insert into memo table
				memo.insert(self_ptr, Rc::clone(&state));
				
				// Add out state to work stack if it's not None
				let out_ref = basic.out.borrow();
				let should_add = !State::is_none_ptr(&out_ref);
				
				if should_add {
					(state, true)
				} else {
					// If out is None, add the out pointer to result_ptr_list
					let ptr = state.borrow().basic_out().unwrap();
					result_ptr_list.push(ptr);
					(state, false)
				}
			},
			
			State::Split(split) => {
				// For split states, we need to clone both out1 and out2
				// Create a split state with placeholder out pointers
				let state = State::split(State::none(), State::none());

				// Insert into memo table
				memo.insert(self_ptr, Rc::clone(&state));
				
				// Check if out1 or out2 are None, if so add them to ptr_list
				let out1_ref = split.out1.borrow();
				let out2_ref = split.out2.borrow();
				
				let out1_is_none = State::is_none_ptr(&out1_ref);
				let out2_is_none = State::is_none_ptr(&out2_ref);
				
				if out1_is_none || out2_is_none {
					let (out1, out2) = state.borrow().split_out().unwrap();
					
					if out1_is_none {
						result_ptr_list.push(out1);
					}
					
					if out2_is_none {
						result_ptr_list.push(out2);
					}
				}
				
				(state, true)
			},
			
			State::Match { id } => (State::match_(*id), false),
			State::NoMatch => (State::no_match(), false),
			State::None => (State::none(), false),
			
			State::StartOfLine { out } => {
				let state = state_ptr(State::StartOfLine { 
					out: var_state_ptr(State::none()) 
				});
				
				memo.insert(self_ptr, Rc::clone(&state));
				
				let out_ref = out.borrow();
				let should_add = !State::is_none_ptr(&out_ref);
				
				if should_add {
					(state, true)
				} else {
					// If out is None, add the out pointer to result_ptr_list
					let ptr = state.borrow().basic_out().unwrap();
					result_ptr_list.push(ptr);
					(state, false)
				}
			},
			
			State::EndOfLine { out } => {
				let state = state_ptr(State::EndOfLine { 
					out: var_state_ptr(State::none()) 
				});
				
				memo.insert(self_ptr, Rc::clone(&state));
				
				let out_ref = out.borrow();
				let should_add = !State::is_none_ptr(&out_ref);
				
				if should_add {
					(state, true)
				} else {
					// If out is None, add the out pointer to result_ptr_list
					let ptr = state.borrow().basic_out().unwrap();
					result_ptr_list.push(ptr);
					(state, false)
				}
			},
		}
	}
	
	fn add_outgoing_states_to_stack(
		&self,
		initial_result: &StatePtr,
		work_stack: &mut Vec<WorkItem>
	) {
		match self {
			State::Basic(basic) => {
				let out_ref = Rc::clone(&basic.out.borrow());
				work_stack.push(WorkItem {
					state: out_ref,
					cloned: None,
					parent: Some(Rc::clone(initial_result)),
					is_first_out: true,
				});
			},
			
			State::Split(split) => {
				let out1_ref = Rc::clone(&split.out1.borrow());
				let out2_ref = Rc::clone(&split.out2.borrow());
				
				work_stack.push(WorkItem {
					state: out1_ref,
					cloned: None,
					parent: Some(Rc::clone(initial_result)),
					is_first_out: true,
				});
				
				work_stack.push(WorkItem {
					state: out2_ref,
					cloned: None,
					parent: Some(Rc::clone(initial_result)),
					is_first_out: false,
				});
			},
			
			State::StartOfLine { out } | State::EndOfLine { out } => {
				let out_ref = Rc::clone(&out.borrow());
				work_stack.push(WorkItem {
					state: out_ref,
					cloned: None,
					parent: Some(Rc::clone(initial_result)),
					is_first_out: true,
				});
			},
			
			_ => { /* No outgoing states to process */ }
		}
	}
	
	fn process_work_stack(
		&self,
		work_stack: &mut Vec<WorkItem>,
		memo: &mut HashMap<*const State, StatePtr>,
		result_ptr_list: &mut Vec<VarStatePtr>
	) {
		while let Some(work_item) = work_stack.pop() {
			if State::is_none_ptr(&work_item.state) {
				continue;
			}
			
			let raw_ptr = work_item.state.borrow().deref() as *const State;
			
			// Check if we've already cloned this state
			if let Some(cached_clone) = memo.get(&raw_ptr) {
				// Update parent pointer
				if let Some(parent) = &work_item.parent {
					State::update_parent_pointer(parent, Rc::clone(cached_clone), work_item.is_first_out);
				}
				continue;
			}
			
			// Clone current state
			let state_ref = &*work_item.state.borrow();
			let (new_state, child_items) = Self::clone_state(state_ref, raw_ptr, memo, work_stack);
			
			// Update parent pointer if this state has a parent
			if let Some(parent) = &work_item.parent {
				State::update_parent_pointer(parent, Rc::clone(&new_state), work_item.is_first_out);
			}
			
			// Add any new pointers to the result list
			result_ptr_list.extend(child_items);
		}
	}
	
	fn clone_state(
		state_ref: &State,
		raw_ptr: *const State,
		memo: &mut HashMap<*const State, StatePtr>,
		work_stack: &mut Vec<WorkItem>
	) -> (StatePtr, Vec<VarStatePtr>) {
		match state_ref {
			State::Basic(basic) => {
				let cloned_regex = basic.c.clone();
				
				// Create empty state with placeholder out pointer
				let state = state_ptr(State::Basic(BasicState {
					c: cloned_regex.clone(),
					out: var_state_ptr(State::none()),
				}));
				
				// Insert into memo table
				memo.insert(raw_ptr, Rc::clone(&state));
				
				// Add out state to work stack
				let out_ref = Rc::clone(&basic.out.borrow());
				
				if !State::is_none_ptr(&out_ref) {
					work_stack.push(WorkItem {
						state: out_ref,
						cloned: None,
						parent: Some(Rc::clone(&state)),
						is_first_out: true,
					});
					(state, vec![])
				} else {
					// If out is None, add the out pointer to result_ptr_list
					let ptr = state.borrow().basic_out().unwrap();
					(state, vec![ptr])
				}
			},
			
			State::Split(split) => {
				// For split states, we need to clone both out1 and out2
				let state = State::split(State::none(), State::none());
				
				// Insert into memo table
				memo.insert(raw_ptr, Rc::clone(&state));
				
				// Add both outputs to work stack
				let out1_ref = Rc::clone(&split.out1.borrow());
				let out2_ref = Rc::clone(&split.out2.borrow());
				
				let mut ptr_list = vec![];
				
				// Handle out1
				if State::is_none_ptr(&out1_ref) {
					// If out1 is None, add the out1 pointer to result_ptr_list
					let (out1, _) = state.borrow().split_out().unwrap();
					ptr_list.push(out1);
				} else {
					work_stack.push(WorkItem {
						state: out1_ref,
						cloned: None,
						parent: Some(Rc::clone(&state)),
						is_first_out: true,
					});
				}
				
				// Handle out2
				if State::is_none_ptr(&out2_ref) {
					// If out2 is None, add the out2 pointer to result_ptr_list
					let (_, out2) = state.borrow().split_out().unwrap();
					ptr_list.push(out2);
				} else {
					work_stack.push(WorkItem {
						state: out2_ref,
						cloned: None,
						parent: Some(Rc::clone(&state)),
						is_first_out: false,
					});
				}
				
				(state, ptr_list)
			},
			
			State::Match { id } => {
				let state = State::match_(*id);
				memo.insert(raw_ptr, Rc::clone(&state));
				(state, vec![])
			},
			State::NoMatch => {
				let state = State::no_match();
				memo.insert(raw_ptr, Rc::clone(&state));
				(state, vec![])
			},
			State::None => {
				let state = State::none();
				memo.insert(raw_ptr, Rc::clone(&state));
				(state, vec![])
			},
			
			State::StartOfLine { out } => {
				let state = state_ptr(State::StartOfLine { 
					out: var_state_ptr(State::none()) 
				});
				
				memo.insert(raw_ptr, Rc::clone(&state));
				
				let out_ref = Rc::clone(&out.borrow());
				
				let mut ptr_list = vec![];
				
				if State::is_none_ptr(&out_ref) {
					// If out is None, add the out pointer to result_ptr_list
					if let Some(out_ptr) = state.borrow().start_of_line_out() {
						ptr_list.push(out_ptr);
					}
				} else {
					work_stack.push(WorkItem {
						state: out_ref,
						cloned: None,
						parent: Some(Rc::clone(&state)),
						is_first_out: true,
					});
				}
				
				(state, ptr_list)
			},
			
			State::EndOfLine { out } => {
				let state = state_ptr(State::EndOfLine { 
					out: var_state_ptr(State::none()) 
				});
				
				memo.insert(raw_ptr, Rc::clone(&state));
				
				let out_ref = Rc::clone(&out.borrow());
				
				let mut ptr_list = vec![];
				
				if State::is_none_ptr(&out_ref) {
					// If out is None, add the out pointer to result_ptr_list
					if let Some(out_ptr) = state.borrow().end_of_line_out() {
						ptr_list.push(out_ptr);
					}
				} else {
					work_stack.push(WorkItem {
						state: out_ref,
						cloned: None,
						parent: Some(Rc::clone(&state)),
						is_first_out: true,
					});
				}
				
				(state, ptr_list)
			},
		}
	}
	
	// Helper function to update a parent state's outgoing pointers
	fn update_parent_pointer(parent: &StatePtr, child: StatePtr, is_first_out: bool) {
		let parent_ref = parent.borrow();
		
		if let Some(_) = parent_ref.into_basic() {
			drop(parent_ref); // Release the immutable borrow
			parent.borrow_mut().into_basic().unwrap().out.replace(child);
		} else if let Some(_) = parent_ref.into_split() {
			drop(parent_ref); // Release the immutable borrow
			if is_first_out {
				parent.borrow_mut().into_split().unwrap().out1.replace(child);
			} else {
				parent.borrow_mut().into_split().unwrap().out2.replace(child);
			}
		} else if parent_ref.is_start_of_line() {
			if let Some(out) = parent_ref.start_of_line_out() {
				out.replace(child);
			}
		} else if parent_ref.is_end_of_line() {
			if let Some(out) = parent_ref.end_of_line_out() {
				out.replace(child);
			}
		}
		// No else case needed - no outgoing pointers to update for other state types
	}


}

impl Fragment {
    pub fn new(start: StatePtr, ptr_list: Vec<VarStatePtr>) -> Self {
        Self { start, ptr_list }
    }

	pub fn start_of_line(self) -> Self {
		let start = State::start_of_line();
		let out = start.borrow().start_of_line_out().unwrap();
		
		Fragment::new(start, vec![out]).and(self)
	}

	pub fn end_of_line(self) -> Self {
		let start = State::end_of_line();
		let out = start.borrow().end_of_line_out().unwrap();

		self.and(Fragment::new(start, vec![out]))
	}

    pub fn basic(start: StatePtr) -> Self {
        let ptr = start.borrow().basic_out().unwrap();

        let frag = Fragment {
            start,
            ptr_list: vec![ptr],
        };

        return frag;
    }

    pub fn and(self, e2: Self) -> Self {
        utils::patch(&self.ptr_list, &e2.start);

        Fragment::new(self.start, e2.ptr_list)
    }

    /// Creates an OR operation in the NFA by using a Split state to branch between two fragments.
    /// This implements the alternation (|) operation in regular expressions.
    /// The Split state allows the NFA to follow either path during matching.
    pub fn or(self, e2: Self) -> Self {
        let s = State::split(self.start, e2.start);

        Fragment::new(s, utils::append(self.ptr_list, e2.ptr_list))
    }

    /// Creates an OR operation with a None state, allowing the pattern to be skipped.
    /// This is similar to the `or` operation but instead of branching between two fragments,
    /// it branches between the fragment and a None state.
    /// 
    /// This is used in implementing optional patterns and other quantifiers where
    /// one path needs to bypass the pattern entirely.
    pub fn or_none(self) -> Self {
        let s = State::split(self.start, State::none());
        
        let none_out = s.borrow().split_out().unwrap().1;
        
        let ptr_list = utils::append(self.ptr_list, utils::list1(none_out));
        
        Fragment::new(s, ptr_list)
    }

    pub fn optional(self) -> Self {
        let s = State::split(self.start, State::none());

        let none_out = s.borrow().split_out().unwrap().1;

        let ptr_list = utils::append(self.ptr_list, utils::list1(none_out));

        Fragment::new(s, ptr_list)
    }

    /// Implements the Kleene star (*) operation, which matches zero or more repetitions of the pattern.
    /// Unlike optional(), which matches 0 or 1 occurrence, this allows unlimited repetitions.
    /// This creates a split state that can either skip the pattern (matching 0 times) or
    /// enter the pattern and then loop back to the split state after completion (allowing multiple matches).
    /// 
    /// This is one of several quantifiers that match at least 0 occurrences:
    /// - optional_repeat(*): matches 0 or more times
    /// - range({0,n}): matches between 0 and n times
    /// - at_least({0,}): equivalent to optional_repeat (matches 0 or more times)
    pub fn optional_repeat(self) -> Self {
        let s = State::split(self.start, State::none());

        utils::patch(&self.ptr_list, &s);

        let none_out = s.borrow().split_out().unwrap().1;

        let ptr_list = utils::list1(none_out);

        Fragment::new(s, ptr_list)
    }

    pub fn exact_repeat(self, n: &usize) -> Self {
        let mut fragment = self.deep_clone();
        let n = *n;

        if n == 0 {
            utils::patch(&fragment.ptr_list, &State::no_match());

            return Fragment::new(fragment.start, vec![]);
        }

        for _ in 1..n {
            let cloned_fragment = self.deep_clone();

            fragment = fragment.and(cloned_fragment);
        }

        fragment
    }

    pub fn at_least(self, n: &usize) -> Self {
        if n > &0 {
            let clone = self.deep_clone();

            let repeat = self.exact_repeat(n);
            let optional = clone.optional_repeat();

			repeat.and(optional)
        } else {
            self.optional_repeat()
        }
    }

    pub fn range(self, at_least: &usize, at_most: &usize) -> Self {
        let optional_count = at_most - at_least;

		if optional_count > 0 {
			let fragment = if at_least > &0 {
                Some(self.deep_clone().exact_repeat(at_least))
            } else {
                None
            };

            let mut optional_part = self.deep_clone().optional();

            for _ in 1..optional_count {
                let next_optional = self.deep_clone();
                optional_part = optional_part.and(next_optional.optional());
            }

            optional_part = optional_part;

			if fragment.is_none() {
				// at_least == 0
				optional_part
			} else {
				fragment.unwrap().and(optional_part)
			}

        } else if optional_count == 0 {
            return self.exact_repeat(at_least);
        } else {
            panic!("Invalid Range parameter")
        }
    }

    pub fn deep_clone(&self) -> Self {
        let (cloned_start, cloned_ptr_list) = State::deep_clone(&self.start);

        Self {
            start: cloned_start,
            ptr_list: cloned_ptr_list,
        }
    }

    /// Yes, this is how regex quantifiers are handled in the NFA:
	/// 
    /// '*' (zero or more) -> implemented as optional_repeat()
	/// 
    /// '+' (one or more) -> implemented as at_least(1)
    /// 
    /// '?' (zero or one) -> implemented as optional()
    /// 
    /// '{n}' (exactly n) -> implemented as exact_repeat(n)
    /// 
    /// '{n,}' (n or more) -> implemented as at_least(n)
    /// 
    /// '{n,m}' (between n and m) -> implemented as range(n,m)
    pub fn quantify(self, quantifier: &Quantifier) -> Self {
        match quantifier {
            // {n}
            Quantifier::Exact(n) => self.exact_repeat(n),

            // {n,}
            Quantifier::AtLeast(n) => self.at_least(n),

            // {n, m}
            Quantifier::Range(at_least, at_most) => self.range(at_least, at_most),
        }
    }
}

/// Represents the NFA (Non-deterministic Finite Automaton)
///
/// end of line and start of line are handled as flags in the NFA
#[derive(Debug)]
pub struct Nfa {
    pub start: StatePtr,

    pub end_of_line: bool,
    pub start_of_line: bool,
}

impl Nfa {
    pub fn new() -> Self {
        Nfa {
            start: State::none(),
            end_of_line: false,
            start_of_line: false,
        }
    }
}

// 3. DISPLAY IMPLEMENTATIONS
// =========================

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            State::Basic(basic) => write!(
                f,
                "{}",
                basic
                    .out
                    .borrow()
                    .borrow()
                    .is_none()
                    .then_some("...")
                    .unwrap_or("None")
            ),

            State::NoMatch => write!(f, "NoMatch()"),

            State::Match { id } => write!(f, "Match[{id}]"),

            State::None => write!(f, "None"),

            State::Split(split) => write!(
                f,
                "Split({:?}, {:?})",
                split
                    .out1
                    .borrow()
                    .borrow()
                    .is_none()
                    .then_some("...")
                    .unwrap_or("None"),
                split
                    .out1
                    .borrow()
                    .borrow()
                    .is_none()
                    .then_some("...")
                    .unwrap_or("None")
            ),
            
            State::StartOfLine { out } => write!(
                f,
                "StartOfLine({})",
                out
                    .borrow()
                    .borrow()
                    .is_none()
                    .then_some("...")
                    .unwrap_or("None")
            ),
            
            State::EndOfLine { out } => write!(
                f,
                "EndOfLine({})",
                out
                    .borrow()
                    .borrow()
                    .is_none()
                    .then_some("...")
                    .unwrap_or("None")
            ),
        }
    }
}

impl fmt::Display for BasicState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ c: {}, out: ... }}", self.c)
    }
}

impl fmt::Display for SplitState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ out1: {:?}, out2: {:?} }}",
            State::is_none_var_ptr(&self.out1)
                .then_some("None")
                .unwrap_or("..."),
            State::is_none_var_ptr(&self.out2)
                .then_some("None")
                .unwrap_or("..."),
        )
    }
}

impl fmt::Display for Fragment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Fragment {{ start: {:?}, ptr_list: [{}] }}",
            State::is_none_ptr(&self.start)
                .then_some(self.start.borrow().to_string().as_str())
                .unwrap_or("None"),
            self.ptr_list.len()
        )
    }
}

impl fmt::Debug for BasicState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Debug for SplitState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}


// 4. NFA CONSTRUCTION FUNCTIONS
// =============================

/// This function implements Thompson's construction algorithm to convert the postfix regex to an NFA
pub fn post2nfa(mut postfix: VecDeque<TokenType>, id: usize) -> ParsingResult<StatePtr> {
	if postfix.is_empty() {
		return Err(ParsingError::unrecognized_rule());
	}

	let mut start_of_line = false;
	let mut end_of_line = false;
    let mut fragments: Vec<Fragment> = vec![];

    while let Some(token) = postfix.pop_front() {
        match token.into_owned_inner() {
            RegexType::Concatenation => {
                let e2 = fragments.pop().ok_or(ParsingError::unrecognized_rule())?;

                let e1 = fragments.pop().ok_or(ParsingError::unrecognized_rule())?;

                fragments.push(e1.and(e2));
            }

            RegexType::Or => {
                let e2 = fragments
                    .pop()
                    .ok_or(ParsingError::unrecognized_rule().because("Unexpected '|'"))?;

                let e1 = fragments
                    .pop()
                    .ok_or(ParsingError::unrecognized_rule().because("Unexpected '|'"))?;

                fragments.push(e1.or(e2));
            }

            RegexType::Quant(quantifier) => {
                let e = fragments
                    .pop()
                    .ok_or(ParsingError::unrecognized_rule().because("Unexpected quantifier"))?;

				fragments.push(e.quantify(&quantifier));
			}

            RegexType::LineEnd => {
                if end_of_line == true || postfix.front().is_some() {
                    return Err(ParsingError::unrecognized_rule()
                        .because("unexpected '$' special character"));
                }

                end_of_line = true;
            }

            RegexType::LineStart => {
                if start_of_line == true || fragments.last().is_some() {
                    return Err(ParsingError::unrecognized_rule()
                        .because("unexpected '^' special character"));
                }

                start_of_line = true;
            }

            c => {
                let s = State::basic(c);

                let frag = Fragment::basic(s);

                fragments.push(frag);
            }
        }
    }

	if fragments.len() > 1 {
		return Err(ParsingError::unrecognized_rule());
	} else if fragments.len() == 0 {
		return Err(ParsingError::unrecognized_rule());
    }

    let mut e = fragments.pop().unwrap();

    if State::is_none_ptr(&e.start) {
        return Err(ParsingError::unrecognized_rule());
    }

	if start_of_line {
		e = e.start_of_line();
	}

	if end_of_line {
		e = e.end_of_line();
	}

    utils::last_patch(&e.ptr_list, id);

	Ok(e.start)
}

// 4. UTILITY FUNCTIONS
// ====================

pub mod utils {
    use super::*;

    pub fn last_patch(ptr_list: &Vec<VarStatePtr>, id: usize) {
        utils::patch(ptr_list, &State::match_(id));
    }

    /// It connects dangling transitions to a specific state
    pub fn patch(ptr_list: &Vec<VarStatePtr>, state: &StatePtr) {
        for ptr in ptr_list {
            ptr.replace(Rc::clone(state));
        }
    }

    pub fn list1(endpoint: VarStatePtr) -> Vec<VarStatePtr> {
        vec![endpoint]
    }

    pub fn append(mut list1: Vec<VarStatePtr>, list2: Vec<VarStatePtr>) -> Vec<VarStatePtr> {
        list1.extend(list2);

        list1
    }
}
