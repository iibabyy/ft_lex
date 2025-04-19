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

#[test]
fn test_simulate_basic_matching() {
    // Create a simple DFA for pattern "abc"
    let pattern = "abc";
    let nfa = post2nfa(into_postfix(pattern), 1).unwrap();
    let mut dfa = Dfa::new(vec![nfa]);
    let normalized_dfa = NormalizedDfa::from(&mut dfa);
    
    // Test exact match
    let result = simulate("abc", &normalized_dfa);
    assert!(result.is_some());
    let match_result = result.unwrap();
    assert_eq!(match_result.length(), 3);
    
    // Test partial match
    let result = simulate("ab", &normalized_dfa);
    assert!(result.is_none());
    
    // Test no match
    let result = simulate("xyz", &normalized_dfa);
    assert!(result.is_none());
}

#[test]
fn test_simulate_with_anchors() {
    // Create a DFA for pattern "^abc$"
    let pattern = "^abc$";
    let nfa = post2nfa(into_postfix(pattern), 1).unwrap();
    let mut dfa = Dfa::new(vec![nfa]);
    let normalized_dfa = NormalizedDfa::from(&mut dfa);
    
    // Test exact match with anchors
    let result = simulate("abc\n", &normalized_dfa);
    assert!(result.is_some());
    let match_result = result.unwrap();
    assert_eq!(match_result.length(), 3);
    
    // Test no match when not at start/end
    let result = simulate("xabc", &normalized_dfa);
    assert!(result.is_none());
    
    let result = simulate("abcx", &normalized_dfa);
    assert!(result.is_none());
}

#[test]
fn test_simulate_multiple_matches() {
    // Create a DFA with multiple patterns: "abc" and "ab"
    let nfa1 = post2nfa(into_postfix("abc"), 1).unwrap();
    let nfa2 = post2nfa(into_postfix("ab"), 2).unwrap();
    let mut dfa = Dfa::new(vec![nfa1, nfa2]);
    let normalized_dfa = NormalizedDfa::from(&mut dfa);
    
    // Test with "abc" - should match the longer pattern
    let result = simulate("abc", &normalized_dfa);
    assert!(result.is_some());
    let match_result = result.unwrap();
    assert_eq!(match_result.length(), 3);
    
    // Test with "ab" - should match the shorter pattern
    let result = simulate("ab", &normalized_dfa);
    assert!(result.is_some());
    let match_result = result.unwrap();
    assert_eq!(match_result.length(), 2);
}

#[test]
fn test_simulate_with_newlines() {
    // Create a DFA for pattern "a\nb"
    let pattern = "a\nb";
    let nfa = post2nfa(into_postfix(pattern), 1).unwrap();
    let mut dfa = Dfa::new(vec![nfa]);
    let normalized_dfa = NormalizedDfa::from(&mut dfa);
    
    // Test with newline
    let result = simulate("a\nb", &normalized_dfa);
    assert!(result.is_some());
    let match_result = result.unwrap();
    assert_eq!(match_result.length(), 3);
    
    // Test without newline
    let result = simulate("ab", &normalized_dfa);
    assert!(result.is_none());
}

#[test]
fn test_simulate_edge_cases() {
    // Create a DFA for pattern "a*"
    let pattern = "a*";
    let nfa = post2nfa(into_postfix(pattern), 1).unwrap();
    let mut dfa = Dfa::new(vec![nfa]);
    let normalized_dfa = NormalizedDfa::from(&mut dfa);

    // Test empty string
    let result = simulate("", &normalized_dfa);
    assert!(result.is_some());
	let match_result = result.unwrap();
    assert_eq!(match_result.length(), 0);
    
    // Test very long string
    let long_string = "a".repeat(1000);
    let result = simulate(&long_string, &normalized_dfa);
    assert!(result.is_some());
    let match_result = result.unwrap();
    assert_eq!(match_result.length(), 1000);
}

#[test]
fn test_simulate_with_special_characters() {
    // Create a DFA for pattern "[a-z]+"
    let pattern = "[a-z]+";
    let nfa = post2nfa(into_postfix(pattern), 1).unwrap();
    let mut dfa = Dfa::new(vec![nfa]);
    let normalized_dfa = NormalizedDfa::from(&mut dfa);
    
    // Test with lowercase letters
    let result = simulate("hello", &normalized_dfa);
    assert!(result.is_some());
    let match_result = result.unwrap();
    assert_eq!(match_result.length(), 5);
    
    // Test with uppercase letters (should not match)
    let result = simulate("HELLO", &normalized_dfa);
    assert!(result.is_none());
    
    // Test with mixed case
    let result = simulate("hELLO", &normalized_dfa);
    assert!(result.is_some());
    let match_result = result.unwrap();
    assert_eq!(match_result.length(), 1); // Only 'h' should match
}

