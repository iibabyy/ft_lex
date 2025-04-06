use crate::regex::parsing::*;
use crate::regex::re2post::*;
use crate::regex::post2nfa::*;
use crate::regex::nfa_simulation::*;
use crate::regex::*;
use crate::parsing::error::ParsingResult;
use std::collections::VecDeque;

// Helper function to convert a pattern to NFA
fn pattern_to_nfa(pattern: &str) -> ParsingResult<Nfa> {
    let tokens = Regex::tokens(pattern)?;
    let infix = Regex::add_concatenation(tokens);
    let postfix = re2post(infix)?;
	dbg!(&postfix);
	post2nfa(postfix)
}

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

// Helper function to run NFA simulation that returns the longest match
fn run_simulation(nfa: &Nfa, input: &str, at_start_of_line: bool) -> Option<usize> {
    let mut simulation = NfaSimulation::new(nfa);
    simulation.start(at_start_of_line);
    
    for (i, c) in input.chars().enumerate() {
        let is_last = i == input.len() - 1;
        simulation.step(&c, is_last);
    }
    
    simulation.longest_match
}

// ==============================================
// 1. STATELIST FUNCTIONALITY TESTS
// ==============================================

#[test]
fn test_statelist_creation() {
    // Test empty StateList creation
    let empty_list = StateList::new();
    assert!(empty_list.is_empty());
    
    // Test StateList from a state
    let state = State::basic(RegexType::Char('a'));
    let list_from_state = StateList::from(&state);
    assert!(!list_from_state.is_empty());
    assert_eq!(list_from_state.iter().count(), 1);
}

#[test]
fn test_adding_states() {
    // Create a StateList
    let mut list = StateList::new();
    
    // Add a basic state
    let basic_state = State::basic(RegexType::Char('a'));
    list.add_state(&basic_state);
    assert_eq!(list.iter().count(), 1);
    
    // Add the same state again - shouldn't duplicate
    list.add_state(&basic_state);
    assert_eq!(list.iter().count(), 1);
    
    // Add a different state
    let another_state = State::basic(RegexType::Char('b'));
    list.add_state(&another_state);
    assert_eq!(list.iter().count(), 2);
}

#[test]
fn test_adding_split_states() {
    // Create a StateList
    let mut list = StateList::new();
    
    // Create out1 and out2 states
    let out1 = State::basic(RegexType::Char('a'));
    let out2 = State::basic(RegexType::Char('b'));
    
    // Create a split state
    let split = State::split(out1.clone(), out2.clone());
    
    // Add the split state
    list.add_state(&split);
    
    // Should add both out1 and out2, not the split itself
    assert_eq!(list.iter().count(), 2);
    assert!(list.contains(&out1));
    assert!(list.contains(&out2));
}

#[test]
fn test_contains_state() {
    // Create a StateList
    let mut list = StateList::new();
    
    // Create and add a state
    let state = State::basic(RegexType::Char('a'));
    list.add_state(&state);
    
    // Check that the list contains the state
    assert!(list.contains(&state));
    
    // Create a different state
    let other_state = State::basic(RegexType::Char('b'));
    
    // Check that the list doesn't contain the other state
    assert!(!list.contains(&other_state));
}

#[test]
fn test_is_matched() {
    // Create a StateList
    let mut list = StateList::new();
    
    // Add a basic state (not match)
    let basic_state = State::basic(RegexType::Char('a'));
    list.add_state(&basic_state);
    assert!(!list.is_matched());
    
    // Add a match state
    let match_state = State::match_();
    list.add_state(&match_state);
    assert!(list.is_matched());
}

#[test]
fn test_remove_matches() {
    // Create a StateList
    let mut list = StateList::new();
    
    // Add a basic state and a match state
    let basic_state = State::basic(RegexType::Char('a'));
    let match_state = State::match_();
    
    list.add_state(&basic_state);
    list.add_state(&match_state);
    
    // Verify both states are in the list
    assert_eq!(list.iter().count(), 2);
    assert!(list.is_matched());
    
    // Remove match states
    list.remove_matchs();
    
    // Verify only the basic state remains
    assert_eq!(list.iter().count(), 1);
    assert!(!list.is_matched());
    assert!(list.contains(&basic_state));
}

#[test]
fn test_basic_list_operations() {
    // Create a StateList
    let mut list = StateList::new();
    
    // Test push
    let state = State::basic(RegexType::Char('a'));
    list.push(&state);
    assert_eq!(list.iter().count(), 1);
    
    // Test clear
    list.clear();
    assert!(list.is_empty());
    
    // Test push again after clear
    list.push(&state);
    assert!(!list.is_empty());
}

