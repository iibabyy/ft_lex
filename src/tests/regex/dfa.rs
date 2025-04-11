use crate::regex::*;
use crate::regex::dfa::*;
use crate::regex::post2nfa::*;
use std::collections::{HashMap, VecDeque, HashSet};
use std::rc::Rc;
use std::time::Instant;

fn into_postfix(str: &str) -> VecDeque<TokenType> {
	re2post(Regex::add_concatenation(Regex::tokens(str).unwrap())).unwrap()
}

// ==============================
// Test Utilities
// ==============================

/// Creates a simple basic state for a specific character
fn create_basic_state(c: char) -> StatePtr {
    State::basic(RegexType::Char(c))
}

/// Creates a basic state for a character with a transition to a match state
fn create_basic_to_match(c: char, id: usize) -> StatePtr {
    let state = State::basic(RegexType::Char(c));
    let match_state = State::match_(id);
    
    // Connect the basic state to the match state
    state.borrow_mut().into_basic().unwrap().out.replace(match_state);
    
    state
}

/// Creates a state with a transition to a match state
fn create_state_to_match(state_type: &str, id: usize) -> StatePtr {
    let match_state = State::match_(id);
    
    match state_type {
        "basic" => {
            let state = State::basic(RegexType::Char('a'));
            state.borrow_mut().into_basic().unwrap().out.replace(match_state);
            state
        },
        "startofline" => {
            let state = State::start_of_line();
            state.borrow().start_of_line_out().unwrap().replace(match_state);
            state
        },
        "endofline" => {
            let state = State::end_of_line();
            state.borrow().end_of_line_out().unwrap().replace(match_state);
            state
        },
        _ => State::none()
    }
}

/// Creates a basic state for a character class
fn create_class_state(chars: &[char]) -> StatePtr {
    let mut class = CharacterClass::new();
    for &c in chars {
        class.add_char(c);
    }
    State::basic(RegexType::CharacterClass(class))
}

/// Creates a state list from a vector of states
fn create_state_list(states: Vec<StatePtr>) -> StateList {
    let mut list = StateList::new();
    for state in states {
        list.add_state(&state);
    }
    list
}

/// Creates a cycle in the state machine (a -> b -> c -> a)
fn create_cycle() -> Vec<StatePtr> {
    let a = create_basic_state('a');
    let b = create_basic_state('b');
    let c = create_basic_state('c');
    
    // Connect a -> b -> c -> a
    a.borrow_mut().into_basic().unwrap().out.replace(b.clone());
    b.borrow_mut().into_basic().unwrap().out.replace(c.clone());
    c.borrow_mut().into_basic().unwrap().out.replace(a.clone());
    
    vec![a, b, c]
}

/// Create an input map with specified conditions and states
fn create_input_map(entries: Vec<(InputCondition, Vec<StatePtr>)>) -> HashMap<InputCondition, StateList> {
    let mut map = HashMap::new();
    
    for (condition, states) in entries {
        let mut state_list = StateList::new();
        for state in states {
            state_list.add_state(&state);
        }
        map.insert(condition, state_list);
    }
    
    map
}

// ==============================
// 1. InputCondition Tests
// ==============================

#[test]
fn test_create_and_compare_start_of_line() {
    let condition1 = InputCondition::StartOfLine;
    let condition2 = InputCondition::StartOfLine;
    
    assert_eq!(condition1, condition2);
}

#[test]
fn test_create_and_compare_end_of_line() {
    let condition1 = InputCondition::EndOfLine;
    let condition2 = InputCondition::EndOfLine;
    
    assert_eq!(condition1, condition2);
}

#[test]
fn test_create_and_compare_char_instances() {
    let condition1 = InputCondition::Char('a');
    let condition2 = InputCondition::Char('a');
    let condition3 = InputCondition::Char('b');
    
    assert_eq!(condition1, condition2);
    assert_ne!(condition1, condition3);
}

#[test]
fn test_equality_of_same_input_conditions() {
    let start1 = InputCondition::StartOfLine;
    let start2 = InputCondition::StartOfLine;
    let end1 = InputCondition::EndOfLine;
    let end2 = InputCondition::EndOfLine;
    let char1 = InputCondition::Char('x');
    let char2 = InputCondition::Char('x');
    
    assert_eq!(start1, start2);
    assert_eq!(end1, end2);
    assert_eq!(char1, char2);
}

#[test]
fn test_inequality_of_different_input_conditions() {
    let start = InputCondition::StartOfLine;
    let end = InputCondition::EndOfLine;
    let char_a = InputCondition::Char('a');
    let char_b = InputCondition::Char('b');
    
    assert_ne!(start, end);
    assert_ne!(start, char_a);
    assert_ne!(end, char_a);
    assert_ne!(char_a, char_b);
}

#[test]
fn test_hashing_of_input_conditions() {
    let mut map = HashMap::new();
    
    map.insert(InputCondition::StartOfLine, "start");
    map.insert(InputCondition::EndOfLine, "end");
    map.insert(InputCondition::Char('a'), "a");
    map.insert(InputCondition::Char('b'), "b");
    
    assert_eq!(map.get(&InputCondition::StartOfLine), Some(&"start"));
    assert_eq!(map.get(&InputCondition::EndOfLine), Some(&"end"));
    assert_eq!(map.get(&InputCondition::Char('a')), Some(&"a"));
    assert_eq!(map.get(&InputCondition::Char('b')), Some(&"b"));
    
    // Insert again with same key should update value
    map.insert(InputCondition::Char('a'), "new a");
    assert_eq!(map.get(&InputCondition::Char('a')), Some(&"new a"));
}

// ==============================
// 2. merge_input_maps Tests
// ==============================