#[test]
fn test_simulate_with_nested_patterns() {
    // Create a DFA for pattern "a(b|c)d"
    let pattern = "a(b|c)d";
    let nfa = post2nfa(into_postfix(pattern), 1).unwrap();
    let mut dfa = Dfa::new(vec![nfa]);
    let normalized_dfa = NormalizedDfa::from(&mut dfa);
    
    // Test valid matches
    assert!(simulate("abd", &normalized_dfa).is_some());
    assert!(simulate("acd", &normalized_dfa).is_some());
    
    // Test invalid matches
    assert!(simulate("ad", &normalized_dfa).is_none());
    assert!(simulate("abcd", &normalized_dfa).is_none());
    assert!(simulate("abbd", &normalized_dfa).is_none());
}

#[test]
fn test_simulate_with_alternations() {
    // Create a DFA with multiple alternative patterns of different lengths
    let pattern = "a|abc|abcde";
    let nfa = post2nfa(into_postfix(pattern), 1).unwrap();
    let mut dfa = Dfa::new(vec![nfa]);
    let normalized_dfa = NormalizedDfa::from(&mut dfa);
    
    // Test all valid matches - should match the longest possible
    let result = simulate("a", &normalized_dfa);
    assert!(result.is_some());
    assert_eq!(result.unwrap().length(), 1);
    
    let result = simulate("abc", &normalized_dfa);
    assert!(result.is_some());
    assert_eq!(result.unwrap().length(), 3);
    
    let result = simulate("abcde", &normalized_dfa);
    assert!(result.is_some());
    assert_eq!(result.unwrap().length(), 5);
    
    // Test partial match - should still match the correct part
    let result = simulate("abcdef", &normalized_dfa);
    assert!(result.is_some());
    assert_eq!(result.unwrap().length(), 5);
}

#[test]
fn test_simulate_with_complex_repetition() {
    // Create a DFA with complex repetition
    let pattern = "a{2,4}";  // matches "aa", "aaa", or "aaaa"
    let nfa = post2nfa(into_postfix(pattern), 1).unwrap();
    let mut dfa = Dfa::new(vec![nfa]);
    let normalized_dfa = NormalizedDfa::from(&mut dfa);
    
    // Test valid matches
    assert!(simulate("aa", &normalized_dfa).is_some());
    assert!(simulate("aaa", &normalized_dfa).is_some());
    assert!(simulate("aaaa", &normalized_dfa).is_some());
    
    // Test invalid matches
    assert!(simulate("a", &normalized_dfa).is_none());
    assert!(simulate("aaaaa", &normalized_dfa).is_some()); // Should match first 4 'a's
    let result = simulate("aaaaa", &normalized_dfa);
    assert_eq!(result.unwrap().length(), 4);
}

#[test]
fn test_simulate_with_complex_anchors() {
    // Create a DFA with complex anchored pattern
    let pattern = "^a.*b$";
    let nfa = post2nfa(into_postfix(pattern), 1).unwrap();
    let mut dfa = Dfa::new(vec![nfa]);
    let normalized_dfa = NormalizedDfa::from(&mut dfa);
    
    // Test valid matches
    assert!(simulate("ab\n", &normalized_dfa).is_some());
    assert!(simulate("axyzb\n", &normalized_dfa).is_some());
    
    // Test invalid matches
    assert!(simulate("ab", &normalized_dfa).is_none());
    assert!(simulate("xa", &normalized_dfa).is_none());
    assert!(simulate("abx", &normalized_dfa).is_none());
    assert!(simulate("xaby", &normalized_dfa).is_none());
}

#[test]
fn test_simulate_with_multiline() {
    // Test pattern with both start and end anchors
    let pattern = "^a$";
    let nfa = post2nfa(into_postfix(pattern), 1).unwrap();
    let mut dfa = Dfa::new(vec![nfa]);
    let normalized_dfa = NormalizedDfa::from(&mut dfa);
    
    // Test with multiline text
    let result = simulate("a\nb\nc", &normalized_dfa);
    assert!(result.is_some());
    assert_eq!(result.unwrap().length(), 1);
    
    // Test another multiline pattern
    let pattern = "^b$";
    let nfa = post2nfa(into_postfix(pattern), 1).unwrap();
    let mut dfa = Dfa::new(vec![nfa]);
    let normalized_dfa = NormalizedDfa::from(&mut dfa);
    
    let result = simulate("b\nc", &normalized_dfa);
    assert!(result.is_some());
    assert_eq!(result.unwrap().length(), 1);
}

#[test]
fn test_simulate_with_overlapping_patterns() {
    // Create a DFA with overlapping patterns: "ab" and "abc"
    let nfa1 = post2nfa(into_postfix("ab"), 1).unwrap();
    let nfa2 = post2nfa(into_postfix("abc"), 2).unwrap();
    let mut dfa = Dfa::new(vec![nfa1, nfa2]);
    let normalized_dfa = NormalizedDfa::from(&mut dfa);
    
    // Test with "abc" - should match the longer pattern (id 2)
    let result = simulate("abc", &normalized_dfa);
    assert!(result.is_some());
    
    // TODO: To properly test the match ID, we would need a method to access it
    assert_eq!(result.unwrap().length(), 3);
    
    // Test with "ab" - should match the shorter pattern (id 1)
    let result = simulate("ab", &normalized_dfa);
    assert!(result.is_some());
    assert_eq!(result.unwrap().length(), 2);
}

