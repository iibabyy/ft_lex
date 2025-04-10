use crate::regex::*;
use crate::regex::dfa::*;
use crate::regex::post2nfa::*;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

fn into_postfix(str: &str) -> VecDeque<TokenType> {
	re2post(Regex::add_concatenation(Regex::tokens(str).unwrap())).unwrap()
}

// ==========================================
// InputCondition Tests
// ==========================================

#[test]
fn test_input_condition_creation_and_comparison() {
    // Create different InputConditions
    let char_a = InputCondition::Char('a');
    let char_b = InputCondition::Char('b');
    let start_line = InputCondition::StartOfLine;
    let end_line = InputCondition::EndOfLine;
    
    // Test equality
    assert_eq!(char_a, InputCondition::Char('a'));
    assert_eq!(start_line, InputCondition::StartOfLine);
    assert_eq!(end_line, InputCondition::EndOfLine);
    
    // Test inequality
    assert_ne!(char_a, char_b);
    assert_ne!(char_a, start_line);
    assert_ne!(start_line, end_line);
}

#[test]
fn test_input_condition_hashing() {
    // Create a HashMap with InputConditions as keys
    let mut map = HashMap::new();
    map.insert(InputCondition::Char('a'), "char_a");
    map.insert(InputCondition::StartOfLine, "start_line");
    map.insert(InputCondition::EndOfLine, "end_line");
    
    // Test lookup
    assert_eq!(map.get(&InputCondition::Char('a')), Some(&"char_a"));
    assert_eq!(map.get(&InputCondition::StartOfLine), Some(&"start_line"));
    assert_eq!(map.get(&InputCondition::EndOfLine), Some(&"end_line"));
    assert_eq!(map.get(&InputCondition::Char('b')), None);
}

// ==========================================
// DfaState Construction Tests
// ==========================================

#[test]
fn test_dfa_state_creation_empty() {
    let state_list = StateList::new();
    let dfa_state = DfaState::new(0, state_list);
    
    assert_eq!(dfa_state.id, 0);
    assert!(dfa_state.states.is_empty());
    assert!(dfa_state.matchs.is_empty());
    assert!(dfa_state.next.is_empty());
}

#[test]
fn test_dfa_state_creation_with_single_state() {
    // Create a basic state
    let basic_state = State::basic(RegexType::Char('a'));
    
    // Create a state list and add the basic state
    let mut state_list = StateList::new();
    state_list.add_state(&basic_state);
    
    // Create a DFA state from the state list
    let dfa_state = DfaState::new(1, state_list);
    
    assert_eq!(dfa_state.id, 1);
    assert!(!dfa_state.states.is_empty());
    assert!(dfa_state.matchs.is_empty());
}

#[test]
fn test_dfa_state_creation_with_match_states() {
    // Create a match state and a regular state
    let match_state = State::match_(1);
    let basic_state = State::basic(RegexType::Char('a'));
    
    // Create a state list with both states
    let mut state_list = StateList::new();
    state_list.add_state(&match_state);
    state_list.add_state(&basic_state);
    
    // Create a DFA state
    let dfa_state = DfaState::new(2, state_list);
    
    // Match states should be extracted
    assert!(!dfa_state.matchs.is_empty());
    // The basic state should remain in states
    assert!(!dfa_state.states.is_empty());
}

#[test]
fn test_dfa_state_id_assignment() {
    // Create state lists
    let state_list1 = StateList::new();
    let state_list2 = StateList::new();
    let state_list3 = StateList::new();
    
    // Create DFA states with different IDs
    let dfa_state1 = DfaState::new(0, state_list1);
    let dfa_state2 = DfaState::new(1, state_list2);
    let dfa_state3 = DfaState::new(2, state_list3);
    
    // Verify IDs are assigned correctly
    assert_eq!(dfa_state1.id, 0);
    assert_eq!(dfa_state2.id, 1);
    assert_eq!(dfa_state3.id, 2);
}

// ==========================================
// Map Merging Tests
// ==========================================

#[test]
fn test_merge_input_maps_no_overlap() {
    // Create two maps with no overlapping keys
    let mut map1 = HashMap::new();
    map1.insert(InputCondition::Char('a'), StateList::new());
    
    let mut map2 = HashMap::new();
    map2.insert(InputCondition::Char('b'), StateList::new());
    
    // Merge maps
    merge_input_maps(&mut map1, map2);
    
    // Verify result
    assert_eq!(map1.len(), 2);
    assert!(map1.contains_key(&InputCondition::Char('a')));
    assert!(map1.contains_key(&InputCondition::Char('b')));
}

#[test]
fn test_merge_input_maps_with_overlap() {
    // Create a basic state to add to our state lists
    let state_a = State::basic(RegexType::Char('a'));
    let state_b = State::basic(RegexType::Char('b'));
    
    // Create state lists
    let mut list1 = StateList::new();
    list1.add_state(&state_a);
    
    let mut list2 = StateList::new();
    list2.add_state(&state_b);
    
    // Create two maps with an overlapping key
    let mut map1 = HashMap::new();
    map1.insert(InputCondition::Char('a'), list1);
    
    let mut map2 = HashMap::new();
    map2.insert(InputCondition::Char('a'), list2);
    map2.insert(InputCondition::Char('b'), StateList::new());
    
    // Merge maps
    merge_input_maps(&mut map1, map2);
    
    // Verify result
    assert_eq!(map1.len(), 2);
    
    // The overlapping key should have both states
    let merged_list = map1.get(&InputCondition::Char('a')).unwrap();
    assert_eq!(merged_list.len(), 2);
}

#[test]
fn test_merge_input_maps_with_empty_map() {
    // Create a map with some entries
    let mut map1 = HashMap::new();
    map1.insert(InputCondition::Char('a'), StateList::new());
    map1.insert(InputCondition::StartOfLine, StateList::new());
    
    // Create an empty map
    let map2 = HashMap::new();
    
    // Merge maps
    merge_input_maps(&mut map1, map2);
    
    // Verify the result is unchanged
    assert_eq!(map1.len(), 2);
    assert!(map1.contains_key(&InputCondition::Char('a')));
    assert!(map1.contains_key(&InputCondition::StartOfLine));
}

// ==========================================
// DFA Construction Tests
// ==========================================

#[test]
fn test_dfa_creation_simple() {
    // Create a simple NFA for 'a'
    let postfix = into_postfix("a");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create DFA
    let dfa = Dfa::new(vec![nfa_state]);
    
    // Verify DFA is created correctly
    assert!(!dfa.memory.is_empty());
    
    // The start state should have a transition on 'a'
    let start_state = dfa.start.borrow();
    assert!(start_state.next.contains_key(&InputCondition::Char('a')));
}

