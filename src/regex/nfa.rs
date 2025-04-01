use std::{cell::RefCell, ops::{Deref, DerefMut}, rc::{Rc, Weak}};

use super::*;
use utils::*;

type Ptrlist = Vec<Weak<RefCell<State>>>;
type StatePtr = Rc<RefCell<State>>;

// État final correspondant à une correspondance
static MATCH_STATE: StatePtr = State::match_();


pub enum State {
	Basic(BasicState),
	Split(SplitState),
	Match
}

impl State {
	pub fn basic(litteral: RegexType) -> StatePtr {
		let res = Self::Basic(BasicState {
			c: litteral,
			out: None
		});

		Rc::new(RefCell::new(res))
	}

	pub fn split(out1: StatePtr, out2: Option<StatePtr>) -> StatePtr {
		let res = Self::Split(SplitState {
			out1,
			out2
		});

		Rc::new(RefCell::new(res))
	}

	pub fn match_() -> StatePtr {
		let res = Self::Match;

		Rc::new(RefCell::new(res))
	}
}

pub struct BasicState {
	pub c: RegexType,

	pub out: Option<StatePtr>,
}

pub struct SplitState {
	pub out1: StatePtr,
	pub out2: Option<StatePtr>,
}

pub struct Fragment {
	pub start: StatePtr,

	pub ptr_list: Ptrlist
}

impl Fragment {
	pub fn new(start: StatePtr, ptr_list: Ptrlist) -> Self {
		Self {
			start,
			ptr_list
		}
	}
}

pub fn post2nfa(mut postfix: VecDeque<TokenType>) -> ParsingResult<State> {
	let mut fragments: Vec<Fragment> = vec![];

	while let Some(token) = postfix.pop_front() {
		match token.into_owned_inner() {
			RegexType::Concatenation => {
				let e1 = fragments.pop()
					.ok_or(ParsingError::unrecognized_rule())?;

				let e2 = fragments.pop()
					.ok_or(ParsingError::unrecognized_rule())?;

				utils::patch(e1.ptr_list, &e2.start);

				fragments.push(Fragment::new(e1.start, e2.ptr_list));
			}

			RegexType::Or => {
				let e1 = fragments.pop()
					.ok_or(ParsingError::unrecognized_rule().because("Unexpected '|'"))?;

				let e2 = fragments.pop()
				.ok_or(ParsingError::unrecognized_rule().because("Unexpected '|'"))?;

				let s = State::split(e1.start, Some(e2.start));

				fragments.push(Fragment::new(s, utils::append(e1.ptr_list, e2.ptr_list)));
			},

			RegexType::QuestionMark => {
				let e = fragments.pop()
					.ok_or(ParsingError::unrecognized_rule().because("Unexpected '|'"))?;

				let s = State::split(e.start, None);

				let ptr_list = utils::list1(&s);

				fragments.push(Fragment::new(s, ptr_list));
			},

			RegexType::Quant(_) => todo!(),

			c => {
				let s = State::basic(c);

				let ptr_list = utils::list1(&s);

				fragments.push(Fragment::new(s, ptr_list));
			}
		}
	}

	if let Some(e) = fragments.pop() {
		utils::patch(e.ptr_list, );
	} else {
		return Err(ParsingError::unrecognized_rule())
	}

	todo!()
}

pub mod utils {
	use super::*;

	pub fn patch(mut list: Ptrlist, state: &StatePtr) {
		todo!()
	}
	
	pub fn list1(endpoint: &StatePtr) -> Ptrlist {
		vec![Rc::downgrade(endpoint)]
	}

	pub fn append(mut list1: Ptrlist, list2: Ptrlist) -> Ptrlist {
		list1.extend(list2);

		list1
	}
	
}