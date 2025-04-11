// use super::*;

// /// Represents the status of the NFA simulation
// pub enum NfaStatus {
// 	Match(usize),
// 	NoMatch,
// 	Pending
// }

// /// Main simulation controller for NFA-based regex matching
// #[derive(Debug)]
// pub struct NfaSimulation<'a> {
// 	/// If the current input is at the start of a line
// 	start_of_line: bool,

// 	/// The current number of characters readed
// 	readed: usize,

// 	/// The number of characters read until match (if matched)
// 	pub longest_match: Option<usize>,

//     /// NFA to use for matching
//     nfa: &'a Nfa,

//     /// All active validation paths
//     current_states: StateList,

// 	/// Next validation paths that have successfully matched
//     next_states: StateList,
// }

// impl<'a> NfaSimulation<'a> {
// 	pub fn new(nfa: &'a Nfa) -> Self {

// 		let readed = 0;
// 		let longest_match = None;

// 		let current_states = StateList::from(&nfa.start);
// 		let next_states = StateList::new();

// 		NfaSimulation {
// 			start_of_line: false,
// 			readed,
// 			longest_match,
// 			nfa,
// 			current_states,
// 			next_states
// 		}
// 	}

// 	/// Current states are now next states, and next states are cleared
// 	pub fn switch_to_next_states(&mut self) {
// 		std::mem::swap(&mut self.current_states, &mut self.next_states);

// 		self.next_states.clear();
// 	}

// 	/// Check if the start of line matches the NFA's start of line condition
// 	pub fn check_start_of_line(&self) -> bool {
// 		self.nfa.start_of_line == false || self.start_of_line == true
// 	}
// 	/// Check if the end of line matches the NFA's end of line condition
// 	pub fn check_end_of_line(&self, end_of_line: bool) -> bool {
// 		self.nfa.end_of_line == false || end_of_line == true
// 	}

// 	pub fn status(&self) -> NfaStatus {
// 		if self.check_start_of_line() == false {
// 			return NfaStatus::NoMatch
// 		}

// 		if self.current_states.is_empty() == false {
// 			return NfaStatus::Pending
// 		}

// 		if self.longest_match.is_none() {
// 			return NfaStatus::NoMatch
// 		}

// 		NfaStatus::Match(self.longest_match.unwrap())
// 	}

// 	/// Step the simulation forward by one character.
// 	/// 
// 	/// - c :  The current character
// 	/// 
// 	/// - end_of_line :  If the current character is at the end of a line
// 	pub fn step(&mut self, c: &char, end_of_line: bool) -> NfaStatus {

// 		if self.check_start_of_line() == false || self.current_states.is_empty() {
// 			return self.status()
// 		}

// 		self.readed += 1;

// 		for state in &self.current_states {
// 			// The states should be basic states
// 			if State::is_basic_ptr(state) == false {
// 				continue;
// 			}

// 			let borrowed_state = state.borrow();

// 			// Check if the state matches the current character
// 			if borrowed_state.matche_with(c) {
// 				let out = &borrowed_state.basic_out().unwrap();
// 				let next_state = out.borrow();

// 				self.next_states.add_state(&next_state);
// 			}
// 		}

// 		// Check if the next states have a match
// 		if self.next_states.is_matched() {
// 			if self.check_end_of_line(end_of_line) {
// 				self.longest_match = Some(self.readed);
// 			}
// 			self.next_states.remove_matchs();
// 		}

// 		// remove the matchs, to only keep active states in the next states
// 		self.switch_to_next_states();
// 		return self.status()
// 	}

// 	pub fn start(&mut self, start_of_line: bool) {
// 		self.readed = 0;
// 		self.longest_match = None;
// 		self.current_states.clear();
// 		self.current_states.add_state(&self.nfa.start);
// 		self.next_states.clear();
// 		self.start_of_line = start_of_line;
// 	}
// }

// /// Implements a traditional NFA simulation where we track all possible states simultaneously. \
// /// The algorithm maintains two SETS of states (current_states and next_states) and follows all possible
// /// paths through the NFA in parallel. This approach handles nondeterminism by exploring all possible
// /// transitions for each input character, which is the defining characteristic of Thompson's NFA simulation.
// pub fn input_match(nfa: &Nfa, input: &str) -> bool {
//     let mut simulation = NfaSimulation::new(nfa);

// 	let mut chars = input.chars().peekable();

// 	let start_of_line = true;

// 	simulation.start(start_of_line);

// 	// Check if the next states have a match
// 	if simulation.current_states.is_matched() {
// 		return simulation.nfa.end_of_line == false || input.is_empty();
// 	}	

// 	while let Some(c) = chars.next() {
// 		let peek = chars.peek();
// 		// check if the next character is the end of a line
// 		let end_of_line = peek == None || peek == Some(&'\n');

// 		match simulation.step(&c, end_of_line) {
// 			NfaStatus::Pending => continue,

// 			// finished
// 			_ => break,
// 		}
// 	}

// 	simulation.longest_match.is_some()
// }