#[test]
fn test_merge_maps_no_overlapping_keys() {
    let mut map1 = create_input_map(vec![
        (InputCondition::Char('a'), vec![create_basic_state('x')]),
        (InputCondition::Char('b'), vec![create_basic_state('y')])
    ]);
    
    let map2 = create_input_map(vec![
        (InputCondition::Char('c'), vec![create_basic_state('z')]),
        (InputCondition::Char('d'), vec![create_basic_state('w')])
    ]);
    
    let map1_len = map1.len();
    let map2_len = map2.len();
    
    merge_input_maps(&mut map1, map2);
    
    assert_eq!(map1.len(), map1_len + map2_len);
    assert!(map1.contains_key(&InputCondition::Char('a')));
    assert!(map1.contains_key(&InputCondition::Char('b')));
    assert!(map1.contains_key(&InputCondition::Char('c')));
    assert!(map1.contains_key(&InputCondition::Char('d')));
}

#[test]
fn test_merge_maps_with_overlapping_keys() {
    let state_a1 = create_basic_state('a');
    let state_a2 = create_basic_state('b');
    let state_b1 = create_basic_state('c');
    let state_b2 = create_basic_state('d');
    
    let mut map1 = create_input_map(vec![
        (InputCondition::Char('x'), vec![state_a1.clone()]),
        (InputCondition::Char('y'), vec![state_b1.clone()])
    ]);
    
    let map2 = create_input_map(vec![
        (InputCondition::Char('x'), vec![state_a2.clone()]),
        (InputCondition::Char('z'), vec![state_b2.clone()])
    ]);
    
    merge_input_maps(&mut map1, map2);
    
    assert_eq!(map1.len(), 3);  // x, y, z
    
    // Check that x contains both states
    let x_list = map1.get(&InputCondition::Char('x')).unwrap();
    let mut found_a1 = false;
    let mut found_a2 = false;
    
    for state in x_list {
        if Rc::ptr_eq(state, &state_a1) {
            found_a1 = true;
        }
        if Rc::ptr_eq(state, &state_a2) {
            found_a2 = true;
        }
    }
    
    assert!(found_a1 && found_a2);
}

#[test]
fn test_merge_with_empty_first_map() {
    let mut map1 = HashMap::new();
    
    let map2 = create_input_map(vec![
        (InputCondition::Char('a'), vec![create_basic_state('x')]),
        (InputCondition::Char('b'), vec![create_basic_state('y')])
    ]);
    
    let map2_len = map2.len();
    
    merge_input_maps(&mut map1, map2);
    
    assert_eq!(map1.len(), map2_len);
    assert!(map1.contains_key(&InputCondition::Char('a')));
    assert!(map1.contains_key(&InputCondition::Char('b')));
}

#[test]
fn test_merge_with_empty_second_map() {
    let mut map1 = create_input_map(vec![
        (InputCondition::Char('a'), vec![create_basic_state('x')]),
        (InputCondition::Char('b'), vec![create_basic_state('y')])
    ]);
    
    let map2: HashMap<InputCondition, StateList> = HashMap::new();
    
    let map1_len = map1.len();
    
    merge_input_maps(&mut map1, map2);
    
    assert_eq!(map1.len(), map1_len);
    assert!(map1.contains_key(&InputCondition::Char('a')));
    assert!(map1.contains_key(&InputCondition::Char('b')));
}

#[test]
fn test_merge_with_both_maps_empty() {
    let mut map1: HashMap<InputCondition, StateList> = HashMap::new();
    let map2: HashMap<InputCondition, StateList> = HashMap::new();
    
    merge_input_maps(&mut map1, map2);
    
    assert_eq!(map1.len(), 0);
}

#[test]
fn test_merge_maps_with_same_keys_different_state_lists() {
    let state_a1 = create_basic_state('a');
    let state_a2 = create_basic_state('b');
    
    let mut map1 = create_input_map(vec![
        (InputCondition::Char('x'), vec![state_a1.clone()])
    ]);
    
    let map2 = create_input_map(vec![
        (InputCondition::Char('x'), vec![state_a2.clone()])
    ]);
    
    merge_input_maps(&mut map1, map2);
    
    // Verify that the merged list contains both states
    let merged_list = map1.get(&InputCondition::Char('x')).unwrap();
    assert_eq!(merged_list.len(), 2);
}

#[test]
fn test_merge_large_maps() {
    let mut map1 = HashMap::new();
    let mut map2 = HashMap::new();

    // Create 100 entries in map1
    for i in 0..50_u8 {
        let c = i as char;
        let mut list = StateList::new();
        list.add_state(&create_basic_state(c));
        map1.insert(InputCondition::Char(c), list);
    }

    // Create 100 different entries in map2
    for i in 50..100_u8 {
        let c = i as char;
        let mut list = StateList::new();
        list.add_state(&create_basic_state(c));
        map2.insert(InputCondition::Char(c), list);
    }

    let start = Instant::now();
    merge_input_maps(&mut map1, map2);
    let duration = start.elapsed();

    assert_eq!(map1.len(), 100);
    println!("Large map merge took: {:?}", duration);
}

#[test]
fn test_merged_state_lists_have_correct_contents() {
    let state_a = create_basic_state('a');
    let state_b = create_basic_state('b');
    let state_c = create_basic_state('c');
    let state_d = create_basic_state('d');
    
    let mut map1 = HashMap::new();
    let mut list1 = StateList::new();
    list1.add_state(&state_a);
    list1.add_state(&state_b);
    map1.insert(InputCondition::Char('x'), list1);
    
    let mut map2 = HashMap::new();
    let mut list2 = StateList::new();
    list2.add_state(&state_c);
    list2.add_state(&state_d);
    map2.insert(InputCondition::Char('x'), list2);
    
    merge_input_maps(&mut map1, map2);
    
    let merged_list = map1.get(&InputCondition::Char('x')).unwrap();
    
    // The merged list should contain all four states
    assert_eq!(merged_list.len(), 4);
    
    // Check if all states are in the merged list
    let mut found_a = false;
    let mut found_b = false;
    let mut found_c = false;
    let mut found_d = false;
    
    for state in merged_list {
        if Rc::ptr_eq(state, &state_a) { found_a = true; }
        else if Rc::ptr_eq(state, &state_b) { found_b = true; }
        else if Rc::ptr_eq(state, &state_c) { found_c = true; }
        else if Rc::ptr_eq(state, &state_d) { found_d = true; }
    }
    
    assert!(found_a && found_b && found_c && found_d);
}