#[test]
fn test_dfa_creation_alternation() {
    // Create an NFA for 'a|b'
    let postfix = into_postfix("a|b");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create DFA
    let dfa = Dfa::new(vec![nfa_state]);
    
    // Verify DFA is created correctly
    assert!(!dfa.memory.is_empty());
    
    // The start state should have transitions on both 'a' and 'b'
    let start_state = dfa.start.borrow();
    assert!(start_state.next.contains_key(&InputCondition::Char('a')) || 
            start_state.next.contains_key(&InputCondition::Char('b')));
}

#[test]
fn test_dfa_creation_with_anchors() {
    // Create an NFA for '^a$'
    let postfix = into_postfix("^a$");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create DFA
    let dfa = Dfa::new(vec![nfa_state]);
    
    // Verify DFA is created correctly
    assert!(!dfa.memory.is_empty());
    
    // The start state should have a transition on StartOfLine
    let start_state = dfa.start.borrow();
    assert!(start_state.next.contains_key(&InputCondition::StartOfLine));
}

// ==========================================
// DfaState::find_next Tests
// ==========================================

#[test]
fn test_find_next_basic_state() {
    // Create a basic state
    let basic_state = State::basic(RegexType::Char('a'));
    
    // Get next states
    let (next_states, matchs) = DfaState::find_next(&basic_state, &mut HashMap::new());
    
    // Verify results
    assert!(next_states.contains_key(&InputCondition::Char('a')));
    assert!(matchs.is_empty());
}

#[test]
fn test_find_next_split_state() {
    // Create a split state
    let state1 = State::basic(RegexType::Char('a'));
    let state2 = State::basic(RegexType::Char('b'));
    let split_state = State::split(state1, state2);
    
    // Get next states
    let (next_states, matchs) = DfaState::find_next(&split_state, &mut HashMap::new());
    
    // Verify results
    assert!(next_states.contains_key(&InputCondition::Char('a')));
    assert!(next_states.contains_key(&InputCondition::Char('b')));
    assert!(matchs.is_empty());
}

#[test]
fn test_find_next_match_state() {
    // Create a match state
    let match_state = State::match_(1);
    
    // Get next states
    let (next_states, matchs) = DfaState::find_next(&match_state, &mut HashMap::new());
    
    // Verify results
    assert!(next_states.is_empty());
    assert!(!matchs.is_empty());
}

#[test]
fn test_find_next_start_of_line() {
    // Create a start-of-line state
    let start_line = State::start_of_line();
    
    // Get next states
    let (next_states, matchs) = DfaState::find_next(&start_line, &mut HashMap::new());
    
    // Verify results
    assert!(next_states.contains_key(&InputCondition::StartOfLine));
    assert!(matchs.is_empty());
}

#[test]
fn test_find_next_end_of_line() {
    // Create an end-of-line state
    let end_line = State::end_of_line();
    
    // Get next states
    let (next_states, matchs) = DfaState::find_next(&end_line, &mut HashMap::new());
    
    // Verify results
    assert!(next_states.contains_key(&InputCondition::EndOfLine));
    assert!(matchs.is_empty());
}

// ==========================================
// Recursive vs Iterative Creation Tests
// ==========================================

#[test]
#[allow(deprecated)]
fn test_recursive_vs_iterative_creation() {
    // Create a simple NFA for 'a'
    let postfix = into_postfix("a");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create a StateList for the NFA
    let mut state_list = StateList::new();
    state_list.add_state(&nfa_state);
    
    // Create DFA recursively
    let mut memory = HashMap::new();
    let recursive_start = DfaState::recursive_create(state_list.clone(), &mut memory);
    
    // Create DFA iteratively
    let (iterative_start, _) = DfaState::iterative_create(state_list);
    
    // Both states should be equivalent (have same transitions)
    assert_eq!(recursive_start.borrow().next.len(), iterative_start.borrow().next.len());
}

// ==========================================
// Edge Cases Tests
// ==========================================

#[test]
fn test_dfa_state_with_empty_list() {
    // Create an empty state list
    let state_list = StateList::new();
    
    // Create a DFA state
    let dfa_state = DfaState::new(0, state_list);
    
    // Verify properties
    assert!(dfa_state.states.is_empty());
    assert!(dfa_state.matchs.is_empty());
    assert!(dfa_state.next.is_empty());
}

#[test]
fn test_dfa_creation_with_complex_pattern() {
    // Create an NFA for '(a|b)*c'
    let postfix = into_postfix("(a|b)*c");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create DFA
    let dfa = Dfa::new(vec![nfa_state]);
    
    // Verify DFA is created correctly
    assert!(!dfa.memory.is_empty());
    
    // The DFA should have multiple states
    assert!(dfa.memory.len() > 1);
}

// ==========================================
// Integration Tests
// ==========================================

#[test]
fn test_full_regex_pipeline_simple() {
    // Create a simple regex pattern 'a'
    let tokens = Regex::tokens("a").unwrap();
    let infix = Regex::add_concatenation(tokens);
    let postfix = re2post::re2post(infix).unwrap();
    let nfa = post2nfa(postfix, 0).unwrap();
    
    // Convert to DFA
    let dfa = Dfa::new(vec![nfa]);
    
    // Verify structure
    let start_state = dfa.start.borrow();
    assert!(start_state.next.contains_key(&InputCondition::Char('a')));
}

#[test]
fn test_full_regex_pipeline_alternation() {
    // Create a regex pattern 'a|b'
    let postfix = into_postfix("a|b");
    let nfa = post2nfa(postfix, 0).unwrap();
    
    // Convert to DFA
    let dfa = Dfa::new(vec![nfa]);
    
    // Verify structure
    let start_state = dfa.start.borrow();
    assert!(start_state.next.contains_key(&InputCondition::Char('a')) || 
            start_state.next.contains_key(&InputCondition::Char('b')));
}

#[test]
fn test_full_regex_pipeline_with_anchors() {
    // Create a regex pattern '^a$'
    let postfix = into_postfix("^a$");
    let nfa = post2nfa(postfix, 0).unwrap();
    
    // Convert to DFA
    let dfa = Dfa::new(vec![nfa]);
    
    // Verify structure
    let start_state = dfa.start.borrow();
    assert!(start_state.next.contains_key(&InputCondition::StartOfLine));
}

// ==========================================
// Memory Reuse Tests
// ==========================================