// ==============================================
// 2. NFA STATUS HANDLING TESTS
// ==============================================

#[test]
fn test_nfa_status_types() {
    // Test Match status with a position
    let match_status = NfaStatus::Match(5);
    
    // Test NoMatch status
    let no_match_status = NfaStatus::NoMatch;
    
    // Test Pending status
    let pending_status = NfaStatus::Pending;
    
    // Since enums don't have meaningful equality tests without extra code,
    // we'll just assert they exist to ensure compilation
    match match_status {
        NfaStatus::Match(pos) => assert_eq!(pos, 5),
        _ => panic!("Expected Match status"),
    }
    
    match no_match_status {
        NfaStatus::NoMatch => (),
        _ => panic!("Expected NoMatch status"),
    }
    
    match pending_status {
        NfaStatus::Pending => (),
        _ => panic!("Expected Pending status"),
    }
}

#[test]
fn test_simulation_status_reporting() {
    // Create an NFA for pattern 'a'
    let nfa = pattern_to_nfa("a").unwrap();
    
    // Create a simulation
    let mut simulation = NfaSimulation::new(&nfa);
    
    // Initial status should be Pending
    match simulation.status() {
        NfaStatus::Pending => (),
        _ => panic!("Expected Pending status initially"),
    }
    
    // Simulate non-matching character to clear current states
    simulation.step(&'b', true);
    
    // Status should be NoMatch now
    match simulation.status() {
        NfaStatus::NoMatch => (),
        _ => panic!("Expected NoMatch status after non-matching step"),
    }
    
    // Restart simulation and match
    simulation.start(true);
    simulation.step(&'a', true);
    
    // Status should be Match now
    match simulation.status() {
        NfaStatus::Match(_) => (),
        _ => panic!("Expected Match status after matching"),
    }
}

// ==============================================
// 3. NFASIMULATION CORE LOGIC TESTS
// ==============================================

#[test]
fn test_nfa_simulation_initialization() {
    // Create an NFA for pattern 'a'
    let nfa = pattern_to_nfa("a").unwrap();
    
    // Create a simulation
    let simulation = NfaSimulation::new(&nfa);
    
    // Use status() to verify initial state is Pending (has current states)
    match simulation.status() {
        NfaStatus::Pending => (),
        _ => panic!("Expected Pending status initially"),
    }
    
    // No longest match initially
    match simulation.status() {
        NfaStatus::Match(_) => panic!("Should not be matched initially"),
        _ => (),
    }
}

#[test]
fn test_switching_states() {
    // Test indirectly through processing multiple characters
    let nfa = pattern_to_nfa("abc").unwrap();
    let mut simulation = NfaSimulation::new(&nfa);
    
    // Process first character - should be pending
    let status1 = simulation.step(&'a', false);
    match status1 {
        NfaStatus::Pending => (),
        _ => panic!("Expected Pending after first character"),
    }
    
    // Process second character - should still be pending
    let status2 = simulation.step(&'b', false);
    match status2 {
        NfaStatus::Pending => (),
        _ => panic!("Expected Pending after second character"),
    }
    
    // Process third character - should match
    let status3 = simulation.step(&'c', true);
    match status3 {
        NfaStatus::Match(_) => (),
        _ => panic!("Expected Match after third character"),
    }
}

#[test]
fn test_start_of_line_checking() {
    // Create an NFA for pattern '^a'
    let nfa = pattern_to_nfa("^a").unwrap();
    
    // Create a simulation
    let mut simulation = NfaSimulation::new(&nfa);
    
    // Default - not at start of line, should not match
    simulation.start(false);
    simulation.step(&'a', true);
    match simulation.status() {
        NfaStatus::NoMatch => (),
        _ => panic!("Should not match when not at start of line"),
    }
    
    // Set start_of_line to true, should match
    simulation.start(true);
    simulation.step(&'a', true);
    match simulation.status() {
        NfaStatus::Match(_) => (),
        _ => panic!("Should match when at start of line"),
    }
    
    // Create an NFA without start of line anchor
    let nfa2 = pattern_to_nfa("a").unwrap();
    let mut simulation2 = NfaSimulation::new(&nfa2);
    
    // Should match regardless of start_of_line flag
    simulation2.start(false);
    simulation2.step(&'a', true);
    match simulation2.status() {
        NfaStatus::Match(_) => (),
        _ => panic!("Should match without start of line anchor"),
    }
}

