pub mod parsing;
pub use parsing::*;

pub mod re2post;
pub use re2post::*;

pub mod post2nfa;
pub use post2nfa::*;

pub mod nfa_simulation;
pub use nfa_simulation::*;

pub mod dfa;
pub use dfa::*;

pub mod dfa_simulation;
pub use dfa_simulation::*;

use std::{collections::{HashSet, VecDeque}, fmt, ops, str::Chars};

use super::*;

// 1. BASIC TYPE DEFINITIONS
// =========================

pub struct Regex {
	// Needed for the conversion to postfix
    operator_stack: Vec<parsing::RegexType>,
    output_stack: Vec<parsing::RegexType>,
}

// 6. REGEX PARSING IMPLEMENTATION
// ==============================

impl Regex {
    pub fn new(expr: String, id: usize) -> ParsingResult<StatePtr> {
        let tokens = Self::tokens(&expr)?;

        let tokens_with_concatenation = Self::add_concatenation(tokens);

        let postfix = re2post(tokens_with_concatenation)?;

		let start = post2nfa(postfix, id)?;

        Ok(start)
    }
}


// Function to print NFA structure iteratively
pub fn print_state_structure(nfa: &StatePtr, title: &str) {
	println!("=== {} ===", title);
	
	let mut stack = Vec::new();
	let mut visited = HashSet::new();
	
	// Start with the root state
	stack.push((nfa.clone(), 0, String::from("root")));
	
	while let Some((state, depth, path)) = stack.pop() {
		let state_ptr = &*state.borrow() as *const State;
		
		// Skip if already visited
		if visited.contains(&state_ptr) && !(State::is_match_ptr(&state) || State::is_nomatch_ptr(&state) || State::is_none_ptr(&state)) {
			continue;
		}
		
		// Mark as visited
		visited.insert(state_ptr);
		
		// Indent based on depth
		let indent = "|  ".repeat(depth);
		
		// Print state information
		match &*state.borrow() {
			State::Basic(basic) => {
				let char_repr = match basic.c.char() {
					Some(c) => format!("'{}'", c),
					None => format!("{:?}", basic.c),
				};
				println!("{}{}Basic: {}", indent, path, char_repr);
				
				// Add out state to stack
				stack.push((basic.out.borrow().clone(), depth + 1, format!("out→")));
			},
			State::Split(split) => {
				println!("{}{}Split", indent, path);
				
				// Add both branches to stack
				stack.push((split.out2.borrow().clone(), depth + 1, format!("out2→")));
				stack.push((split.out1.borrow().clone(), depth + 1, format!("out1→")));
			},
			State::Match { id } => {
				println!("{}{}Match({})", indent, path, id);
			},
			State::StartOfLine { out } => {
				println!("{}{}StartOfLine", indent, path);
				stack.push((out.borrow().clone(), depth + 1, format!("out→")));
			},
			State::EndOfLine { out } => {
				println!("{}{}EndOfLine", indent, path);
				stack.push((out.borrow().clone(), depth + 1, format!("out→")));
			},
			State::NoMatch => {
				println!("{}{}NoMatch", indent, path);
			},
			State::None => {
				println!("{}{}None", indent, path);
			},
		}
	}
	
	println!("=== End of {} ===", title);
}

pub fn print_dfa_structure(dfa: &dfa::Dfa, title: &str) {
    println!("=== {} Structure ===", title);
    println!("Total states: {}", dfa.memory.len());
    
    // Print start state
    println!("Start State (ID: {})", dfa.start.borrow().id);
    
    // Track visited states to avoid cycles
    let mut visited = std::collections::HashSet::new();
    
    // Queue for BFS traversal
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(dfa.start.clone());
    
    while let Some(state_rc) = queue.pop_front() {
        let state = state_rc.borrow();
        
        // Skip if already visited
        if !visited.insert(state.id) {
            continue;
        }
        
        println!("State ID: {}", state.id);
        
        // Print match information
        if state.is_match() {
            println!("  Match State (ID: {})", state.match_id().unwrap());
        }
        
        // Print transitions
        if state.next.is_empty() {
            println!("  No transitions");
        } else {
            println!("  Transitions:");
            for (condition, next_id) in &state.next {
                let condition_str = match condition {
                    dfa::InputCondition::Char(c) => format!("'{}'", c),
                    _ => format!("{:?}", condition),
                };
                println!("    {} → {}", condition_str, next_id);
                
                // Add next state to queue if it exists in memory
                if let Some(next_state) = dfa.memory.get(next_id) {
                    queue.push_back(next_state.clone());
                }
            }
        }
        
        // Print NFA states contained in this DFA state
        println!("  Contains NFA states:");
        for (i, nfa_state) in state.states.iter().enumerate() {
            println!("  NFA State {}:", i);
            print_state_structure(nfa_state, &format!("NFA State in DFA {}", state.id));
        }
        
        println!();
    }
    
    println!("=== End of {} Structure ===", title);
}