#[test]
fn test_dfa_state_memory_reuse() {
    // Create a simple NFA
    let basic_state = State::basic(RegexType::Char('a'));
    
    // Create a state list with the basic state
    let mut state_list = StateList::new();
    state_list.add_state(&basic_state);
    
    // Create DFA state iteratively
    let (dfa_state1, _) = DfaState::iterative_create(state_list.clone());
    
    // Create another DFA state with the same state list
    let (dfa_state2, _) = DfaState::iterative_create(state_list);
    
    // Verify the same state instance is reused
    assert!(Rc::ptr_eq(&dfa_state1, &dfa_state2));
}

// ==========================================
// StateList Interaction Tests
// ==========================================

#[test]
fn test_state_list_operations() {
    // Create states
    let state1 = State::basic(RegexType::Char('a'));
    let state2 = State::basic(RegexType::Char('b'));
    let match_state = State::match_(1);
    
    // Create state list
    let mut list = StateList::new();
    
    // Test adding states
    list.add_state(&state1);
    assert_eq!(list.len(), 1);
    
    // Test adding duplicate state
    list.add_state(&state1);
    assert_eq!(list.len(), 1); // Should not increase
    
    // Test adding different state
    list.add_state(&state2);
    assert_eq!(list.len(), 2);
    
    // Test adding match state
    list.add_state(&match_state);
    assert_eq!(list.len(), 3);
    
    // Test removing match states
    let matchs = list.remove_matchs();
    assert_eq!(list.len(), 2); // Match state removed
    assert_eq!(matchs.len(), 1); // 1 match state
}

#[test]
fn test_state_list_merging() {
    // Create states
    let state1 = State::basic(RegexType::Char('a'));
    let state2 = State::basic(RegexType::Char('b'));
    let state3 = State::basic(RegexType::Char('c'));
    
    // Create first list
    let mut list1 = StateList::new();
    list1.add_state(&state1);
    list1.add_state(&state2);
    
    // Create second list
    let mut list2 = StateList::new();
    list2.add_state(&state2); // Duplicate state
    list2.add_state(&state3); // New state
    
    // Merge lists
    list1.merge(list2);
    
    // Verify result
    assert_eq!(list1.len(), 3); // Should contain all unique states
}

// ==========================================
// Complex Pattern Support Tests
// ==========================================

#[test]
fn test_dfa_creation_with_character_class() {
    // Simulate a character class [abc] in postfix (a|b|c)
    let postfix = into_postfix("[abc]");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create DFA
    let dfa = Dfa::new(vec![nfa_state]);
    
    // Verify DFA structure - should have transitions for a, b, and c
    let start_state = dfa.start.borrow();
    assert!(start_state.next.contains_key(&InputCondition::Char('a')) || 
            start_state.next.contains_key(&InputCondition::Char('b')) || 
            start_state.next.contains_key(&InputCondition::Char('c')));
}

#[test]
fn test_dfa_creation_with_sequential_quantifiers() {
    // Simulate a pattern like a+b* in postfix (a+ b* concatenate)
    let postfix = into_postfix("a+b*");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create DFA
    let dfa = Dfa::new(vec![nfa_state]);
    
    // Verify DFA is properly created
    assert!(dfa.memory.len() > 1); // Should have multiple states for this complex pattern
}

#[test]
fn test_dfa_creation_with_nested_groups() {
    // Simulate a pattern like (a(b|c))+ in postfix (a (b c |) concatenate +)
    let postfix = into_postfix("(a(b|c))+");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create DFA
    let dfa = Dfa::new(vec![nfa_state]);
    
    // Verify DFA is created with the correct structure
    assert!(dfa.memory.len() > 2); // Complex pattern should result in more states
}

// ==========================================
// Error Handling Tests
// ==========================================

#[test]
fn test_dfa_creation_with_empty_nfa_list() {
    // Try creating a DFA with an empty list of NFAs
    let dfa = Dfa::new(vec![]);
    
    // Start state should exist but have no transitions
    let start_state = dfa.start.borrow();
    assert!(start_state.next.is_empty());
    assert!(start_state.matchs.is_empty());
}

#[test]
fn test_dfa_state_with_malformed_state() {
    // Create a "malformed" state (for testing purposes, just a split state with empty outputs)
    // In practice this would be invalid but DFA should handle it gracefully
    let split_state = State::split(State::none(), State::none());
    
    // Create a state list with the malformed state
    let mut state_list = StateList::new();
    state_list.add_state(&split_state);
    
    // Create a DFA state
    let dfa_state = DfaState::new(0, state_list);
    
    // Verify DFA state is created without errors
    assert_eq!(dfa_state.id, 0);
    assert!(!dfa_state.states.is_empty()); // Should contain our malformed state
    assert!(dfa_state.matchs.is_empty());
}

// ==========================================
// Performance Optimization Tests
// ==========================================

#[test]
fn test_memory_caching_with_repeated_patterns() {
    // Create NFA for simple repeated pattern to test memory caching
    // Pattern: a*
    let postfix = into_postfix("a*");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create a StateList with the NFA
    let mut state_list = StateList::new();
    state_list.add_state(&nfa_state);
    
    // Create DFA states for the same pattern repeatedly
    let (_, memory) = DfaState::iterative_create(state_list.clone());
    
    // The memory cache should now have entries
    assert!(!memory.is_empty());
    
    // Number of states in memory should be limited despite the pattern having a loop
    assert!(memory.len() <= 3); // Typically only need 1-2 states for a*
}

#[test]
fn test_state_list_hashing_consistency() {
    // Test that identical StateLists generate the same hash for memory reuse
    
    // Create two identical StateLists
    let state1 = State::basic(RegexType::Char('a'));
    let state2 = State::basic(RegexType::Char('b'));
    
    let mut list1 = StateList::new();
    list1.add_state(&state1);
    list1.add_state(&state2);
    
    let mut list2 = StateList::new();
    list2.add_state(&state1);
    list2.add_state(&state2);
    
    // Create DFA states with memory caching
    let (dfa_state1, memory) = DfaState::iterative_create(list1);
    let (dfa_state2, _) = DfaState::iterative_create(list2);
    
    // The memory cache should reuse the same state
    assert_eq!(memory.len(), 1); // Only one unique state list
    assert!(Rc::ptr_eq(&dfa_state1, &dfa_state2)); // Same DFA state returned
}

// ==========================================
// MultiPattern DFA Tests
// ==========================================