#[test]
fn test_step_function_basic() {
    // Create an NFA for pattern 'a'
    let nfa = pattern_to_nfa("a").unwrap();
    
    // Create a simulation
    let mut simulation = NfaSimulation::new(&nfa);
    
    // Step with matching character
    let status = simulation.step(&'a', true);
    
    // Should be a match
    match status {
        NfaStatus::Match(_) => (),
        _ => panic!("Expected Match status after matching step"),
    }
    
    // Try a different simulation with non-matching character
    let mut simulation2 = NfaSimulation::new(&nfa);
    
    // Step with non-matching character
    let status2 = simulation2.step(&'b', true);
    
    // Should not be a match
    match status2 {
        NfaStatus::NoMatch => (),
        _ => panic!("Expected NoMatch status after non-matching step"),
    }
}

#[test]
fn test_restart_simulation() {
    // Create an NFA for pattern 'a'
    let nfa = pattern_to_nfa("a").unwrap();
    
    // Create a simulation and make it match something
    let mut simulation = NfaSimulation::new(&nfa);
    simulation.step(&'a', true);
    
    // Should have a match now
    match simulation.status() {
        NfaStatus::Match(_) => (),
        _ => panic!("Expected Match status after matching"),
    }
    
    // Restart simulation
    simulation.start(true);
    
    // Should be pending again (not matched)
    match simulation.status() {
        NfaStatus::Pending => (),
        _ => panic!("Expected Pending status after restart"),
    }
    
    // Match again to verify it works after restart
    simulation.step(&'a', true);
    match simulation.status() {
        NfaStatus::Match(_) => (),
        _ => panic!("Expected Match status after matching again"),
    }
}

// ==============================================
// 4. STATE TRANSITIONS TESTS
// ==============================================

#[test]
fn test_character_matching_transitions() {
    // Create an NFA for pattern 'abc'
    let nfa = pattern_to_nfa("abc").unwrap();
    
    // Create a simulation
    let mut simulation = NfaSimulation::new(&nfa);
    
    // Process first character
    let status = simulation.step(&'a', false);
    match status {
        NfaStatus::Pending => (),
        _ => panic!("Expected Pending status after first character"),
    }
    
    // Process second character
    let status = simulation.step(&'b', false);
    match status {
        NfaStatus::Pending => (),
        _ => panic!("Expected Pending status after second character"),
    }
    
    // Process third character with end of line
    let status = simulation.step(&'c', true);
    match status {
        NfaStatus::Match(_) => (),
        _ => panic!("Expected Match status after third character"),
    }
}

#[test]
fn test_detecting_match_states() {
    // Create an NFA for pattern 'a'
    let nfa = pattern_to_nfa("a").unwrap();
    
    // Create a simulation
    let mut simulation = NfaSimulation::new(&nfa);
    
    // Process matching character
    simulation.step(&'a', true);
    
    // Should have a longest match
    match simulation.status() {
        NfaStatus::Match(pos) => assert_eq!(pos, 1),
        _ => panic!("Expected Match status with position 1"),
    }
}

#[test]
fn test_empty_state_handling() {
    // Create an NFA for pattern 'a'
    let nfa = pattern_to_nfa("a").unwrap();
    
    // Create a simulation
    let mut simulation = NfaSimulation::new(&nfa);
    
    // Make it step with non-matching character to clear current states
    simulation.step(&'b', true);
    
    // Step again should keep NoMatch status
    let status = simulation.step(&'a', true);
    match status {
        NfaStatus::NoMatch => (),
        _ => panic!("Expected NoMatch status when current_states is empty"),
    }
}

// ==============================================
// 5. LINE BOUNDARY HANDLING TESTS
// ==============================================

#[test]
fn test_start_of_line_handling() {
    // Create an NFA for pattern '^a'
    let nfa = pattern_to_nfa("^a").unwrap();
    
    // Create a simulation with start_of_line = true
    let mut simulation = NfaSimulation::new(&nfa);
    simulation.start(true);
    
    // Step with matching character
    let status = simulation.step(&'a', true);
    match status {
        NfaStatus::Match(_) => (),
        _ => panic!("Expected Match status with start-of-line anchor matching"),
    }
    
    // Create a simulation with start_of_line = false
    let mut simulation2 = NfaSimulation::new(&nfa);
    simulation2.start(false);
    
    // Step with matching character but not at start of line
    let status2 = simulation2.step(&'a', true);
    match status2 {
        NfaStatus::NoMatch => (),
        _ => panic!("Expected NoMatch status when not at start of line"),
    }
}