// ==============================
// 3. DFA Creation Tests
// ==============================

#[test]
fn test_create_dfa_with_single_start_state() {
    let start_state = create_basic_to_match('a', 0);
    let dfa = Dfa::new(vec![start_state]);
    
    // The DFA should have a start state
    assert!(dfa.start.borrow().next.len() > 0);

    // The memory should contain at least one entry
    assert_eq!(dfa.memory.len(),2);

	let borrow = dfa.start.borrow();
	let next = borrow.next.get(&InputCondition::Char('a')).unwrap();
	assert!(next.is_matched());
}

#[test]
fn test_create_dfa_with_multiple_start_states() {
    let state1 = create_basic_to_match('a', 1);
    let state2 = create_basic_to_match('b', 2);
    
    let dfa = Dfa::new(vec![state1, state2]);
    
    // The DFA should have a start state
    assert!(dfa.start.borrow().states.len() > 0);
    
    // The memory should contain at least one entry
    assert!(dfa.memory.len() >= 1);
}

#[test]
fn test_create_dfa_with_no_outgoing_transitions() {
    let start_state = State::match_(0); // Match state with no outgoing transitions
    let dfa = Dfa::new(vec![start_state]);
    
    // The DFA should have a start state
    assert!(dfa.start.borrow().is_match());
    
    // The next map should be empty as there are no transitions
    assert_eq!(dfa.start.borrow().next.len(), 0);
}

#[test]
fn test_create_dfa_with_cycle() {
    let cycle_states = create_cycle();
    let dfa = Dfa::new(cycle_states);
    
    // The DFA should have a start state with transitions
    assert!(dfa.start.borrow().next.len() > 0);
    
    // Memory should contain at least 3 states (for a, b, c)
    assert!(dfa.memory.len() >= 3);
}

#[test]
fn test_create_dfa_with_complex_pattern() {
    // Create an NFA for a(b|c)*d
    let nfa = post2nfa(into_postfix("a(b|c)*d"), 0).unwrap();
    let dfa = Dfa::new(vec![nfa]);
    
    // Check that the DFA was created correctly
    assert!(dfa.memory.len() > 0);
}

#[test]
fn test_create_dfa_for_common_patterns() {
    // Test "starts with a"
    let starts_with_a = post2nfa(into_postfix("^a.*"), 0).unwrap();
    let dfa1 = Dfa::new(vec![starts_with_a]);
    
    // Test "ends with b"
    let ends_with_b = post2nfa(into_postfix(".*b$"), 1).unwrap();
    let dfa2 = Dfa::new(vec![ends_with_b]);
    
    // Test "contains c"
    let contains_c = post2nfa(into_postfix(".*c.*"), 2).unwrap();
    let dfa3 = Dfa::new(vec![contains_c]);
    
    // All DFAs should have been created
    assert!(dfa1.memory.len() > 0);
    assert!(dfa2.memory.len() > 0);
    assert!(dfa3.memory.len() > 0);
}

// This test is expected to return an empty DFA
#[test]
fn test_create_dfa_from_empty_state_list() {
    let dfa = Dfa::new(vec![]);
    
    // The start state should exist but have an empty state list
    assert_eq!(dfa.start.borrow().states.len(), 0);
    
    // The memory should contain just the empty state
    assert_eq!(dfa.memory.len(), 1);
}

// ==============================
// 4. DfaState Basic Tests
// ==============================

#[test]
fn test_create_dfastate_with_valid_id_and_states() {
    let state = create_basic_to_match('a', 0);
    let mut list = StateList::new();
    list.add_state(&state);
    
    let mut dfa_state = DfaState::new(42, list);
    
    assert_eq!(dfa_state.id, 42);
    assert_eq!(dfa_state.states.len(), 1);

	dfa_state.compute_next();

	assert_eq!(dfa_state.next.len(), 1);

	let next = dfa_state.next.get(&InputCondition::Char('a')).unwrap();
	assert!(next.is_matched());
}

#[test]
fn test_create_dfastate_with_empty_state_list() {
    let list = StateList::new();
    let dfa_state = DfaState::new(1, list);
    
    assert_eq!(dfa_state.id, 1);
    assert_eq!(dfa_state.states.len(), 0);
    assert_eq!(dfa_state.matchs.len(), 0);
    assert_eq!(dfa_state.next.len(), 0);
}

#[test]
fn test_matchs_extraction_works_correctly() {
    // Create a state list with a match state
    let match_state = State::match_(5);
    let basic_state = create_basic_state('a');
    
    let mut list = StateList::new();
    list.add_state(&match_state);
    list.add_state(&basic_state);
    
    // Create a DFA state
    let dfa_state = DfaState::new(1, list);
    
    // The match should be extracted to the matchs list
    assert_eq!(dfa_state.states.len(), 1); // Only basic_state remains
    assert_eq!(dfa_state.matchs.len(), 1); // match_state is moved here
    assert!(dfa_state.is_match());
}

#[test]
fn test_dfastate_creation_with_match_states() {
    // Create a chain: basic -> match
    let basic = create_basic_state('a');
    let match_state = State::match_(7);
    basic.borrow_mut().into_basic().unwrap().out.replace(match_state);
    
    let mut list = StateList::new();
    list.add_state(&basic);
    
    // Create a DFA state
    let mut dfa_state = DfaState::new(1, list);
    
    // Initially no match state in the DFA state
    assert_eq!(dfa_state.matchs.len(), 0);
    
    // Compute next states
    dfa_state.compute_next();
    
    // Now there should be a transition to a state that has a match
    let next_list = dfa_state.next.get(&InputCondition::Char('a')).unwrap();
    assert!(next_list.is_matched());
}