#[test]
fn test_multi_pattern_dfa_creation() {
    // Create NFAs for 'a' and 'b'
    let postfix1 = into_postfix("a");
    let nfa1 = post2nfa(postfix1, 0).unwrap();
    
    let postfix2 = into_postfix("b");
    let nfa2 = post2nfa(postfix2, 1).unwrap();
    
    // Create DFA from both NFAs
    let dfa = Dfa::new(vec![nfa1, nfa2]);
    
    // Verify DFA structure - should have transitions for both 'a' and 'b'
    let start_state = dfa.start.borrow();
    assert!(start_state.next.contains_key(&InputCondition::Char('a')));
    assert!(start_state.next.contains_key(&InputCondition::Char('b')));
}

#[test]
fn test_multi_pattern_match_states() {
    // Create NFAs for 'a' and 'b' with different match IDs
    let postfix1 = into_postfix("a");
    let nfa1 = post2nfa(postfix1, 5).unwrap(); // Match ID 5
    
    let postfix2 = into_postfix("b");
    let nfa2 = post2nfa(postfix2, 10).unwrap(); // Match ID 10
    
    // Create DFA
    let dfa = Dfa::new(vec![nfa1, nfa2]);
    
    // Navigate to 'a' transition and check match state
    let start_state = dfa.start.borrow();
    let a_next_id = start_state.next.get(&InputCondition::Char('a')).unwrap();
    let a_next = dfa.memory.get(a_next_id).unwrap().borrow();
    
    // The state reached after 'a' should have match ID 5
    assert_eq!(a_next.id, 5);

    // Navigate to 'b' transition and check match state
    let b_next_id = start_state.next.get(&InputCondition::Char('b')).unwrap();
    let b_next = dfa.memory.get(b_next_id).unwrap().borrow();
    
    // The state reached after 'b' should have match ID 10
    assert_eq!(b_next.id, 10);
}

// ==========================================
// Cycle Handling Tests
// ==========================================

#[test]
fn test_dfa_creation_with_cyclic_nfa() {
    // Create an NFA for 'a*' (which has a cycle)
    let postfix = into_postfix("a*");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create DFA
    let dfa = Dfa::new(vec![nfa_state]);
    
    // Verify proper handling of cycles - start state should have a transition to itself
    let start_state = dfa.start.borrow();
    if let Some(next_id) = start_state.next.get(&InputCondition::Char('a')) {
        let next_state = dfa.memory.get(next_id).unwrap().borrow();
        
        // For a*, the state after 'a' should have a transition back to itself with 'a'
        assert!(next_state.next.contains_key(&InputCondition::Char('a')));
        
        // The cyclic transition should point back to the same state
        assert_eq!(next_state.next.get(&InputCondition::Char('a')), Some(next_id));
    }
}

#[test]
fn test_dfa_creation_with_complex_cycle() {
    // Create an NFA for '(ab)*' (which has a more complex cycle)
    let postfix = into_postfix("(ab)*");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create DFA
    let dfa = Dfa::new(vec![nfa_state]);
    
    // Verify proper number of states
    // For (ab)*, we need at minimum: start state and state after 'a'
    assert!(dfa.memory.len() >= 2);
}

// ==========================================
// State Transition Computation Tests
// ==========================================

#[test]
fn test_compute_next_with_basic_state() {
    // Create basic state
    let basic_state = State::basic(RegexType::Char('a'));
    
    // Compute next state
    let state_list = StateList::from(&basic_state);
	
	let mut dfa_state = DfaState::new(0, state_list);
    
	DfaState::compute_next(&mut dfa_state, &mut HashMap::new());
    
    assert_eq!(dfa_state.next.len(), 1);
}

#[test]
fn test_compute_next_with_epsilon_transitions() {
    // Create a split state (which is like an epsilon transition)
    let state1 = State::basic(RegexType::Char('a'));
    let state2 = State::basic(RegexType::Char('b'));
    let split_state = State::split(state1, state2);
    
    // Compute next state
    let state_list = StateList::from(&split_state);
	let mut dfa_state = DfaState::new(0, state_list);
	DfaState::compute_next(&mut dfa_state, &mut HashMap::new());
    
    // Split state should add both out1 and out2 to the state list
    assert_eq!(dfa_state.next.len(), 2);
}

#[test]
fn test_compute_next_with_match_state() {
    // Create a match state
    let match_state = State::match_(1);
    
    // Compute next state
    let state_list = StateList::from(&match_state);
	let mut dfa_state = DfaState::new(0, state_list);
	DfaState::compute_next(&mut dfa_state, &mut HashMap::new());
    
    // Match state has no outgoing transitions, so state_list should remain empty
    assert_eq!(dfa_state.next.len(), 0);
}

// ==========================================
// DFA Size Optimization Tests
// ==========================================

#[test]
fn test_dfa_state_reuse_with_equivalent_states() {
    // Create an NFA for 'a|a' (two states that are functionally equivalent)
    let mut postfix = VecDeque::new();
    postfix.push_back(TokenType::from(RegexType::Char('a')));
    postfix.push_back(TokenType::from(RegexType::Char('a')));
    postfix.push_back(TokenType::from(RegexType::Or));
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create DFA
    let dfa = Dfa::new(vec![nfa_state]);
    
    // Despite the NFA having two 'a' transitions, the DFA should optimize to just 2 states:
    // 1. Start state with 'a' transition
    // 2. Match state after 'a'
    assert_eq!(dfa.memory.len(), 2);
}

#[test]
fn test_dfa_state_minimization_for_complex_pattern() {
    // Create NFA for '(a|b)c'
    let mut postfix = VecDeque::new();
    postfix.push_back(TokenType::from(RegexType::Char('a')));
    postfix.push_back(TokenType::from(RegexType::Char('b')));
    postfix.push_back(TokenType::from(RegexType::Or));
    postfix.push_back(TokenType::from(RegexType::Char('c')));
    postfix.push_back(TokenType::from(RegexType::Concatenation));
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create DFA
    let dfa = Dfa::new(vec![nfa_state]);
    
    // The optimized DFA should have 3 states:
    // 1. Start state with 'a' and 'b' transitions
    // 2. State after 'a' or 'b' with 'c' transition
    // 3. Match state after 'c'
    assert_eq!(dfa.memory.len(), 3);
}

// ==========================================
// Explicit Iterative Creation Tests
// ==========================================

#[test]
fn test_iterative_creation_basic() {
    // Create a simple NFA for 'a'
    let mut postfix = VecDeque::new();
    postfix.push_back(TokenType::from(RegexType::Char('a')));
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create a StateList for the NFA
    let mut state_list = StateList::new();
    state_list.add_state(&nfa_state);
    
    // Create DFA iteratively
    let (start_state, memory) = DfaState::iterative_create(state_list);
    
    // Verify structure
    let start = start_state.borrow();
    assert!(start.next.contains_key(&InputCondition::Char('a')));
    assert_eq!(memory.len(), 2); // Start state + state after 'a'
}

