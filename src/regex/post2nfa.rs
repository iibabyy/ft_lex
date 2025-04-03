use std::{cell::{RefCell, UnsafeCell}, fmt, ops::{Deref, DerefMut}, rc::{Rc, Weak}};

use super::*;
use utils::*;

// 1. BASIC TYPE DEFINITIONS
// =========================

pub type StatePtr = Rc<UnsafeCell<Rc<State>>>;

#[derive(Debug)]
pub enum State {
	Basic(BasicState),
	Split(SplitState),
	NoMatch,
	Match,
	None
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
	pub start: Rc<State>,
	pub ptr_list: Vec<StatePtr>,
}

// 2. TYPE-SPECIFIC METHODS
// ========================

impl State {
	pub fn basic(litteral: RegexType) -> StatePtr {
		let res = Self::Basic(BasicState {
			c: litteral,
			out: State::none(),
		});

		Rc::new(UnsafeCell::new(Rc::new(res)))
	}

	pub fn split(out1: StatePtr, out2: StatePtr) -> StatePtr {
		let res = Self::Split(SplitState {
			out1,
			out2,
		});

		Rc::new(UnsafeCell::new(Rc::new(res)))
	}

	pub fn match_() -> StatePtr {
		let res = Self::Match;

		Rc::new(UnsafeCell::new(Rc::new(res)))
	}

	pub fn no_match() -> StatePtr {
		let res = Self::NoMatch;

		Rc::new(UnsafeCell::new(Rc::new(res)))
	}

	pub fn none() -> StatePtr {
		Rc::new(UnsafeCell::new(Rc::new(State::None)))
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
		matches!(self, State::Match)
	}

	pub fn is_nomatch(&self) -> bool {
		matches!(self, State::NoMatch)
	}

	pub fn is_basic_ptr(ptr: &StatePtr) -> bool {
		unsafe {
			(&*ptr.get()).is_basic()
		}
	}

	pub fn is_split_ptr(ptr: &StatePtr) -> bool {
		unsafe {
			(&*ptr.get()).is_split()
		}
	}

	pub fn is_none_ptr(ptr: &StatePtr) -> bool {
		unsafe {
			(&*ptr.get()).is_none()
		}
	}

	pub fn is_match_ptr(ptr: &StatePtr) -> bool {
		unsafe {
			(&*ptr.get()).is_match()
		}
	}

	pub fn is_nomatch_ptr(ptr: &StatePtr) -> bool {
		unsafe {
			(&*ptr.get()).is_nomatch()
		}
	}

	pub fn from_ptr(ptr: &StatePtr) -> Rc<Self> {
		unsafe {
			Rc::clone(&*ptr.get())
		}
	}

	pub fn basic_out(&self) -> Option<StatePtr> {
		match self {
			State::Basic(state) => {
				Some(Rc::clone(&state.out))
			},

			_ => None
		}
	}