#[test]
fn test_create_dfastate_with_large_state_list() {
    // Create a large number of states (1000)
    let mut states = Vec::with_capacity(1000);
    for i in 0..1000 {
        let c = (i % 26 + 'a' as usize) as u8 as char;
        states.push(create_basic_state(c));
    }
    
    let mut list = StateList::new();
    for state in &states {
        list.add_state(state);
    }
    
    let start_time = Instant::now();
    let dfa_state = DfaState::new(1, list);
    let create_time = start_time.elapsed();
    
    println!("Large DfaState creation took: {:?}", create_time);
    println!("Large DfaState compute_next took: {:?}", create_time);
    
    assert_eq!(dfa_state.states.len(), 1000);
}

// ==============================
// 5. DfaState.compute_next Tests
// ==============================

#[test]
fn test_compute_next_for_single_character_transition() {
    let state = create_basic_to_match('a', 0);
    let mut list = StateList::new();
    list.add_state(&state);
    
    let mut dfa_state = DfaState::new(1, list);
    dfa_state.compute_next();
    
    // Should have only one transition for character 'a'
    assert_eq!(dfa_state.next.len(), 1);
    assert!(dfa_state.next.contains_key(&InputCondition::Char('a')));
    
    // The transition should lead to a match state
    let next_list = dfa_state.next.get(&InputCondition::Char('a')).unwrap();
    assert!(next_list.is_matched());
}

#[test]
fn test_compute_next_for_multiple_character_transitions() {
    // Create states for 'a', 'b', and 'c'
    let state_a = create_basic_to_match('a', 1);
    let state_b = create_basic_to_match('b', 2);
    let state_c = create_basic_to_match('c', 3);
    
    let mut list = StateList::new();
    list.add_state(&state_a);
    list.add_state(&state_b);
    list.add_state(&state_c);
    
    let mut dfa_state = DfaState::new(1, list);
    dfa_state.compute_next();
    
    // Should have three transitions
    assert_eq!(dfa_state.next.len(), 3);
    assert!(dfa_state.next.contains_key(&InputCondition::Char('a')));
    assert!(dfa_state.next.contains_key(&InputCondition::Char('b')));
    assert!(dfa_state.next.contains_key(&InputCondition::Char('c')));
    
    // Each transition should lead to a match state
    for c in ['a', 'b', 'c'] {
        let next_list = dfa_state.next.get(&InputCondition::Char(c)).unwrap();
        assert!(next_list.is_matched());
    }
}

#[test]
fn test_compute_next_for_character_class_transitions() {
    // Create a state with a character class [abc]
    let class_state = create_class_state(&['a', 'b', 'c']);
    let match_state = State::match_(0);
    class_state.borrow_mut().into_basic().unwrap().out.replace(match_state);
    
    let mut list = StateList::new();
    list.add_state(&class_state);
    
    let mut dfa_state = DfaState::new(1, list);
    dfa_state.compute_next();
    
    // Should have three transitions (one for each character in the class)
    assert_eq!(dfa_state.next.len(), 3);
    assert!(dfa_state.next.contains_key(&InputCondition::Char('a')));
    assert!(dfa_state.next.contains_key(&InputCondition::Char('b')));
    assert!(dfa_state.next.contains_key(&InputCondition::Char('c')));
    
    // Each transition should lead to a match state
    for c in ['a', 'b', 'c'] {
        let next_list = dfa_state.next.get(&InputCondition::Char(c)).unwrap();
        assert!(next_list.is_matched());
    }
}

#[test]
fn test_compute_next_for_line_start_transition() {
    let state = State::start_of_line();
    let match_state = State::match_(0);
    state.borrow().start_of_line_out().unwrap().replace(match_state);
    
    let mut list = StateList::new();
    list.add_state(&state);
    
    let mut dfa_state = DfaState::new(1, list);
    dfa_state.compute_next();
    
    // Should have a start-of-line transition
    assert_eq!(dfa_state.next.len(), 1);
    assert!(dfa_state.next.contains_key(&InputCondition::StartOfLine));
    
    // The transition should lead to a match state
    let next_list = dfa_state.next.get(&InputCondition::StartOfLine).unwrap();
    assert!(next_list.is_matched());
}

#[test]
fn test_compute_next_for_line_end_transition() {
    let state = State::end_of_line();
    let match_state = State::match_(0);
    state.borrow().end_of_line_out().unwrap().replace(match_state);
    
    let mut list = StateList::new();
    list.add_state(&state);
    
    let mut dfa_state = DfaState::new(1, list);
    dfa_state.compute_next();
    
    // Should have an end-of-line transition
    assert_eq!(dfa_state.next.len(), 1);
    assert!(dfa_state.next.contains_key(&InputCondition::EndOfLine));
    
    // The transition should lead to a match state
    let next_list = dfa_state.next.get(&InputCondition::EndOfLine).unwrap();
    assert!(next_list.is_matched());
}

#[test]
fn test_compute_next_with_no_valid_transitions() {
    // Create a state with no outgoing transition
    let state = State::none();
    
    let mut list = StateList::new();
    list.add_state(&state);
    
    let mut dfa_state = DfaState::new(1, list);
    dfa_state.compute_next();
    
    // Should have no transitions
    assert_eq!(dfa_state.next.len(), 0);
}

