use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;

use crate::regex::*;
use crate::regex::dfa::*;
use crate::regex::dfa::normalizer::*;
use crate::regex::post2nfa::*;

// Helper function to convert a pattern to postfix notation
fn into_postfix(str: &str) -> VecDeque<TokenType> {
    re2post(Regex::add_concatenation(Regex::tokens(str).unwrap())).unwrap()
}

// Helper function to create a simple DFA for testing
fn create_test_dfa() -> Dfa {
    // Create a simple pattern "a(b|c)" which should give us a DFA with multiple states
    let nfa = post2nfa(into_postfix("a(b|c)"), 0).unwrap();
    Dfa::new(vec![nfa])
}

// Helper function to create a DFA with multiple match states
fn create_test_dfa_with_multiple_matches() -> Dfa {
    // Create the NFAs for different patterns
    let nfa1 = post2nfa(into_postfix("a"), 1).unwrap();
    let nfa2 = post2nfa(into_postfix("b"), 2).unwrap();
    
    // Combine them into a single DFA
    Dfa::new(vec![nfa1, nfa2])
}

#[test]
fn test_normalized_state_creation() {
    // Create a simple normalized state
    let id = 42;
    let mut matchs = HashSet::new();
    matchs.insert(1);
    matchs.insert(2);
    
    let mut next = HashMap::new();
    next.insert(InputCondition::Char('a'), 10);
    next.insert(InputCondition::Char('b'), 20);
    
    let normalized_state = NormalizedState::new(id, matchs.clone(), next.clone());
    
    // Check that all values were stored correctly
    assert_eq!(normalized_state.id, id);
    assert_eq!(normalized_state.matchs, matchs);
    assert_eq!(normalized_state.next, next);
}

#[test]
fn test_normalized_dfa_from_simple_dfa() {
    // Create a simple DFA
    let mut dfa = create_test_dfa();
    
    // Convert to normalized DFA
    let normalized_dfa = NormalizedDfa::from(&mut dfa);
    
    // The normalized DFA should have the same start ID as the original DFA
    assert_eq!(normalized_dfa.start_id, dfa.start.borrow().id);
    
    // The number of states should match
    assert_eq!(normalized_dfa.states.len(), dfa.memory.len());
    
    // The start state should exist in the normalized states
    assert!(normalized_dfa.states.contains_key(&normalized_dfa.start_id));
}

#[test]
fn test_normalized_dfa_with_multiple_matches() {
    // Create a DFA with multiple match states
    let mut dfa = create_test_dfa_with_multiple_matches();
    
    // Convert to normalized DFA
    let normalized_dfa = NormalizedDfa::from(&mut dfa);
    
    // The matchs map should contain entries for each match ID
    assert!(normalized_dfa.matchs.contains_key(&1));
    assert!(normalized_dfa.matchs.contains_key(&2));
    
    // Each of the states in the matchs map should be a match state with the correct ID
    assert_eq!(normalized_dfa.matchs[&1].borrow().match_id(), Some(1));
    assert_eq!(normalized_dfa.matchs[&2].borrow().match_id(), Some(2));
}

#[test]
fn test_normalize_state() {
    // Create a simple DFA
    let dfa = create_test_dfa();
    
    // Get a state to normalize
    let state_ptr = dfa.start.clone();
    let mut match_memory = HashMap::new();
    
    // Normalize the state
    let normalized_state = NormalizedDfa::normalize_state(&state_ptr, &dfa.memory, &mut match_memory);
    
    // Check that the ID matches
    assert_eq!(normalized_state.id, state_ptr.borrow().id);
    
    // Check that the next map has the right number of entries
    assert_eq!(normalized_state.next.len(), state_ptr.borrow().next.len());
    
    // If the original state has matches, they should be in the normalized state
    if state_ptr.borrow().is_match() {
        assert!(!normalized_state.matchs.is_empty());
    } else {
        assert!(normalized_state.matchs.is_empty());
    }
}

#[test]
fn test_normalize_hashmap() {
    // Create a simple DFA
    let dfa = create_test_dfa();
    
    // Get a state with a non-empty next map
    let state_ptr = dfa.start.clone();
    let next_map = &state_ptr.borrow().next;
    
    // Make sure it has transitions
    assert!(!next_map.is_empty());
    
    // Normalize the hashmap
    let normalized_map = NormalizedDfa::normalize_hashmap(next_map, &dfa.memory);
    
    // The normalized map should have the same number of entries
    assert_eq!(normalized_map.len(), next_map.len());
    
    // Each key in the original map should have a corresponding entry in the normalized map
    for key in next_map.keys() {
        assert!(normalized_map.contains_key(key));
    }
}

#[test]
fn test_normalize_statelist() {
    // Create a simple DFA
    let dfa = create_test_dfa();
    
    // Get a StateList that is definitely in the memory
    let first_transition = dfa.start.borrow().next.values().next().unwrap().clone();
    
    // This should be in the memory
    let id_option = NormalizedDfa::normalize_statelist(&first_transition, &dfa.memory);
    assert!(id_option.is_some());
    
    // The ID should match the one in memory
    let state_ptr = dfa.memory.get(&first_transition).unwrap();
    assert_eq!(id_option.unwrap(), state_ptr.borrow().id);
    
    // Now test with a StateList that's definitely not in memory
    let mut unknown_list = StateList::new();
    unknown_list.add_state(&State::match_(999));
    
    let id_option = NormalizedDfa::normalize_statelist(&unknown_list, &dfa.memory);
    assert!(id_option.is_none());
}