	pub fn split_out(&self) -> Option<(StatePtr, StatePtr)> {
		match self {
			State::Split(state) => {
				let ptr1 = Rc::clone(&state.out1);
				let ptr2 = Rc::clone(&state.out2);

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

	fn self_ptr_deep_clone(&self) -> (StatePtr, Vec<StatePtr>) {
		match self {

			State::Basic(basic) => {
				let cloned_regex = basic.c.clone();

				let (cloned_out, cloned_ptr_list) = Self::deep_clone(&basic.out);
				let cloned_out_is_some = State::is_none_ptr(&cloned_out) == false;

				let state = Rc::new(UnsafeCell::new(Rc::new(State::Basic(BasicState {
					c: cloned_regex,
					out: cloned_out,
				}))));

				let ptr_list = if cloned_out_is_some {
					cloned_ptr_list
				} else {
					let ptr = State::from_ptr(&state).basic_out().unwrap();

					vec![ptr]
				}; 

				(state, ptr_list)
			},

			State::Split(split) => {
				let (cloned_out1, cloned_ptr_list1) = Self::deep_clone(&split.out1);
				let cloned_1_is_some = State::is_none_ptr(&cloned_out1);

				let (cloned_out2, cloned_ptr_list2) = Self::deep_clone(&split.out2);
				let cloned_2_is_some = State::is_none_ptr(&cloned_out2);

				let state = Rc::new(UnsafeCell::new(Rc::new(State::Split(SplitState {
					out1: cloned_out1,
					out2: cloned_out2,
				}))));

				let mut ptr_list1 = if cloned_1_is_some {
					cloned_ptr_list1
				} else {
					let ptr1 = State::from_ptr(&state).split_out().unwrap().0;

					vec![ptr1]
				};

				let prt_list_2 = if cloned_2_is_some {
					cloned_ptr_list2
				} else {
					let ptr2 = State::from_ptr(&state).split_out().unwrap().1;

					vec![ptr2]
				};

				ptr_list1.extend(prt_list_2);

				(state, ptr_list1)
			},

			State::Match => {
				(Rc::new(UnsafeCell::new(Rc::new(State::NoMatch))), vec![])
			},

			State::NoMatch => {
				(Rc::new(UnsafeCell::new(Rc::new(State::NoMatch))), vec![])
			},

			State::None => {
				(Rc::new(UnsafeCell::new(Rc::new(State::None))), vec![])
			}
		}
	}

	pub fn deep_clone(state: &StatePtr) -> (StatePtr, Vec<StatePtr>) {
		if State::is_none_ptr(state) {
			return (State::none(), vec![])
		}

		State::from_ptr(state).self_ptr_deep_clone()
	}

	pub fn matche_with(&self, c: &char) -> bool {
		match self {

			Self::Basic(basic) => basic.c.match_(&c),

			_ => false
		}
	}
}

impl Fragment {
	pub fn new(start: Rc<State>, ptr_list: Vec<StatePtr>) -> Self {
		Self {
			start,
			ptr_list,
		}
	}

	pub fn char(start: Rc<State>) -> Self {
		let ptr = start.basic_out().unwrap();
		let mut frag = Fragment {
			start,
			ptr_list: vec![]
		};

		frag.ptr_list.push(ptr);

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
		let mut s = State::split(self.start, State::none());

		let out2 = State::from_ptr(&s).split_out().unwrap().1;

		let ptr_list = utils::append(self.ptr_list, utils::list1(out2));

		Fragment::new(s, ptr_list)
	}

	pub fn optional_repeat(self) -> Self {
		let s = State::split(self.start, State::none());

		utils::patch(self.ptr_list, &s);

		let state = State::from_ptr(&s);


		let out1 = state.split_out().unwrap().0;

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
            State::Basic(basic) => write!(f, "{basic}"),
            State::Split(_) => write!(f, "Split({:?}, {:?})", "...", "..."),
            State::NoMatch => write!(f, "NoMatch()"),
            State::Match => write!(f, "Match()"),
			State::None => write!(f, "None")
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
        write!(f, "{{ out1: {:?}, out2: {:?} }}", 
			State::is_none_ptr(&self.out1).then_some("...").unwrap_or("None"),
			State::is_none_ptr(&self.out2).then_some("...").unwrap_or("None"),
        )
    }
}

impl fmt::Display for Fragment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fragment {{ start: {:?}, ptr_list: [{}] }}", 
			State::is_none_ptr(&self.start).then_some("...").unwrap_or("None"),
            self.ptr_list.len()
        )
    }
}

// 4. NFA CONSTRUCTION FUNCTIONS
// =============================

pub fn post2nfa(mut postfix: VecDeque<TokenType>) -> ParsingResult<StatePtr> {
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
				let s = State::basic(c);

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

	if State::is_none_ptr(&e.start) {
		return Err(ParsingError::unrecognized_rule())
	}

	Ok(e.start)
}

// 4. UTILITY FUNCTIONS
// ====================

pub mod utils {
	use super::*;

	pub fn last_patch(ptr_list: Vec<StatePtr>) {

		utils::patch(ptr_list, &Rc::new(State::Match));
	}

	pub fn patch(ptr_list: Vec<StatePtr>, state: &Rc<State>) {

		for ptr in ptr_list {
			unsafe { *ptr.get() = Rc::clone(state) }
		}
	}
	
	pub fn list1(endpoint: StatePtr) -> Vec<StatePtr> {
		vec![endpoint]
	}

	pub fn append(mut list1: Vec<StatePtr>, list2: Vec<StatePtr>) -> Vec<StatePtr> {
		list1.extend(list2);

		list1
	}
	
}