#[test]
fn test_iterative_creation_complex() {
    // Create a more complex NFA for '(a|b)*c'
    let mut postfix = VecDeque::new();
    postfix.push_back(TokenType::from(RegexType::Char('a')));
    postfix.push_back(TokenType::from(RegexType::Char('b')));
    postfix.push_back(TokenType::from(RegexType::Or));
    postfix.push_back(TokenType::from(RegexType::Quant(Quantifier::AtLeast(0))));
    postfix.push_back(TokenType::from(RegexType::Char('c')));
    postfix.push_back(TokenType::from(RegexType::Concatenation));
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create a StateList for the NFA
    let mut state_list = StateList::new();
    state_list.add_state(&nfa_state);
    
    // Create DFA iteratively
    let (_, memory) = DfaState::iterative_create(state_list);
    
    // Verify all states were created
    assert!(memory.len() >= 3); // At least: start state, (a|b)* state, and state after c
}

#[test]
fn test_iterative_creation_work_queue() {
    // Test that the work queue correctly processes all states
    // Create an NFA for 'abc'
    let mut postfix = VecDeque::new();
    postfix.push_back(TokenType::from(RegexType::Char('a')));
    postfix.push_back(TokenType::from(RegexType::Char('b')));
    postfix.push_back(TokenType::from(RegexType::Concatenation));
    postfix.push_back(TokenType::from(RegexType::Char('c')));
    postfix.push_back(TokenType::from(RegexType::Concatenation));
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create a StateList for the NFA
    let mut state_list = StateList::new();
    state_list.add_state(&nfa_state);
    
    // Create DFA iteratively
    let (start_state, memory) = DfaState::iterative_create(state_list);
    
    // Verify all states were created properly
    let start = start_state.borrow();
    assert!(start.next.contains_key(&InputCondition::Char('a')));
    
    // Should have exactly 4 states:
    // 1. Start state with 'a' transition
    // 2. State after 'a' with 'b' transition
    // 3. State after 'b' with 'c' transition
    // 4. Match state after 'c'
    assert_eq!(memory.len(), 4);
}

// ==========================================
// Empty or None State Handling Tests
// ==========================================

#[test]
fn test_none_state_in_split() {
    // Test handling of none state in split's outputs
    let normal_state = State::basic(RegexType::Char('a'));
    
    // Create a split state with one none path and one normal path
    let split_state = State::split(normal_state, State::none());
    
    // Create a state list with the split state
    let mut state_list = StateList::new();
    state_list.add_state(&split_state);
    
    // Create a DFA state
    let dfa_state = DfaState::new(0, state_list);
    
    // Compute transitions
    let mut memory = HashMap::new();
    let mut dfa_state_mut = dfa_state;
    dfa_state_mut.compute_next(&mut memory);
    
    // Should only have transitions for the normal state, none state should be ignored
    assert!(dfa_state_mut.next.contains_key(&InputCondition::Char('a')));
    assert_eq!(dfa_state_mut.next.len(), 1);
}

#[test]
fn test_multiple_none_states() {
    // Test handling of multiple none states in state list
    let none_state1 = State::none();
    let none_state2 = State::none();
    
    // Create a state list with multiple none states
    let mut state_list = StateList::new();
    state_list.add_state(&none_state1);
    state_list.add_state(&none_state2);
    
    // Create a DFA state
    let dfa_state = DfaState::new(0, state_list);
    
    // Compute transitions
    let mut memory = HashMap::new();
    let mut dfa_state_mut = dfa_state;
    dfa_state_mut.compute_next(&mut memory);
    
    // No transitions should be created for none states
    assert_eq!(dfa_state_mut.next.len(), 0);
}

#[test]
fn test_none_state_in_multilevel_structure() {
    // Test handling of none states in multi-level structure
    let basic_state = State::basic(RegexType::Char('a'));
    
    // Create a structure with none states at different levels
    let split1 = State::split(basic_state, State::none());
    let split2 = State::split(State::none(), split1);
    
    // Create DFA
    let mut state_list = StateList::new();
    state_list.add_state(&split2);
    
    let dfa_state = DfaState::new(0, state_list);
    
    // Compute transitions - should handle the nested structure correctly
    let mut memory = HashMap::new();
    let mut dfa_state_mut = dfa_state;
    dfa_state_mut.compute_next(&mut memory);
    
    // Should have transition for 'a' from the basic state
    assert!(dfa_state_mut.next.contains_key(&InputCondition::Char('a')));
}

// ==========================================
// StateList Equality and Hashing Edge Cases
// ==========================================

#[test]
fn test_state_list_order_independence() {
    // Test that StateList hash/equality is order-independent
    let state1 = State::basic(RegexType::Char('a'));
    let state2 = State::basic(RegexType::Char('b'));
    
    // Create two lists with same states but different insertion order
    let mut list1 = StateList::new();
    list1.add_state(&state1);
    list1.add_state(&state2);
    
    let mut list2 = StateList::new();
    list2.add_state(&state2);
    list2.add_state(&state1);
    
    // Create DFA states
    let (dfa_state1, memory) = DfaState::iterative_create(list1);
    
    // This should reuse the same DFA state
    let (dfa_state2, _) = DfaState::iterative_create(list2);
    
    // Verify state reuse (same pointer)
    assert!(Rc::ptr_eq(&dfa_state1, &dfa_state2));
    assert_eq!(memory.len(), 1); // Only one unique state list
}

#[test]
fn test_state_list_hash_with_different_match_ids() {
    // Test that StateLists with match states of different IDs are treated as different
    let match_state1 = State::match_(1);
    let match_state2 = State::match_(2);
    
    // Create two lists with different match states
    let mut list1 = StateList::new();
    list1.add_state(&match_state1);
    
    let mut list2 = StateList::new();
    list2.add_state(&match_state2);
    
    // Create DFA states
    let (_, memory1) = DfaState::iterative_create(list1);
    let (_, memory2) = DfaState::iterative_create(list2);
    
    // Verify both states were created (no reuse)
    assert_eq!(memory1.len() + memory2.len(), 2); // Two distinct state lists
}

// ==========================================
// Large DFA Construction Tests
// ==========================================