#[test]
fn test_end_of_line_handling() {
    // Create an NFA for pattern 'a$'
    let nfa = pattern_to_nfa("a$").unwrap();
    
    // Create a simulation
    let mut simulation = NfaSimulation::new(&nfa);
    
    // Step with matching character at end of line
    let status = simulation.step(&'a', true);
    match status {
        NfaStatus::Match(_) => (),
        _ => panic!("Expected Match status with end-of-line anchor matching"),
    }
    
    // Create another simulation
    let mut simulation2 = NfaSimulation::new(&nfa);
    
    // Step with matching character not at end of line
    let status2 = simulation2.step(&'a', false);
    match status2 {
        NfaStatus::NoMatch => (),
        _ => panic!("Expected NoMatch status when not at end of line"),
    }
}

#[test]
fn test_both_line_boundaries() {
    // Create an NFA for pattern '^a$'
    let nfa = pattern_to_nfa("^a$").unwrap();
    
    // Create a simulation with start_of_line = true
    let mut simulation = NfaSimulation::new(&nfa);
    simulation.start(true);
    
    // Step with matching character at both boundaries
    let status = simulation.step(&'a', true);
    match status {
        NfaStatus::Match(_) => (),
        _ => panic!("Expected Match status with both line anchors matching"),
    }
    
    // Create a simulation with start_of_line = false
    let mut simulation2 = NfaSimulation::new(&nfa);
    simulation2.start(false);
    
    // Step with matching character but not at start of line
    let status2 = simulation2.step(&'a', true);
    match status2 {
        NfaStatus::NoMatch => (),
        _ => panic!("Expected NoMatch status when not at start of line"),
    }
    
    // Create a simulation with start_of_line = true
    let mut simulation3 = NfaSimulation::new(&nfa);
    simulation3.start(true);
    
    // Step with matching character but not at end of line
    let status3 = simulation3.step(&'a', false);
    match status3 {
        NfaStatus::NoMatch => (),
        _ => panic!("Expected NoMatch status when not at end of line"),
    }
}

// ==============================================
// 6. COMPLETE INPUT MATCHING TESTS
// ==============================================

#[test]
fn test_basic_matching() {
    // Test matching simple patterns
    let nfa_a = pattern_to_nfa("a").unwrap();
    assert!(input_match(&nfa_a, "a"));
    assert!(!input_match(&nfa_a, "b"));
    
    let nfa_hello = pattern_to_nfa("hello").unwrap();
    assert!(input_match(&nfa_hello, "hello"));
    assert!(!input_match(&nfa_hello, "hell"));
    assert!(input_match(&nfa_hello, "hello!"));
}

#[test]
fn test_anchored_pattern_matching() {
    // Test start-of-line anchor
    let nfa_start = pattern_to_nfa("^abc").unwrap();
    assert!(input_match(&nfa_start, "abc"));
    assert!(!input_match(&nfa_start, "xabc"));
    
    // Test end-of-line anchor
    let nfa_end = pattern_to_nfa("abc$").unwrap();
    assert!(input_match(&nfa_end, "abc"));
    assert!(!input_match(&nfa_end, "abcx"));
    
    // Test both anchors
    let nfa_both = pattern_to_nfa("^abc$").unwrap();
    assert!(input_match(&nfa_both, "abc"));
    assert!(!input_match(&nfa_both, "abcx"));
    assert!(!input_match(&nfa_both, "xabc"));
}

#[test]
fn test_special_character_matching() {
    // Test wildcard
    let nfa_any = pattern_to_nfa("a.c").unwrap();
    assert!(input_match(&nfa_any, "abc"));
    assert!(input_match(&nfa_any, "adc"));
    assert!(!input_match(&nfa_any, "ac"));
    
    // Test character class
    let nfa_class = pattern_to_nfa("[abc]").unwrap();
    assert!(input_match(&nfa_class, "a"));
    assert!(input_match(&nfa_class, "b"));
    assert!(input_match(&nfa_class, "c"));
    assert!(!input_match(&nfa_class, "d"));
    
    // Test escaped character
    let nfa_escaped = pattern_to_nfa("\\d").unwrap();
    assert!(input_match(&nfa_escaped, "5"));
    assert!(!input_match(&nfa_escaped, "a"));
}