#[test]
fn test_transitions_preserved_in_normalized_dfa() {
    // Create a DFA with a pattern that has clear transitions: a -> (b|c)
    let mut dfa = create_test_dfa();
    
    // Convert to normalized DFA
    let normalized_dfa = NormalizedDfa::from(&mut dfa);
    
    // Get the start state
    let start_state = &normalized_dfa.states[&normalized_dfa.start_id];
    
    // Start state should have a transition for 'a'
    assert!(start_state.next.iter().any(|(k, _)| 
        if let InputCondition::Char('a') = k { true } else { false }
    ));
    
    // Follow the 'a' transition
    let a_transition = start_state.next.iter()
        .find(|(k, _)| if let InputCondition::Char('a') = k { true } else { false })
        .map(|(_, &v)| v)
        .unwrap();
    
    // Get the state after 'a'
    let after_a_state = &normalized_dfa.states[&a_transition];
    
    // This state should have transitions for 'b' and 'c'
    assert!(after_a_state.next.iter().any(|(k, _)| 
        if let InputCondition::Char('b') = k { true } else { false }
    ));
    
    assert!(after_a_state.next.iter().any(|(k, _)| 
        if let InputCondition::Char('c') = k { true } else { false }
    ));
}

#[test]
fn test_match_states_in_normalized_dfa() {
    // Create a simple DFA that ends with a match
    let pattern = "abc";
    let nfa = post2nfa(into_postfix(pattern), 42).unwrap();
    let mut dfa = Dfa::new(vec![nfa]);
    
    // Convert to normalized DFA
    let normalized_dfa = NormalizedDfa::from(&mut dfa);
    
    // The matchs map should contain the match ID
    assert!(normalized_dfa.matchs.contains_key(&42));
    
    // Follow the transitions to find the end state
    let start_state = &normalized_dfa.states[&normalized_dfa.start_id];
    
    // Follow 'a'
    let a_id = start_state.next.iter()
        .find(|(k, _)| if let InputCondition::Char('a') = k { true } else { false })
        .map(|(_, &v)| v)
        .unwrap();
    
    let a_state = &normalized_dfa.states[&a_id];
    
    // Follow 'b'
    let b_id = a_state.next.iter()
        .find(|(k, _)| if let InputCondition::Char('b') = k { true } else { false })
        .map(|(_, &v)| v)
        .unwrap();
    
    let b_state = &normalized_dfa.states[&b_id];
    
    // Follow 'c'
    let c_id = b_state.next.iter()
        .find(|(k, _)| if let InputCondition::Char('c') = k { true } else { false })
        .map(|(_, &v)| v)
        .unwrap();
    
    let c_state = &normalized_dfa.states[&c_id];
    
    // This should be a match state with ID 42
    assert!(c_state.matchs.contains(&42));
}

#[test]
fn test_empty_dfa_normalization() {
    // Create an empty DFA
    let mut dfa = Dfa::new(vec![]);
    
    // Convert to normalized DFA
    let normalized_dfa = NormalizedDfa::from(&mut dfa);
    
    // Should have the same number of states
    assert_eq!(normalized_dfa.states.len(), dfa.memory.len());
}

#[test]
fn test_dfa_with_special_transitions() {
    // Create a DFA with StartOfLine and EndOfLine transitions
    let mut dfa = Dfa::new(vec![post2nfa(into_postfix("^a$"), 0).unwrap()]);
    
    // Convert to normalized DFA
    let normalized_dfa = NormalizedDfa::from(&mut dfa);
    
    // Start state should have a StartOfLine transition
    let start_state = &normalized_dfa.states[&normalized_dfa.start_id];
    assert!(start_state.next.iter().any(|(k, _)| 
        matches!(k, InputCondition::StartOfLine)
    ));
    
    // Follow the transitions to find a state with EndOfLine
    let sol_id = start_state.next.iter()
        .find(|(k, _)| matches!(k, InputCondition::StartOfLine))
        .map(|(_, &v)| v)
        .unwrap();
    
    let sol_state = &normalized_dfa.states[&sol_id];
    
    // Follow 'a'
    let a_id = sol_state.next.iter()
        .find(|(k, _)| if let InputCondition::Char('a') = k { true } else { false })
        .map(|(_, &v)| v)
        .unwrap();
    
    let a_state = &normalized_dfa.states[&a_id];
    
    // Should have an EndOfLine transition
    assert!(a_state.next.iter().any(|(k, _)| 
        matches!(k, InputCondition::EndOfLine)
    ));
}

#[test]
fn test_complex_dfa_normalization() {
    // Create a more complex DFA with multiple patterns
    let patterns = ["abc", "def", "a(b|c)*d", "[0-9]+"];
    let mut nfas = Vec::new();
    
    for (i, pattern) in patterns.iter().enumerate() {
        nfas.push(post2nfa(into_postfix(pattern), i).unwrap());
    }
    
    let mut dfa = Dfa::new(nfas);
    
    // Convert to normalized DFA
    let normalized_dfa = NormalizedDfa::from(&mut dfa);
    
    // The normalized DFA should have the same number of states
    assert_eq!(normalized_dfa.states.len(), dfa.memory.len());
    
    // The matchs map should contain all match IDs
    for i in 0..patterns.len() {
        assert!(normalized_dfa.matchs.contains_key(&i));
    }
}