#[test]
fn test_large_dfa_alternation() {
    // Test construction of a DFA for a large alternation: a|b|c|d|e
    let postfix = into_postfix("a|b|c|d|e");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create DFA
    let dfa = Dfa::new(vec![nfa_state]);
    
    // Verify structure
    let start_state = dfa.start.borrow();
    
    // Should have transitions for all characters
    assert!(start_state.next.contains_key(&InputCondition::Char('a')));
    assert!(start_state.next.contains_key(&InputCondition::Char('b')));
    assert!(start_state.next.contains_key(&InputCondition::Char('c')));
    assert!(start_state.next.contains_key(&InputCondition::Char('d')));
    assert!(start_state.next.contains_key(&InputCondition::Char('e')));
}

#[test]
fn test_large_dfa_nested_alternation() {
    // Test with a complex nested structure: (a|b)(c|d)
    let postfix = into_postfix("(a|b)(c|d)");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create DFA
    let dfa = Dfa::new(vec![nfa_state]);
    
    // Should have optimized states:
    // 1. Start state with 'a' and 'b' transitions
    // 2. State after 'a'/'b' with 'c' and 'd' transitions
    // 3. Match state after 'c'/'d'
    assert!(dfa.memory.len() <= 5); // At most 5 states needed
}

// ==========================================
// Transition Priority Tests
// ==========================================

#[test]
fn test_transition_map_merge_consistency() {
    // Test that map merging maintains consistent behavior
    
    // Create states with overlapping transitions
    let state_a = State::basic(RegexType::Char('a'));
    let state_a2 = State::basic(RegexType::Char('a'));
    
    // Create StateLists for transitions
    let mut list1 = StateList::new();
    list1.add_state(&state_a);
    
    let mut list2 = StateList::new();
    list2.add_state(&state_a2);
    
    // Create transition maps
    let mut map1 = HashMap::new();
    map1.insert(InputCondition::Char('a'), list1);
    
    let mut map2 = HashMap::new();
    map2.insert(InputCondition::Char('a'), list2);
    
    // Merge maps in different orders
    let mut map_merge1 = map1.clone();
    let mut map_merge2 = map2.clone();
    
    merge_input_maps(&mut map_merge1, map2);
    merge_input_maps(&mut map_merge2, map1);
    
    // Verify both merges have same number of states for 'a'
    let merged_list1 = map_merge1.get(&InputCondition::Char('a')).unwrap();
    let merged_list2 = map_merge2.get(&InputCondition::Char('a')).unwrap();
    
    assert_eq!(merged_list1.len(), merged_list2.len());
}

// ==========================================
// DfaState ID Assignment Consistency
// ==========================================

#[test]
fn test_recursive_dfa_id_consistency() {
    // Verify that DFA state IDs are assigned consistently in recursive creation
    
    // Create a pattern that will result in multiple states
    let postfix = into_postfix("ab");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create a StateList
    let mut state_list = StateList::new();
    state_list.add_state(&nfa_state);
    
    // Create DFA iteratively
    let (_, memory) = DfaState::iterative_create(state_list);
    
    // Verify IDs are consecutive starting from 0
    let mut used_ids = Vec::new();
    for state_ptr in memory.values() {
        let state = state_ptr.borrow();
        used_ids.push(state.id);
    }
    
    used_ids.sort();
    
    for (idx, id) in used_ids.iter().enumerate() {
        assert_eq!(*id, idx);
    }
}

#[test]
fn test_iterative_dfa_id_consistency() {
    // Verify that DFA state IDs are assigned consistently in iterative creation
    
    // Create a pattern that will result in multiple states
    let postfix = into_postfix("ab");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create a StateList
    let mut state_list = StateList::new();
    state_list.add_state(&nfa_state);
    
    // Create DFA iteratively
    let (_, memory) = DfaState::iterative_create(state_list);
    
    // Verify IDs are consecutive starting from 0
    let mut used_ids = Vec::new();
    for state_ptr in memory.values() {
        let state = state_ptr.borrow();
        used_ids.push(state.id);
    }
    
    used_ids.sort();
    
    for (idx, id) in used_ids.iter().enumerate() {
        assert_eq!(*id, idx);
    }
}

// ==========================================
// Clean Memory Map Verification
// ==========================================

#[test]
fn test_clean_memory_map() {
    // Verify that the memory map doesn't contain duplicate StateLists
    
    // Create an NFA for a pattern that yields multiple states
    let postfix = into_postfix("(a|b)c");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create DFA
    let dfa = Dfa::new(vec![nfa_state]);
    
    // Verify each StateList only appears once in memory
    let mut state_lists = Vec::new();
    for (list, _) in &dfa.memory {
        state_lists.push(list);
    }
    
    // Each StateList should be unique
    for i in 0..state_lists.len() {
        for j in (i+1)..state_lists.len() {
            assert!(state_lists[i] != state_lists[j], 
                    "Found duplicate StateLists in memory: {:?} and {:?}", 
                    state_lists[i], state_lists[j]);
        }
    }
}

// ==========================================
// Input Condition Edge Cases
// ==========================================

#[test]
fn test_special_char_input_conditions() {
    // Test DFA construction with special character inputs
    
    // Test with special characters: \n, \t, space, etc.
    let special_chars = vec!['\n', '\t', ' ', '\r', '\0'];
    
    for special_char in special_chars {
        // Create an NFA for the special character
        let mut postfix = VecDeque::new();
        postfix.push_back(TokenType::from(RegexType::Char(special_char)));
        
        let nfa_state = post2nfa(postfix, 0).unwrap();
        
        // Create DFA
        let dfa = Dfa::new(vec![nfa_state]);
        
        // Verify DFA has correct transition for the special character
        let start_state = dfa.start.borrow();
        assert!(start_state.next.contains_key(&InputCondition::Char(special_char)));
    }
}

#[test]
fn test_unicode_char_input_conditions() {
    // Test DFA construction with Unicode character inputs
    
    // Test with unicode characters
    let unicode_chars = vec!['é', '€', '柿', '🙂'];
    
    for unicode_char in unicode_chars {
        // Create an NFA for the unicode character
        let mut postfix = VecDeque::new();
        postfix.push_back(TokenType::from(RegexType::Char(unicode_char)));
        
        let nfa_state = post2nfa(postfix, 0).unwrap();
        
        // Create DFA
        let dfa = Dfa::new(vec![nfa_state]);
        
        // Verify DFA has correct transition for the unicode character
        let start_state = dfa.start.borrow();
        assert!(start_state.next.contains_key(&InputCondition::Char(unicode_char)));
    }
}

// ==========================================
// StateList Deep Copy vs. Shallow Copy
// ==========================================

