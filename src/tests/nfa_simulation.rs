use crate::parsing::error::ParsingResult;
use crate::regex::{State, BasicState, SplitState, StatePtr, RegexType, List, add_state, input_match};
use std::rc::Rc;
use std::iter::Peekable;
use std::str::Chars;

#[test]
fn test_list_operations() {
    // Test creating an empty list
    let mut list = List::new();
    assert_eq!(list.states.len(), 0);
    
    // Test adding a state to the list
    let state = Rc::new(State::Basic(BasicState {
        c: RegexType::Char('a'),
        out: None,
    }));
    
    list.push(state.clone());
    assert_eq!(list.states.len(), 1);
    
    // Test contains method
    assert!(list.contains(&state));
    
    // Test creating a list from a state
    let list_from_state = List::from(Some(state.clone()));
    assert_eq!(list_from_state.states.len(), 1);
    assert!(list_from_state.contains(&state));
    
    // Test clear method
    list.clear();
    assert_eq!(list.states.len(), 0);
}

#[test]
fn test_list_is_matched() {
    // Test with no match state
    let mut list = List::new();
    let state = Rc::new(State::Basic(BasicState {
        c: RegexType::Char('a'),
        out: None,
    }));
    
    list.push(state);
    assert!(!list.is_matched());
    
    // Test with match state
    list.push(Rc::new(State::Match));
    assert!(list.is_matched());
}

#[test]
fn test_add_state_basic() {
    let mut list = List::new();
    
    // Create a basic state
    let state = Rc::new(State::Basic(BasicState {
        c: RegexType::Char('a'),
        out: None,
    }));
    
    // Add the state to the list
    add_state(Some(&state), &mut list);
    
    // Verify the state was added
    assert_eq!(list.states.len(), 1);
    assert!(list.contains(&state));
    
    // Adding the same state again should have no effect (duplicate prevention)
    add_state(Some(&state), &mut list);
    assert_eq!(list.states.len(), 1);
}

#[test]
fn test_add_state_split() {
    let mut list = List::new();
    
    // Create two basic states for the split state to point to
    let state1 = Rc::new(State::Basic(BasicState {
        c: RegexType::Char('a'),
        out: None,
    }));
    
    let state2 = Rc::new(State::Basic(BasicState {
        c: RegexType::Char('b'),
        out: None,
    }));
    
    // Create a split state that points to both basic states
    let split_state = Rc::new(State::Split(SplitState {
        out1: Some(state1.clone()),
        out2: Some(state2.clone()),
    }));
    
    // Add the split state to the list
    add_state(Some(&split_state), &mut list);
    
    // Verify both states were added (the split state follows epsilon transitions)
    assert_eq!(list.states.len(), 2);
    assert!(list.contains(&state1));
    assert!(list.contains(&state2));
}

#[test]
fn test_input_match_simple() -> ParsingResult<()> {
    // Create an NFA that matches 'a'
    let out = Some(Rc::new(State::Match));
    let state = Rc::new(State::Basic(BasicState {
        c: RegexType::Char('a'),
        out,
    }));
    
    // Test matching
    assert!(input_match(Some(state.clone()), "a"));
    assert!(!input_match(Some(state), "b"));
    
    Ok(())
}

#[test]
fn test_input_match_concatenation() -> ParsingResult<()> {
    // Create an NFA that matches 'ab'
    let end = Some(Rc::new(State::Match));
    let b_state = Rc::new(State::Basic(BasicState {
        c: RegexType::Char('b'),
        out: end,
    }));
    
    let a_state = Rc::new(State::Basic(BasicState {
        c: RegexType::Char('a'),
        out: Some(b_state),
    }));
    
    // Test matching
    assert!(input_match(Some(a_state.clone()), "ab"));
    assert!(!input_match(Some(a_state.clone()), "a"));
    assert!(!input_match(Some(a_state.clone()), "b"));
    assert!(!input_match(Some(a_state), "abc"));
    
    Ok(())
}

#[test]
fn test_input_match_alternation() -> ParsingResult<()> {
    // Create an NFA that matches 'a|b'
    let end = Some(Rc::new(State::Match));
    
    let a_state = Rc::new(State::Basic(BasicState {
        c: RegexType::Char('a'),
        out: end.clone(),
    }));
    
    let b_state = Rc::new(State::Basic(BasicState {
        c: RegexType::Char('b'),
        out: end,
    }));
    
    let start = Rc::new(State::Split(SplitState {
        out1: Some(a_state),
        out2: Some(b_state),
    }));
    
    // Test matching
    assert!(input_match(Some(start.clone()), "a"));
    assert!(input_match(Some(start.clone()), "b"));
    assert!(!input_match(Some(start), "c"));
    
    Ok(())
}

#[test]
fn test_step_function() -> ParsingResult<()> {
    // Create a couple of states
    let end = Some(Rc::new(State::Match));
    
    let a_state = Rc::new(State::Basic(BasicState {
        c: RegexType::Char('a'),
        out: end.clone(),
    }));
    
    let b_state = Rc::new(State::Basic(BasicState {
        c: RegexType::Char('b'),
        out: end,
    }));
    
    // Test stepping through 'a'
    let current_states = List::from(Some(a_state.clone()));
    let mut next_states = List::new();
    
    let mut chars = "a".chars().peekable();
    crate::regex::step(&mut chars, &current_states, &mut next_states);
    
    assert!(next_states.is_matched());
    assert_eq!(next_states.states.len(), 1);
    
    // Test stepping through 'b' (should not match with a_state)
    let current_states = List::from(Some(a_state));
    let mut next_states = List::new();
    
    let mut chars = "b".chars().peekable();
    crate::regex::step(&mut chars, &current_states, &mut next_states);
    
    assert!(!next_states.is_matched());
    assert_eq!(next_states.states.len(), 0);
    
    Ok(())
}

#[test]
fn test_input_match_complex() -> ParsingResult<()> {
    // Create an NFA that matches 'a(b|c)'
    let end = Some(Rc::new(State::Match));
    
    let b_state = Rc::new(State::Basic(BasicState {
        c: RegexType::Char('b'),
        out: end.clone(),
    }));
    
    let c_state = Rc::new(State::Basic(BasicState {
        c: RegexType::Char('c'),
        out: end,
    }));
    
    let split = Rc::new(State::Split(SplitState {
        out1: Some(b_state),
        out2: Some(c_state),
    }));
    
    let a_state = Rc::new(State::Basic(BasicState {
        c: RegexType::Char('a'),
        out: Some(split),
    }));
    
    // Test matching
    assert!(input_match(Some(a_state.clone()), "ab"));
    assert!(input_match(Some(a_state.clone()), "ac"));
    assert!(!input_match(Some(a_state.clone()), "a"));
    assert!(!input_match(Some(a_state.clone()), "b"));
    assert!(!input_match(Some(a_state.clone()), "c"));
    assert!(!input_match(Some(a_state), "abc"));
    
    Ok(())
} 