#[test]
fn test_simulate_with_zero_width_patterns() {
    // Create a DFA with a pattern that can match zero-width: "a*"
    let pattern = "a*";
    let nfa = post2nfa(into_postfix(pattern), 1).unwrap();
    let mut dfa = Dfa::new(vec![nfa]);
    let normalized_dfa = NormalizedDfa::from(&mut dfa);
    
    // Test with empty string (should match)
    let result = simulate("", &normalized_dfa);
    assert!(result.is_some());
    assert_eq!(result.unwrap().length(), 0);
    
    // Test with non-matching string (should still match with zero width)
    let result = simulate("xyz", &normalized_dfa);
    assert!(result.is_some());
    assert_eq!(result.unwrap().length(), 0);
}

#[test]
fn test_simulate_with_complex_character_classes() {
    // Create a DFA with a complex character class: [a-zA-Z0-9_]+
    let pattern = "[a-zA-Z0-9_]+";
    let nfa = post2nfa(into_postfix(pattern), 1).unwrap();
    let mut dfa = Dfa::new(vec![nfa]);
    let normalized_dfa = NormalizedDfa::from(&mut dfa);
    
    // Test with various valid inputs
    let result = simulate("abc123_XYZ", &normalized_dfa);
    assert!(result.is_some());
    assert_eq!(result.unwrap().length(), 10);
    
    // Test with partially valid input
    let result = simulate("abc$123", &normalized_dfa);
    assert!(result.is_some());
    assert_eq!(result.unwrap().length(), 3); // Only "abc" should match
    
    // Test with invalid input
    let result = simulate("!@#$%^", &normalized_dfa);
    assert!(result.is_none());
}


fn test_simulate(pattern: &str, valid_matches: Vec<&str>, invalid_matches: Vec<&str>) {
    // Create a DFA with complex anchored pattern
    let nfa = post2nfa(into_postfix(pattern), 1).unwrap();
    let mut dfa = Dfa::new(vec![nfa]);
    let normalized_dfa = NormalizedDfa::from(&mut dfa);

    // Test valid matches
	for match_ in valid_matches {
		let expected_length = match_.len() - (pattern.chars().last().unwrap() == '$').then_some(1).unwrap_or(0);
		
		let result = simulate(match_, &normalized_dfa);
		assert!(result.is_some(),
			"Expected match for '{}', but got no match",
			match_
		);

		let result = result.unwrap();
		assert!(result.length() == expected_length,
			"Expected length {} for match '{}', but got {}",
			expected_length,
			match_,
			result.length()
		);
	}

    // Test invalid matches
	for match_ in invalid_matches {
		let expected_length = match_.len() - (pattern.chars().last().unwrap() == '$').then_some(1).unwrap_or(0);
		
		let result = simulate(match_, &normalized_dfa);
		
		if result.is_some() {
			let result = result.unwrap();
			assert!(result.length() != expected_length,
				"Expected no (complete) match for '{}', but got a match of length {}",
				match_,
				result.length()
			);
		}
	}
}


#[test]
fn test_simulate_very_complex_pattern() {
    let patterns: Vec<(&str, Vec<&str>, Vec<&str>)> = vec![
        // Nested groups with alternation
        (
            "^(a+|b*)c(d|e)*(f{2,5}|g+)$",	// pattern
            vec!["aacddff\n", "bceeggg\n", "aacddeefff\n"],	// valid matches
            vec!["aacdfg\n", "bceg", "aacddeef\n"]	// invalid matches
        ),
        // Complex character classes with ranges
        (
            "\\w*", // word character
            vec!["abc123_XYZ", "a1b2c3_", "ABC_123"],
            vec!["-abc$123", "!@#$%^", "^abc-123"]
        ),
        // Complex boundaries
        (
            "\\w+([-']\\w+)*",
            vec!["hello-world", "don't", "test-case-123"],
            vec!["-hello", "world-", "test--case"]
        ),
        // Complex alternation with quantifiers
        (
            "(a{2,4}|b+)(c|d){1,3}",
            vec!["aac", "bbbd", "aaaacd", "bcc"],
            vec!["a", "aaaaac", "bdddd"]
        ),
        // Nested optional groups
        (
            "a(b(c(d)?)?)?",
            vec!["a", "ab", "abc", "abcd"],
            vec!["ac", "abd", "abdc", "bcd"]
        ),
        // Complex character class combinations
        (
            "[a-z\\-\\.\\+]{1,25}@[a-z]{2,}(\\.([a-z]{2,})){1,3}",
            vec!["test@example.com", "user.name+tag@sub.domain.co.uk"],
            vec!["@example.com", "test@.com", "test@example", "test@example..com"]
        )
    ];

    for (regex, valid_matches, invalid_matches) in patterns {
        test_simulate(regex, valid_matches, invalid_matches);
    }
}