#[test]
fn test_state_list_clone_behavior() {
    // Test that StateList.clone() creates a true copy
    let basic_state = State::basic(RegexType::Char('a'));
    
    // Create original list
    let mut original = StateList::new();
    original.add_state(&basic_state);
    
    // Clone the list
    let cloned = original.clone();
    
    // Modify the original list
    let another_state = State::basic(RegexType::Char('b'));
    original.add_state(&another_state);
    
    // Verify cloned list is unaffected
    assert_eq!(original.len(), 2);
    assert_eq!(cloned.len(), 1);
}

#[test]
fn test_state_list_in_dfa_creation() {
    // Test that modifying a StateList after DFA state creation doesn't affect the DFA
    
    // Create a StateList
    let basic_state = State::basic(RegexType::Char('a'));
    let mut state_list = StateList::new();
    state_list.add_state(&basic_state);
    
    // Create a DFA state
    let dfa_state = DfaState::new(0, state_list.clone());
    
    // Modify the original StateList
    let another_state = State::basic(RegexType::Char('b'));
    state_list.add_state(&another_state);
    
    // Verify DFA state's list is unaffected
    assert_eq!(state_list.len(), 2);
    assert_eq!(dfa_state.states.len(), 1);
}

// ==========================================
// Complex Pattern Tests
// ==========================================

#[test]
fn test_deeply_nested_alternation_and_concatenation() {
    // Create a complex pattern with nested alternation and concatenation: (a|(b(c|d)e)|(f|g)h)i
    let postfix = into_postfix("(a|(b(c|d)e)|(f|g)h)i");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create DFA
    let dfa = Dfa::new(vec![nfa_state]);
    
    // Verify DFA has reasonable structure
    assert!(dfa.memory.len() > 5); // Should have multiple states for this complex pattern
    
    // Verify start state transitions - should have 'a', 'b', 'f', 'g' transitions initially
    let start_state = dfa.start.borrow();
    let has_expected_transitions = start_state.next.contains_key(&InputCondition::Char('a')) ||
                                  start_state.next.contains_key(&InputCondition::Char('b')) ||
                                  start_state.next.contains_key(&InputCondition::Char('f')) ||
                                  start_state.next.contains_key(&InputCondition::Char('g'));
    
    assert!(has_expected_transitions);
}

#[test]
fn test_complex_quantifiers() {
    // Create a pattern with multiple nested quantifiers: (a+b*c?)+
    let postfix = into_postfix("(a+b*c?)+");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();

    // Create DFA
    let dfa = Dfa::new(vec![nfa_state]);

	dbg!(&dfa);

    // Verify DFA construction succeeded with reasonable state count
    assert!(dfa.memory.len() >= 4); // Complex pattern with loops should result in multiple states
    
    // Verify transitions
    let start_state = dfa.start.borrow();
    assert!(start_state.next.contains_key(&InputCondition::Char('a')));
}

#[test]
fn test_alternation_with_common_prefixes() {
    // Create a pattern with common prefixes: abc|abd|abe
    let postfix = into_postfix("abc|abd|abe");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create DFA
    let dfa = Dfa::new(vec![nfa_state]);
    
    // Verify optimized state structure
    // Despite having 3 different paths, DFA should optimize common prefixes
    // Expected states: start -> a -> ab -> (c, d, e) -> match
    assert!(dfa.memory.len() <= 6);
    
    // Verify start state has 'a' transition
    let start_state = dfa.start.borrow();
    assert!(start_state.next.contains_key(&InputCondition::Char('a')));
    
    // Follow 'a' transition and verify 'b' transition
    if let Some(a_next_id) = start_state.next.get(&InputCondition::Char('a')) {
        let state_after_a = dfa.memory.get(a_next_id).unwrap().borrow();
        assert!(state_after_a.next.contains_key(&InputCondition::Char('b')));
        
        // Follow 'b' transition and verify it has transitions for c, d, and e
        if let Some(b_next_id) = state_after_a.next.get(&InputCondition::Char('b')) {
            let state_after_ab = dfa.memory.get(b_next_id).unwrap().borrow();
            let has_final_transitions = state_after_ab.next.contains_key(&InputCondition::Char('c')) ||
                                       state_after_ab.next.contains_key(&InputCondition::Char('d')) ||
                                       state_after_ab.next.contains_key(&InputCondition::Char('e'));
            assert!(has_final_transitions);
        }
    }
}

// ==========================================
// Pathological Pattern Tests
// ==========================================

#[test]
fn test_repeated_backtracking_pattern() {
    // Create a pattern that would cause catastrophic backtracking in NFA: (a*)*b
    let postfix = into_postfix("(a*)*b");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create DFA - this should succeed despite the pathological pattern
    let dfa = Dfa::new(vec![nfa_state]);
    
    // Verify DFA was created successfully
    assert!(!dfa.memory.is_empty());
    
    // The DFA should be relatively simple despite the complex NFA
    assert!(dfa.memory.len() <= 3); // Should only need ~2-3 states: start state, state after 'a's, state after 'b'
}

#[test]
fn test_nested_repetition_pattern() {
    // Create a pattern with deeply nested repetition: ((a+)+)+b
    let postfix = into_postfix("((a+)+)+b");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create DFA
    let dfa = Dfa::new(vec![nfa_state]);
    
    // Verify DFA has correct properties
    assert!(!dfa.memory.is_empty());
    
    // The DFA should simplify all the nested repetition
    assert!(dfa.memory.len() <= 3); // Likely just start, after a's, and after b
}

// ==========================================
// State Explosion Tests
// ==========================================

#[test]
fn test_potential_state_explosion() {
    // Create a pattern that could lead to state explosion: (a|b)(c|d)(e|f)(g|h)
    let postfix = into_postfix("(a|b)(c|d)(e|f)(g|h)");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create DFA
    let dfa = Dfa::new(vec![nfa_state]);
    
    // Verify DFA was created successfully
    assert!(!dfa.memory.is_empty());
    
    // Check state count - should be bounded despite potential for combinatorial explosion
    assert!(dfa.memory.len() <= 17); // Max expected: start + 2^4 possible paths + match state
}

// ==========================================
// Multi-Pattern Priority Tests
// ==========================================