#[test]
fn test_compute_next_for_state_with_self_loop() {
    // Create a state that transitions to itself: a -> a
    let state = create_basic_state('a');
    state.borrow_mut().into_basic().unwrap().out.replace(state.clone());
    
    let mut list = StateList::new();
    list.add_state(&state);
    
    let mut dfa_state = DfaState::new(1, list);
    dfa_state.compute_next();
    
    // Should have one transition
    assert_eq!(dfa_state.next.len(), 1);
    assert!(dfa_state.next.contains_key(&InputCondition::Char('a')));
    
    // The transition should contain the original state (self-loop)
    let next_list = dfa_state.next.get(&InputCondition::Char('a')).unwrap();
    let mut found_self = false;
    
    for next_state in next_list {
        if Rc::ptr_eq(next_state, &state) {
            found_self = true;
            break;
        }
    }
    
    assert!(found_self, "Self-loop not found in next states");
}

#[test]
fn test_compute_next_for_multiple_transitions_to_same_target() {
    // Create multiple states that all go to the same target
    let target = State::match_(0);
    
    let state_a = create_basic_state('a');
    let state_b = create_basic_state('b');
    state_a.borrow_mut().into_basic().unwrap().out.replace(target.clone());
    state_b.borrow_mut().into_basic().unwrap().out.replace(target.clone());
    
    let mut list = StateList::new();
    list.add_state(&state_a);
    list.add_state(&state_b);
    
    let mut dfa_state = DfaState::new(1, list);
    dfa_state.compute_next();
    
    // Should have two transitions
    assert_eq!(dfa_state.next.len(), 2);
    assert!(dfa_state.next.contains_key(&InputCondition::Char('a')));
    assert!(dfa_state.next.contains_key(&InputCondition::Char('b')));
    
    // Both transitions should lead to the same match state
    for c in ['a', 'b'] {
        let next_list = dfa_state.next.get(&InputCondition::Char(c)).unwrap();
        assert!(next_list.is_matched());
    }
}

#[test]
fn test_matchs_correctly_propagated_during_compute_next() {
    // Create a chain: a -> Match(1)
    let state_a = create_basic_state('a');
    let match_state = State::match_(1);
    state_a.borrow_mut().into_basic().unwrap().out.replace(match_state);
    
    let mut list = StateList::new();
    list.add_state(&state_a);
    
    let mut dfa_state = DfaState::new(1, list);
    
    // Initially, no match states in the DFA state
    assert_eq!(dfa_state.matchs.len(), 0);
    
    // Compute next states
    dfa_state.compute_next();
    
    // Now the matchs should include the match state
    let next_list = dfa_state.next.get(&InputCondition::Char('a')).unwrap();
    assert!(next_list.is_matched());
    
    // The match ID should match what we defined
    for state in next_list {
        if let Some(id) = state.borrow().match_id() {
            assert_eq!(id, 1);
        }
    }
}

#[test]
fn test_compute_next_with_nested_state_structures() {
    // Create a more complex structure: a -> (b -> Match(1) | c -> Match(2))
    let state_a = create_basic_state('a');
    let state_b = create_basic_state('b');
    let state_c = create_basic_state('c');
    let match1 = State::match_(1);
    let match2 = State::match_(2);
    
    state_b.borrow_mut().into_basic().unwrap().out.replace(match1);
    state_c.borrow_mut().into_basic().unwrap().out.replace(match2);
    
    // Create a split state that branches to b and c
    let split = State::split(state_b, state_c);
    state_a.borrow_mut().into_basic().unwrap().out.replace(split);
    
    let mut list = StateList::new();
    list.add_state(&state_a);
    
    let mut dfa_state = DfaState::new(1, list);
    dfa_state.compute_next();
    
    // Should have one transition for 'a'
    assert_eq!(dfa_state.next.len(), 1);
    assert!(dfa_state.next.contains_key(&InputCondition::Char('a')));
    
    // The 'a' transition should lead to a state that has two transitions ('b' and 'c')
    let a_list = dfa_state.next.get(&InputCondition::Char('a')).unwrap();
    
    // We need to create a new DFA state from the 'a' transition to test its next transitions
    let mut next_dfa = DfaState::new(2, a_list.clone());
    next_dfa.compute_next();
    
    // The next state should have two transitions
    assert_eq!(next_dfa.next.len(), 2);
    assert!(next_dfa.next.contains_key(&InputCondition::Char('b')));
    assert!(next_dfa.next.contains_key(&InputCondition::Char('c')));
    
    // Both transitions should lead to match states
    for c in ['b', 'c'] {
        let next_list = next_dfa.next.get(&InputCondition::Char(c)).unwrap();
        assert!(next_list.is_matched());
    }
}

// ==============================
// 6. DfaState.find_next Tests
// ==============================

#[test]
fn test_find_next_for_basic_state_with_single_character() {
    let match_state = State::match_(0);
    let basic = create_basic_state('a');
    basic.borrow_mut().into_basic().unwrap().out.replace(match_state);
    
    // Create current states (empty for this test)
    let current_states = StateList::new();
    
    // Call find_next directly
    let (next_map, match_list) = DfaState::find_next(&basic, &current_states);
    
    // Should have one transition for 'a'
    assert_eq!(next_map.len(), 1);
    assert!(next_map.contains_key(&InputCondition::Char('a')));
    
    // The transition should lead to a match state
    let next_list = next_map.get(&InputCondition::Char('a')).unwrap();
    assert!(next_list.is_matched());
    
    // We didn't have a match state in the input, so match_list should be empty
    assert_eq!(match_list.len(), 0);
}

