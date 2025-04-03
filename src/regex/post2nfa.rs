use std::{cell::RefCell, fmt, ops::{Deref, DerefMut}, rc::{Rc, Weak}};

use super::*;
use utils::*;

// 1. BASIC TYPE DEFINITIONS
// =========================

pub type StatePtr = Option<Rc<State>>;

#[derive(Debug)]
pub enum State {
	Basic(BasicState),
	Split(SplitState),
	NoMatch,
	Match
}

#[derive(Debug)]
pub struct BasicState {
	pub c: RegexType,
	pub out: StatePtr,
}

#[derive(Debug)]
pub struct SplitState {
	pub out1: StatePtr,
	pub out2: StatePtr,
}

#[derive(Debug)]
pub struct Fragment {
	pub start: StatePtr,
	pub ptr_list: Vec<*mut StatePtr>,
}

// 2. TYPE-SPECIFIC METHODS
// ========================

impl State {
	pub fn basic(litteral: RegexType) -> StatePtr {
		let res = Self::Basic(BasicState {
			c: litteral,
			out: None,
		});

		Some(Rc::new(res))
	}

	pub fn split(out1: StatePtr, out2: StatePtr) -> StatePtr {
		let res = Self::Split(SplitState {
			out1,
			out2,
		});

		Some(Rc::new(res))
	}

	pub fn matche() -> StatePtr {
		let res = Self::Match;

		Some(Rc::new(res))
	}

	pub fn no_match() -> StatePtr {
		let res = Self::NoMatch;

		Some(Rc::new(res))
	}

	pub fn null() -> StatePtr {
		None
	}

	pub fn basic_out(&mut self) -> Option<*mut StatePtr> {
		match self {
			State::Basic(state) => {
				let ptr = &mut state.out as *mut Option<Rc<State>>;
				Some(ptr)
			},

			_ => None
		}
	}

