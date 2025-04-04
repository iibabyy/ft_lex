use crate::regex::*;
use crate::regex::post2nfa::*;
use crate::regex::nfa_simulation::*;
use std::collections::VecDeque;
use std::iter::Peekable;
use std::str::Chars;
use std::rc::Rc;

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a basic NFA
    fn create_basic_nfa(c: char) -> StatePtr {
        State::basic(RegexType::Char(c))
    }

    // Helper function to create simple patterns
    fn create_simple_pattern(pattern: &str) -> StatePtr {
        // This is a simplified version just for testing
        let mut states = Vec::new();
        
        // Create a state for each character
        for c in pattern.chars() {
            states.push(create_basic_nfa(c));
        }
        
        // Connect the states in sequence
        for i in 0..states.len() - 1 {
            let out = states[i].borrow().basic_out().unwrap();
            *out.borrow_mut() = Rc::clone(&states[i + 1]);
        }
        
        // Add match state at the end
        if let Some(last) = states.last() {
            let out = last.borrow().basic_out().unwrap();
            *out.borrow_mut() = State::match_();
        }
        
        // Return the start state
        if let Some(first) = states.first() {
            Rc::clone(first)
        } else {
            // Return match state for empty pattern
            State::match_()
        }
    }

    #[test]
    fn test_list_creation() {
        // Create a new empty list
        let list = List::new();
        assert_eq!(list.states.len(), 0);
        
        // Create a list from a state
        let state = State::match_();
        let list = List::from(&state);
        assert_eq!(list.states.len(), 1);
        assert!(list.is_matched());
    }

    #[test]
    fn test_list_operations() {
        let mut list = List::new();
        let state1 = State::match_();
        let state2 = State::basic(RegexType::Char('a'));
        
        // Test push
        list.push(&state1);
        assert_eq!(list.states.len(), 1);
        
        // Test contains
        assert!(list.contains(&state1));
        assert!(!list.contains(&state2));
        
        // Test clear
        list.clear();
        assert_eq!(list.states.len(), 0);
    }

    #[test]
    fn test_add_state() {
        let mut list = List::new();
        
        // Test adding a basic state
        let basic = State::basic(RegexType::Char('a'));
        add_state(&basic, &mut list);
        assert_eq!(list.states.len(), 1);
        
        // Test adding a split state (should add both branches)
        let s1 = State::basic(RegexType::Char('b'));
        let s2 = State::basic(RegexType::Char('c'));
        let split = State::split(s1, s2);
        
        list.clear();
        add_state(&split, &mut list);
        assert_eq!(list.states.len(), 2);
        
        // Test adding a state already in the list (should not duplicate)
        let s3 = list.states[0].clone();
        add_state(&s3, &mut list);
        assert_eq!(list.states.len(), 2);
    }

    #[test]
    fn test_step_function() {
        // Create a simple NFA for 'a'
        let nfa = create_basic_nfa('a');
        
        let mut current_list = List::from(&nfa);
        let mut next_list = List::new();
        
        // Test step with matching character
        let mut chars = "a".chars().peekable();
        step(&mut chars, &current_list, &mut next_list);
        
        // The next list should contain the out state
        assert!(!next_list.states.is_empty());
    }

    #[test]
    fn test_input_match_simple() {
        // Test matching a single character
        let nfa = create_basic_nfa('a');
        assert!(input_match(&nfa, "a"));
        assert!(!input_match(&nfa, "b"));
        assert!(!input_match(&nfa, ""));
        
        // Test matching a sequence
        let nfa = create_simple_pattern("abc");
        assert!(input_match(&nfa, "abc"));
        assert!(!input_match(&nfa, "ab"));
        assert!(!input_match(&nfa, "abx"));
        
        // Test match state
        let match_state = State::match_();
        assert!(input_match(&match_state, ""));
        assert!(input_match(&match_state, "anything"));
    }

    #[test]
    fn test_is_matched() {
        // Create a list with no match state
        let mut list = List::new();
        let state = State::basic(RegexType::Char('a'));
        list.push(&state);
        assert!(!list.is_matched());
        
        // Add a match state
        let match_state = State::match_();
        list.push(&match_state);
        assert!(list.is_matched());
    }
}