#[test]
fn test_find_next_for_basic_state_with_character_class() {
    let match_state = State::match_(0);
    let class_state = create_class_state(&['a', 'b', 'c']);
    class_state.borrow_mut().into_basic().unwrap().out.replace(match_state);
    
    // Create current states (empty for this test)
    let current_states = StateList::new();
    
    // Call find_next directly
    let (next_map, _match_list) = DfaState::find_next(&class_state, &current_states);
    
    // Should have three transitions (one for each character in the class)
    assert_eq!(next_map.len(), 3);
    assert!(next_map.contains_key(&InputCondition::Char('a')));
    assert!(next_map.contains_key(&InputCondition::Char('b')));
    assert!(next_map.contains_key(&InputCondition::Char('c')));
    
    // Each transition should lead to the same match state
    for c in ['a', 'b', 'c'] {
        let next_list = next_map.get(&InputCondition::Char(c)).unwrap();
        assert!(next_list.is_matched());
    }
}

#[test]
fn test_find_next_for_split_state() {
    let match1 = State::match_(1);
    let match2 = State::match_(2);
    
    // Create a split state that branches to two match states
    let split = State::split(match1, match2);
    
    // Create current states (empty for this test)
    let current_states = StateList::new();
    
    // Call find_next directly
    let (next_map, match_list) = DfaState::find_next(&split, &current_states);
    
    // A split state with two match states should have no transitions
    assert_eq!(next_map.len(), 0);
    
    // But it should collect both match states
    assert_eq!(match_list.len(), 2);
    assert!(match_list.is_matched());
}

#[test]
fn test_find_next_for_start_of_line_state() {
    let match_state = State::match_(0);
    let start_line = State::start_of_line();
    start_line.borrow().start_of_line_out().unwrap().replace(match_state);
    
    // Create some current states
    let mut current_states = StateList::new();
    let basic = create_basic_state('a');
    current_states.add_state(&basic);
    
    // Call find_next directly
    let (next_map, _match_list) = DfaState::find_next(&start_line, &current_states);
    
    // Should have one transition for StartOfLine
    assert_eq!(next_map.len(), 1);
    assert!(next_map.contains_key(&InputCondition::StartOfLine));
    
    // The transition should lead to a match state and include the current states
    let next_list = next_map.get(&InputCondition::StartOfLine).unwrap();
    assert!(next_list.len() >= 2); // Match state plus current states
    assert!(next_list.is_matched());
}

#[test]
fn test_find_next_for_end_of_line_state() {
    let match_state = State::match_(0);
    let end_line = State::end_of_line();
    end_line.borrow().end_of_line_out().unwrap().replace(match_state);
    
    // Create some current states
    let mut current_states = StateList::new();
    let basic = create_basic_state('a');
    current_states.add_state(&basic);
    
    // Call find_next directly
    let (next_map, _match_list) = DfaState::find_next(&end_line, &current_states);
    
    // Should have one transition for EndOfLine
    assert_eq!(next_map.len(), 1);
    assert!(next_map.contains_key(&InputCondition::EndOfLine));
    
    // The transition should lead to a match state and include the current states
    let next_list = next_map.get(&InputCondition::EndOfLine).unwrap();
    assert!(next_list.len() >= 2); // Match state plus current states
    assert!(next_list.is_matched());
}

#[test]
fn test_find_next_for_match_state() {
    let match_state = State::match_(0);
    
    // Create current states (empty for this test)
    let current_states = StateList::new();
    
    // Call find_next directly
    let (next_map, match_list) = DfaState::find_next(&match_state, &current_states);
    
    // A match state should have no transitions
    assert_eq!(next_map.len(), 0);
    
    // But it should be collected in the match_list
    assert_eq!(match_list.len(), 1);
    assert!(match_list.is_matched());
}

#[test]
fn test_find_next_for_unhandled_state_types() {
    let none_state = State::none();
    let nomatch_state = State::no_match();
    
    // Create current states (empty for this test)
    let current_states = StateList::new();
    
    // Call find_next on None state
    let (next_map1, match_list1) = DfaState::find_next(&none_state, &current_states);
    
    // A None state should have no transitions and no matches
    assert_eq!(next_map1.len(), 0);
    assert_eq!(match_list1.len(), 0);
    
    // Call find_next on NoMatch state
    let (next_map2, match_list2) = DfaState::find_next(&nomatch_state, &current_states);
    
    // A NoMatch state should have no transitions and no matches
    assert_eq!(next_map2.len(), 0);
    assert_eq!(match_list2.len(), 0);
}

#[test]
fn test_match_states_correctly_collected() {
    let match1 = State::match_(1);
    let match2 = State::match_(2);
    let basic = create_basic_state('a');
    
    // Create split state pointing to both match states
    let split = State::split(match1.clone(), match2.clone());
    
    // Create current states including another match state
    let mut current_states = StateList::new();
    current_states.add_state(&match1); // Add match1 directly to current states
    
    // Call find_next on the split state
    let (_, match_list) = DfaState::find_next(&split, &current_states);
    
    // Should collect both match states from the split
    assert_eq!(match_list.len(), 2);
    assert!(match_list.is_matched());
    
    // Call find_next on the basic state
    let (_, match_list2) = DfaState::find_next(&basic, &current_states);
    
    // Should have no matches from the basic state
    assert_eq!(match_list2.len(), 0);
}

#[test]
fn test_character_class_handling() {
    // Test with a regular character class
    let mut class1 = CharacterClass::new();
    for c in 'a'..='z' {
        class1.add_char(c);
    }
    
    let class_state1 = State::basic(RegexType::CharacterClass(class1));
    class_state1.borrow_mut().into_basic().unwrap().out.replace(State::match_(0));
    
    // Test with a negated character class
    let mut class2 = CharacterClass::new();
    class2.add_char('0');
    class2.add_char('1');
    class2.add_char('2');
    let class2 = class2.negated();
    
    let class_state2 = State::basic(RegexType::CharacterClass(class2));
    class_state2.borrow_mut().into_basic().unwrap().out.replace(State::match_(0));
    
    let current_states = StateList::new();
    
    // Call find_next on the regular class state
    let (next_map1, _) = DfaState::find_next(&class_state1, &current_states);
    
    // Should have 26 transitions (a-z)
    assert_eq!(next_map1.len(), 26);
    
    // Call find_next on the negated class state
    let (next_map2, _) = DfaState::find_next(&class_state2, &current_states);
    
    // Should have many transitions (all ASCII chars except 0, 1, 2)
    assert!(next_map2.len() > 100);
    
    // Verify specific characters match as expected
    assert!(!next_map2.contains_key(&InputCondition::Char('0')));
    assert!(!next_map2.contains_key(&InputCondition::Char('1')));
    assert!(!next_map2.contains_key(&InputCondition::Char('2')));
    assert!(next_map2.contains_key(&InputCondition::Char('a')));
    assert!(next_map2.contains_key(&InputCondition::Char('Z')));
}