	pub fn split_out(&mut self) -> Option<(*mut StatePtr, *mut StatePtr)> {
		match self {
			State::Split(state) => {
				let ptr1 = &mut state.out1 as *mut Option<Rc<State>>;
				let ptr2 = &mut state.out2 as *mut Option<Rc<State>>;

				Some((ptr1, ptr2))
			},

			_ => None
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

	pub fn split_out1(&mut self) -> Option<*mut StatePtr> {
		match self {
			State::Split(state) => {
				let ptr = &mut state.out1 as *mut Option<Rc<State>>;

				Some(ptr)
			},

			_ => None
		}
	}

	pub fn split_out2(&mut self) -> Option<*mut StatePtr> {
		match self {
			State::Split(state) => {
				let ptr = &mut state.out2 as *mut Option<Rc<State>>;

				Some(ptr)
			},

			_ => None
		}
	}

	fn self_ptr_deep_clone(&self) -> (StatePtr, Vec<*mut StatePtr>) {
		match self {

			State::Basic(basic) => {
				let cloned_regex = basic.c.clone();

				let (cloned_out, cloned_ptr_list) = Self::deep_clone(&basic.out);
				let out_is_some = cloned_out.is_some();

				let mut state = Rc::new(State::Basic(BasicState {
					c: cloned_regex,
					out: cloned_out,
				}));

				let ptr_list = if out_is_some {
					cloned_ptr_list
				} else {
					let ptr = Rc::get_mut(&mut state).unwrap().basic_out().unwrap();

					vec![ptr]
				};

				(Some(state), ptr_list)

			},

			State::Split(split) => {
				let (cloned_out1, cloned_ptr_list1) = Self::deep_clone(&split.out1);
				let cloned_1_is_some = cloned_out1.is_some();

				let (cloned_out2, cloned_ptr_list2) = Self::deep_clone(&split.out2);
				let cloned_2_is_some = cloned_out2.is_some();

				let mut state = Rc::new(State::Split(SplitState {
					out1: cloned_out1,
					out2: cloned_out2,
				}));

				let mut ptr_list1 = if cloned_1_is_some {
					cloned_ptr_list1
				} else {
					let ptr1 = Rc::get_mut(&mut state).unwrap().split_out1().unwrap();

					vec![ptr1]
				};

				let prt_list_2 = if cloned_2_is_some {
					cloned_ptr_list2
				} else {
					let ptr2 = Rc::get_mut(&mut state).unwrap().split_out2().unwrap();

					vec![ptr2]
				};

				ptr_list1.extend(prt_list_2);

				(Some(state), ptr_list1)

			},

			State::Match => {
				let state = State::Match;

				(Some(Rc::new(state)), vec![])
			},

			State::NoMatch => {
				let state = State::NoMatch;

				(Some(Rc::new(state)), vec![])
			},

		}
	}

	pub fn deep_clone(state: &StatePtr) -> (StatePtr, Vec<*mut StatePtr>) {
		if state.is_none() {
			return (None, vec![])
		}

		state.as_ref().unwrap().as_ref().self_ptr_deep_clone()
	}

	pub fn matche_with(&self, c: &char) -> bool {
		match self {

			Self::Basic(basic) => basic.c.match_(&c),

			_ => false
		}
	}
}

impl Fragment {
	pub fn new(start: StatePtr, ptr_list: Vec<*mut StatePtr>) -> Self {
		Self {
			start,
			ptr_list,
		}
	}

	pub fn char(start: StatePtr) -> Self {
		let mut frag = Fragment {
			start,
			ptr_list: vec![]
		};

		let state_ptr = &mut frag.start as *mut Option<Rc<State>>;

		frag.ptr_list.push(state_ptr);

		return frag
	}

	pub fn and(self, e2: Self) -> Self {
		utils::patch(self.ptr_list, &e2.start);

		Fragment::new(self.start, e2.ptr_list)
	}
	
	pub fn or(self, e2: Self) -> Self {
		let s = State::split(self.start, e2.start);

		Fragment::new(s, utils::append(self.ptr_list, e2.ptr_list))
	}

	pub fn optional(self) -> Self {
		let mut s = State::split(self.start, State::null());

		let out2 = Rc::get_mut(s.as_mut().unwrap()).unwrap().split_out().unwrap().1;

		let ptr_list = utils::append(self.ptr_list, utils::list1(out2));

		Fragment::new(s, ptr_list)
	}

	pub fn optional_repeat(self) -> Self {
		let s = State::split(self.start, State::null());

		utils::patch(self.ptr_list, &s);

		let mut_s = unsafe {
			// Get raw pointer to the data
			let ptr = Rc::as_ptr(s.as_ref().unwrap()) as *mut State;
			// Dereference the pointer to get a mutable reference
			&mut *ptr
		};


		let out1 = mut_s.split_out1().unwrap();

		let ptr_list = utils::list1(out1);

		Fragment::new(s, ptr_list)
	}

	pub fn exact_repeat(self, n: &usize) -> Self {
		let mut fragment = self;
		let n = *n;

		if n == 0 {
			utils::patch(fragment.ptr_list, &State::no_match());

			return Fragment::new(fragment.start, vec![])
		}
		
		for _ in 1..n {
			let cloned_fragment = fragment.deep_clone();

			fragment = fragment.and(cloned_fragment);
		}

		fragment
	}

	pub fn at_least(self, n: &usize) -> Self {
		let fragment = self.exact_repeat(n);

		dbg!(&fragment);
		fragment.optional_repeat()
	}

	pub fn range(self, at_least: &usize, at_most: &usize) -> Self {
		let optional_count = at_most - at_least;
	
		if optional_count > 0 {
			let fragment = self.deep_clone().exact_repeat(at_least);

			let mut optional_part = self.deep_clone();
			
			for _ in 1..optional_count {
				let next_optional = self.deep_clone();
				optional_part = optional_part.and(next_optional.optional());
			}
			
			optional_part = optional_part.optional();
			
			fragment.and(optional_part)
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

	pub fn quantify(self, quantifier: &Quantifier) -> Self{
		
		match quantifier {
			// {n}
			Quantifier::Exact(n) => {
				self.exact_repeat(n)
			},

			// {n,}
			Quantifier::AtLeast(n) => {
				self.at_least(n)
			},

			// {n, m}
			Quantifier::Range(at_least, at_most) => {
				self.range(at_least, at_most)
			},
		}
	}
}

// 3. DISPLAY IMPLEMENTATIONS
// =========================

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            State::Basic(basic) => write!(f, "Basic({}, {:?})", basic.c, basic.out.as_ref().map(|_| "...")),
            State::Split(_) => write!(f, "Split({:?}, {:?})", "...", "..."),
            State::NoMatch => write!(f, "NoMatch()"),
            State::Match => write!(f, "Match()"),
        }
    }
}

impl fmt::Display for BasicState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ c: {}, out: {:?} }}", self.c, self.out.as_ref().map(|_| "..."))
    }
}