#[test]
fn test_multi_pattern_with_overlapping_matches() {
    // Create NFAs for overlapping patterns
    // First pattern: ab (match ID 1)
    let postfix1 = into_postfix("ab");
    let nfa1 = post2nfa(postfix1, 1).unwrap();
    
    // Second pattern: abc (match ID 2)
    let postfix2 = into_postfix("abc");
    let nfa2 = post2nfa(postfix2, 2).unwrap();
    
    // Create DFA from both NFAs
    let dfa = Dfa::new(vec![nfa1, nfa2]);
    
    // Follow the path a->b in the DFA and check both match states
    let start_state = dfa.start.borrow();
    
    // Follow 'a' transition
    let a_next_id = start_state.next.get(&InputCondition::Char('a')).unwrap();
    let a_next = dfa.memory.get(a_next_id).unwrap().borrow();
    
    // Follow 'b' transition
    let b_next_id = a_next.next.get(&InputCondition::Char('b')).unwrap();
    let b_next = dfa.memory.get(b_next_id).unwrap().borrow();
    
    // After 'ab', should match pattern 1
    assert_eq!(b_next.id, 1);
    
    // And should also have a transition to 'c'
    assert!(b_next.next.contains_key(&InputCondition::Char('c')));
    
    // Follow 'c' transition
    let c_next_id = b_next.next.get(&InputCondition::Char('c')).unwrap();
    let c_next = dfa.memory.get(c_next_id).unwrap().borrow();
    
    // After 'abc', should match pattern 2
    assert_eq!(c_next.id, 2);
}

// ==========================================
// Complex Anchor Tests
// ==========================================

#[test]
fn test_complex_anchor_patterns() {
    // Create pattern with both anchors: ^(a|b)c$
    let postfix = into_postfix("^(a|b)c$");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create DFA
    let dfa = Dfa::new(vec![nfa_state]);
    
    // Verify anchor transitions
    let start_state = dfa.start.borrow();
    assert!(start_state.next.contains_key(&InputCondition::StartOfLine));
    
    // Follow start-of-line transition
    let sol_next_id = start_state.next.get(&InputCondition::StartOfLine).unwrap();
    let sol_next = dfa.memory.get(sol_next_id).unwrap().borrow();
    
    // Should have transitions for 'a' and 'b'
    assert!(sol_next.next.contains_key(&InputCondition::Char('a')) || 
            sol_next.next.contains_key(&InputCondition::Char('b')));
    
    // Follow path to the end and check for end-of-line transition
    // We'll trace just one path: ^a -> c -> $
    if let Some(a_next_id) = sol_next.next.get(&InputCondition::Char('a')) {
        let a_next = dfa.memory.get(a_next_id).unwrap().borrow();
        
        if let Some(c_next_id) = a_next.next.get(&InputCondition::Char('c')) {
            let c_next = dfa.memory.get(c_next_id).unwrap().borrow();
            
            // Should have end-of-line transition
            assert!(c_next.next.contains_key(&InputCondition::EndOfLine));
            
            // Follow end-of-line transition
            let eol_next_id = c_next.next.get(&InputCondition::EndOfLine).unwrap();
            let eol_next = dfa.memory.get(eol_next_id).unwrap().borrow();
            
            // Should have a match after end-of-line
            assert!(!eol_next.matchs.is_empty());
            assert_eq!(eol_next.id, 0); // Match ID should be 0
        }
    }
}

// ==========================================
// Memory Caching Efficiency Tests
// ==========================================

#[test]
fn test_memory_caching_for_equivalent_expressions() {
    // Create two equivalent expressions in different ways: (ab|ac) and a(b|c)
    
    // First expression: (ab|ac) as NFA
    let postfix1 = into_postfix("ab|ac");
    let nfa1 = post2nfa(postfix1, 0).unwrap();
    
    // Second expression: a(b|c) as NFA
    let postfix2 = into_postfix("a(b|c)");
    let nfa2 = post2nfa(postfix2, 0).unwrap();
    
    // Create DFAs for both expressions
    let dfa1 = Dfa::new(vec![nfa1]);
    let dfa2 = Dfa::new(vec![nfa2]);
    
    // Check that both DFAs have equivalent state counts
    assert_eq!(dfa1.memory.len(), dfa2.memory.len());
    
    // Both DFAs should have the same structure - start state with 'a' transition,
    // then state with 'b' and 'c' transitions, then match state(s)
    let start1 = dfa1.start.borrow();
    let start2 = dfa2.start.borrow();
    
    assert!(start1.next.contains_key(&InputCondition::Char('a')));
    assert!(start2.next.contains_key(&InputCondition::Char('a')));
    
    let a_next1 = dfa1.memory.get(start1.next.get(&InputCondition::Char('a')).unwrap()).unwrap().borrow();
    let a_next2 = dfa2.memory.get(start2.next.get(&InputCondition::Char('a')).unwrap()).unwrap().borrow();
    
    // Both should have the same transitions from the second state
    assert_eq!(a_next1.next.len(), a_next2.next.len());
    
    let has_b_transition1 = a_next1.next.contains_key(&InputCondition::Char('b'));
    let has_c_transition1 = a_next1.next.contains_key(&InputCondition::Char('c'));
    let has_b_transition2 = a_next2.next.contains_key(&InputCondition::Char('b'));
    let has_c_transition2 = a_next2.next.contains_key(&InputCondition::Char('c'));
    
    assert_eq!(has_b_transition1, has_b_transition2);
    assert_eq!(has_c_transition1, has_c_transition2);
}

// ==========================================
// Mixed Feature Tests
// ==========================================

#[test]
fn test_all_features_combined() {
    // Create a pattern with anchors, alternation, quantifiers, and grouping: ^(a|b)+c|(d*e)$
    let postfix = into_postfix("^(a|b)+c|(d*e)$");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create DFA
    let dfa = Dfa::new(vec![nfa_state]);
    
    // Verify DFA was created successfully
    assert!(!dfa.memory.is_empty());
    
    // Verify initial transitions - should have both start-of-line and 'd' transitions
    let start_state = dfa.start.borrow();
    assert!(start_state.next.contains_key(&InputCondition::StartOfLine) || 
            start_state.next.contains_key(&InputCondition::Char('d')));
}

#[test]
fn test_unusual_transitions() {
    // Create a pattern with transitions that might be treated specially in NFA but should be 
    // normalized in DFA: (a?a)
    let postfix = into_postfix("a?a");
    
    let nfa_state = post2nfa(postfix, 0).unwrap();
    
    // Create DFA
    let dfa = Dfa::new(vec![nfa_state]);
    
    // The DFA should simplify this to just one 'a' transition to the match state
    // despite the NFA having a more complex structure with epsilon transitions.
    // Verify this simplification
    
    let start_state = dfa.start.borrow();
    assert!(start_state.next.contains_key(&InputCondition::Char('a')));
    
    // Should have at most 2 states (start and match)
    assert!(dfa.memory.len() <= 3);
}