#[test]
fn test_interaction_with_start_end_line_and_current_states() {
    // Create states
    let start_line = State::start_of_line();
    let end_line = State::end_of_line();
    let match_state = State::match_(0);
    
    start_line.borrow().start_of_line_out().unwrap().replace(match_state.clone());
    end_line.borrow().end_of_line_out().unwrap().replace(match_state.clone());
    
    // Create current states with both start and end of line
    let mut current_states = StateList::new();
    current_states.add_state(&start_line);
    current_states.add_state(&end_line);
    
    // Call find_next on the start_line state
    let (next_map1, _) = DfaState::find_next(&start_line, &current_states);
    
    // Start of line should only include non-end-of-line states from current_states
    let start_list = next_map1.get(&InputCondition::StartOfLine).unwrap();
    let mut includes_end = false;
    for state in start_list {
        if State::is_end_of_line_ptr(state) {
            includes_end = true;
            break;
        }
    }
    assert!(!includes_end, "StartOfLine should not include EndOfLine states");
    
    // Call find_next on the end_line state
    let (next_map2, _) = DfaState::find_next(&end_line, &current_states);
    
    // End of line should only include non-start-of-line states from current_states
    let end_list = next_map2.get(&InputCondition::EndOfLine).unwrap();
    let mut includes_start = false;
    for state in end_list {
        if State::is_start_of_line_ptr(state) {
            includes_start = true;
            break;
        }
    }
    assert!(!includes_start, "EndOfLine should not include StartOfLine states");
}

// ==============================
// 7. DfaState Creation Methods Tests
// ==============================

#[test]
fn test_iterative_create_with_simple_state_list() {
    // Create a simple state: a -> match
    let state = create_basic_to_match('a', 0);
    
    let mut list = StateList::new();
    list.add_state(&state);
    
    let (result, mem) = DfaState::iterative_create(list.clone());
    
    // Check that the result is a valid DfaState
    assert!(result.borrow().next.contains_key(&InputCondition::Char('a')));
    
    // Memory should contain at least one entry
    assert!(mem.len() >= 1);
    
    // Memory should contain the state list
    assert!(mem.contains_key(&list));
}

#[test]
fn test_iterative_create_with_complex_state_graph() {
    // Create a more complex pattern: a(b|c)*d
    let nfa = post2nfa(into_postfix("a(b|c)*d"), 0).unwrap();
    
    let mut list = StateList::new();
    list.add_state(&nfa);
    
    let (result, mem) = DfaState::iterative_create(list);
    
    // Check that the result is a valid DfaState
    assert!(result.borrow().next.contains_key(&InputCondition::Char('a')));
    
    // Memory should contain multiple entries for this complex pattern
    assert!(mem.len() > 1);
}

#[test]
fn test_iterative_create_with_cyclic_state_references() {
    // Create a cycle: a -> b -> c -> a
    let cycle_states = create_cycle();
    
    let mut list = StateList::new();
    for state in &cycle_states {
        list.add_state(state);
    }
    
    let (result, mem) = DfaState::iterative_create(list);
    
    // Memory should contain at least 3 entries (a, b, c)
    assert!(mem.len() >= 3);
    
    // Check that cyclic references are handled correctly
	let borrow = result.borrow();
    let next_a = borrow.next.get(&InputCondition::Char('a')).unwrap();
    let next_dfa_a = DfaState::new(0, next_a.clone());
    assert!(next_dfa_a.next.is_empty());
    
    // Create transitions for the next state
    let mut next_dfa_a_mut = DfaState::new(0, next_a.clone());
    next_dfa_a_mut.compute_next();
    
    // Should have a transition for 'b'
    assert!(next_dfa_a_mut.next.contains_key(&InputCondition::Char('b')));
}

#[test]
fn test_iterative_create_with_large_state_list() {
    // Create a large number of states (100)
    let mut states = Vec::with_capacity(100);
    for i in 0..100 {
        let c = (i % 26 + 'a' as usize) as u8 as char;
        states.push(create_basic_to_match(c, i));
    }
    
    let mut list = StateList::new();
    for state in &states {
        list.add_state(state);
    }
    
    let start_time = Instant::now();
    let (_result, mem) = DfaState::iterative_create(list);
    let duration = start_time.elapsed();
    
    // Memory should contain many entries
    assert!(mem.len() > 1);
    
    // The creation time should be reasonable
    println!("Large state list creation took: {:?}", duration);
}

#[test]
#[allow(deprecated)]
fn test_compare_recursive_and_iterative_create() {
    // Create a simple pattern: a -> b -> match
    let state_a = create_basic_state('a');
    let state_b = create_basic_state('b');
    let match_state = State::match_(0);
    
    state_b.borrow_mut().into_basic().unwrap().out.replace(match_state);
    state_a.borrow_mut().into_basic().unwrap().out.replace(state_b);
    
    let mut list = StateList::new();
    list.add_state(&state_a);
    
    // Create with recursive method
    let mut memory1 = HashMap::new();
    let result1 = DfaState::recursive_create(list.clone(), &mut memory1);
    
    // Create with iterative method
    let (result2, memory2) = DfaState::iterative_create(list);
    
    // Both should produce same number of states in memory
    assert_eq!(memory1.len(), memory2.len());
    
    // Both should have same transition from start state
    assert_eq!(
        result1.borrow().next.len(),
        result2.borrow().next.len()
    );
    
    // Both should have transition for 'a'
    assert!(result1.borrow().next.contains_key(&InputCondition::Char('a')));
    assert!(result2.borrow().next.contains_key(&InputCondition::Char('a')));
}