impl fmt::Display for SplitState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ out1: {:?}, out2: {:?} }}", 
            self.out1.as_ref().map(|_| "..."),
            self.out2.as_ref().map(|_| "...")
        )
    }
}

impl fmt::Display for Fragment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fragment {{ start: {:?}, ptr_list: [{}] }}", 
            self.start.as_ref().map(|_| "..."),
            self.ptr_list.len()
        )
    }
}

// 4. NFA CONSTRUCTION FUNCTIONS
// =============================

pub fn post2nfa(mut postfix: VecDeque<TokenType>) -> ParsingResult<Rc<State>> {
	let mut fragments: Vec<Fragment> = vec![];

	while let Some(token) = postfix.pop_front() {
		match token.into_owned_inner() {
			RegexType::Concatenation => {
				let e2 = fragments.pop()
					.ok_or(ParsingError::unrecognized_rule())?;

				let e1 = fragments.pop()
					.ok_or(ParsingError::unrecognized_rule())?;

				fragments.push(e1.and(e2));
			}

			RegexType::Or => {
				let e2 = fragments.pop()
					.ok_or(ParsingError::unrecognized_rule().because("Unexpected '|'"))?;

				let e1 = fragments.pop()
					.ok_or(ParsingError::unrecognized_rule().because("Unexpected '|'"))?;

				fragments.push(e1.or(e2));
			},

			RegexType::QuestionMark => {
				let e = fragments.pop()
					.ok_or(ParsingError::unrecognized_rule().because("Unexpected '?'"))?;

				fragments.push(e.optional());
			},

			RegexType::Quant(quantifier) => {
				let e = fragments.pop()
					.ok_or(ParsingError::unrecognized_rule().because("Unexpected '?'"))?;

				fragments.push(e.quantify(&quantifier));
			},

			c => {
				let mut s = State::basic(c);

				let frag = Fragment::char(s);

				fragments.push(frag);
			}
		}
	}

	if fragments.len() != 1 {
		return Err(ParsingError::unrecognized_rule())
	}

	let e = fragments.pop().unwrap();
	utils::last_patch(e.ptr_list);

	match e.start {
		Some(state) => Ok(state),
		None => Err(ParsingError::unrecognized_rule())
	}
}

// 4. UTILITY FUNCTIONS
// ====================

pub mod utils {
	use super::*;

	pub fn last_patch(ptr_list: Vec<*mut StatePtr>) {

		let state = Rc::new(State::Match);

		for ptr in ptr_list {
			unsafe { *ptr = Some(Rc::clone(&state)) }
		}

	}

	pub fn patch(ptr_list: Vec<*mut StatePtr>, state: &StatePtr) {

		for ptr in ptr_list {
			let state = state.as_ref().and_then(|rc| Some(Rc::clone(rc)));
			unsafe { *ptr = state }
		}

	}
	
	pub fn list1(endpoint: *mut StatePtr) -> Vec<*mut StatePtr> {
		vec![endpoint]
	}

	pub fn append(mut list1: Vec<*mut StatePtr>, list2: Vec<*mut StatePtr>) -> Vec<*mut StatePtr> {
		list1.extend(list2);

		list1
	}
	
}