#[test]
fn test_repetition_matching() {
    // Test Kleene star (0 or more)
    let nfa_star = pattern_to_nfa("a*").unwrap();
    assert!(input_match(&nfa_star, ""));
    assert!(input_match(&nfa_star, "a"));
    assert!(input_match(&nfa_star, "aaa"));
    
    // Test plus (1 or more)
    let nfa_plus = pattern_to_nfa("a+").unwrap();
    assert!(!input_match(&nfa_plus, ""));
    assert!(input_match(&nfa_plus, "a"));
    assert!(input_match(&nfa_plus, "aaa"));
    
    // Test optional (0 or 1)
    let nfa_opt = pattern_to_nfa("a?").unwrap();
    assert!(input_match(&nfa_opt, ""));
    assert!(input_match(&nfa_opt, "a"));
    assert!(input_match(&nfa_opt, "aa"));
    
    // Test exact repetition
    let nfa_exact = pattern_to_nfa("a{3}").unwrap();
    assert!(!input_match(&nfa_exact, "aa"));
    assert!(input_match(&nfa_exact, "aaa"));
    assert!(!input_match(&nfa_exact, "aaaa"));
}

// ==============================================
// 7. EDGE CASES TESTS
// ==============================================

#[test]
fn test_empty_input() {
    // Test that empty pattern returns an error
    let result = pattern_to_nfa("");
    assert!(result.is_err());

    // Test matching empty string with optional pattern
    let nfa_opt = pattern_to_nfa("a?").unwrap();
    assert!(input_match(&nfa_opt, ""));

    // Test matching empty string with anchors
    let nfa_anchors = pattern_to_nfa("^$").unwrap();
    assert!(input_match(&nfa_anchors, ""));
}

#[test]
fn test_single_character_input() {
    // Very basic test but important for boundary conditions
    let nfa = pattern_to_nfa("a").unwrap();
    assert!(input_match(&nfa, "a"));
}

#[test]
fn test_longest_match_wins() {
    // Create an NFA for pattern 'aa?'
    let nfa = pattern_to_nfa("aa?").unwrap();
    
    // Test with our helper function
    let match_pos = run_simulation(&nfa, "aa", true);
    
    // Should have matched the longest version (position 2)
    assert_eq!(match_pos, Some(2));

    // Test with input_match
    assert!(input_match(&nfa, "aa"));
}

#[test]
fn test_alternation() {
    // Test alternation
    let nfa_alt = pattern_to_nfa("a|b").unwrap();
    assert!(input_match(&nfa_alt, "a"));
    assert!(input_match(&nfa_alt, "b"));
    assert!(!input_match(&nfa_alt, "c"));

    // Test alternation with groups
    let nfa_group = pattern_to_nfa("(ab)|(cd)").unwrap();
    assert!(input_match(&nfa_group, "ab"));
    assert!(input_match(&nfa_group, "cd"));
    assert!(!input_match(&nfa_group, "ac"));
}

// ==============================================
// 8. INTEGRATION WITH NFA STRUCTURE TESTS
// ==============================================

#[test]
fn test_different_state_types() {
    // Basic state integration is tested throughout
    
    // Test with patterns that produce split states
    let nfa_split = pattern_to_nfa("a|b").unwrap();
    assert!(input_match(&nfa_split, "a"));
    assert!(input_match(&nfa_split, "b"));
    
    // Test with patterns that require match states
    let nfa_match = pattern_to_nfa("a").unwrap();
    assert!(input_match(&nfa_match, "a"));
}

#[test]
fn test_nfa_completion_conditions() {
    // Test normal completion (step until no more characters)
    let nfa = pattern_to_nfa("abc").unwrap();
    assert!(input_match(&nfa, "abc"));
    
    // Test early completion (stop when match is found)
    let nfa2 = pattern_to_nfa("abc").unwrap();
    
    let mut simulation = NfaSimulation::new(&nfa2);
    simulation.start(true);
    
    simulation.step(&'a', false);  // Pending
    simulation.step(&'b', false);  // Pending
    let status = simulation.step(&'c', true);  // Should match
    
    match status {
        NfaStatus::Match(_) => (),
        _ => panic!("Expected Match status at completion"),
    }
}

#[test]
fn test_complex_integration() {
    // Test a complex regex that exercises many NFA features
    let nfa_complex = pattern_to_nfa("^(a|b)+c*d{2,3}$").unwrap();
    
    // Should match
    assert!(input_match(&nfa_complex, "add"));
    assert!(input_match(&nfa_complex, "bcddd"));
    assert!(input_match(&nfa_complex, "ababccdd"));
    
    // Shouldn't match
    assert!(!input_match(&nfa_complex, "cd"));      // Not enough d's
    assert!(!input_match(&nfa_complex, "abcddddd")); // Too many d's
    assert!(!input_match(&nfa_complex, "xabcdd"));  // Doesn't start with a|b
    assert!(!input_match(&nfa_complex, "abcdde"));  // Doesn't end with d
}