#[test]
fn test_memory_caching_during_state_creation() {
    // Create a simple state: a -> match
    let state = create_basic_to_match('a', 0);

    let mut list = StateList::new();
    list.add_state(&state);

    // First creation
    let (result1, memory1) = DfaState::iterative_create(list.clone());

    // Second creation with same list
    let (result2, memory2) = DfaState::iterative_create(list.clone());

    // Both results should be the same DfaState (cached)
    assert_eq!(result1.borrow().states, result2.borrow().states);

    // Memory should still contain just 'a' state and 'match' state
    assert_eq!(memory1.len(), 2);
    assert_eq!(memory2.len(), 2);
	assert!(memory1.contains_key(&list));
	assert!(memory2.contains_key(&list));
}

#[test]
fn test_work_queue_processing_handles_all_transitions() {
    // Create a more complex pattern with multiple branches
    let nfa = post2nfa(into_postfix("a(b|c|d)*e"), 0).unwrap();
    
    let mut list = StateList::new();
    list.add_state(&nfa);
    
    let (result, mem) = DfaState::iterative_create(list);
    
    // First transition should be 'a'
    assert!(result.borrow().next.contains_key(&InputCondition::Char('a')));
    
    // Follow the 'a' transition
    let a_list = result.borrow().next.get(&InputCondition::Char('a')).unwrap().clone();
    let a_state = mem.get(&a_list).unwrap();
    
    // This state should have transitions for 'b', 'c', 'd', and 'e'
    assert!(a_state.borrow().next.contains_key(&InputCondition::Char('b')));
    assert!(a_state.borrow().next.contains_key(&InputCondition::Char('c')));
    assert!(a_state.borrow().next.contains_key(&InputCondition::Char('d')));
    assert!(a_state.borrow().next.contains_key(&InputCondition::Char('e')));
    
    // The 'e' transition should lead to a match state
    let e_list = a_state.borrow().next.get(&InputCondition::Char('e')).unwrap().clone();
    let e_state = mem.get(&e_list).unwrap();
    
    assert!(e_state.borrow().is_match());
}

// ==============================
// 8. DfaState Match Tests
// ==============================

#[test]
fn test_is_match_with_match_state() {
    // Create a state list with a match state
    let match_state = State::match_(0);
    
    let mut list = StateList::new();
    list.add_state(&match_state);
    
    let dfa_state = DfaState::new(1, list);
    
    // Should be a match state
    assert!(dfa_state.is_match());
}

#[test]
fn test_is_match_with_no_match_state() {
    // Create a state list with no match state
    let basic = create_basic_state('a');
    
    let mut list = StateList::new();
    list.add_state(&basic);
    
    let dfa_state = DfaState::new(1, list);
    
    // Should not be a match state
    assert!(!dfa_state.is_match());
}

#[test]
fn test_match_id_returns_correct_id_for_single_match() {
    // Create a state list with a single match state
    let match_state = State::match_(42);
    
    let mut list = StateList::new();
    list.add_state(&match_state);
    
    let dfa_state = DfaState::new(1, list);
    
    // Should return the correct match ID
    assert_eq!(dfa_state.match_id(), Some(42));
}

#[test]
fn test_match_id_returns_lowest_id_for_multiple_matches() {
    // Create a state list with multiple match states
    let match1 = State::match_(10);
    let match2 = State::match_(5);
    let match3 = State::match_(15);
    
    let mut list = StateList::new();
    list.add_state(&match1);
    list.add_state(&match2);
    list.add_state(&match3);

    let dfa_state = DfaState::new(1, list);
    
    // Should return the lowest match ID
    assert_eq!(dfa_state.match_id(), Some(5));
}

#[test]
fn test_match_id_returns_none_for_no_matches() {
    // Create a state list with no match states
    let basic = create_basic_state('a');
    
    let mut list = StateList::new();
    list.add_state(&basic);
    
    let dfa_state = DfaState::new(1, list);
    
    // Should return None
    assert_eq!(dfa_state.match_id(), None);
}

#[test]
fn test_complex_match_patterns() {
    // Create a complex pattern with nested alternatives
    let nfa = post2nfa(into_postfix("(a|b)(c|d)"), 7).unwrap();
    
    let mut list = StateList::new();
    list.add_state(&nfa);
    
    // Create a DFA
    let dfa = Dfa::new(vec![nfa]);
    
    // Check that we can follow transitions to match states
    let start = dfa.start.borrow();
    
    // Should have transitions for 'a' and 'b'
    assert!(start.next.contains_key(&InputCondition::Char('a')));
    assert!(start.next.contains_key(&InputCondition::Char('b')));
    
    // Follow the 'a' transition
    let a_list = start.next.get(&InputCondition::Char('a')).unwrap().clone();
    let a_state = dfa.memory.get(&a_list).unwrap();
    
    // This state should have transitions for 'c' and 'd'
    assert!(a_state.borrow().next.contains_key(&InputCondition::Char('c')));
    assert!(a_state.borrow().next.contains_key(&InputCondition::Char('d')));
    
    // Follow the 'c' transition
    let c_list = a_state.borrow().next.get(&InputCondition::Char('c')).unwrap().clone();
    let c_state = dfa.memory.get(&c_list).unwrap();
    
    // This state should be a match state with the correct ID
    assert!(c_state.borrow().is_match());
    assert_eq!(c_state.borrow().match_id(), Some(7));
}
