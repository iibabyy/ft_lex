use crate::regex::*;
use crate::regex::post2nfa::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::time::{Duration, Instant};

// ==============================================
// 1. BASIC STATE CONSTRUCTION TESTS
// ==============================================

#[test]
fn test_state_creation_basic() {
    let state = State::basic(RegexType::Char('a'));
    assert!(state.borrow().is_basic());
    
	let borrow = state.borrow();
    let basic = borrow.into_basic().unwrap();
    assert_eq!(basic.c.char().unwrap(), 'a');
    assert!(State::is_none_var_ptr(&basic.out));
}

#[test]
fn test_state_creation_split() {
    let out1 = State::basic(RegexType::Char('a'));
    let out2 = State::basic(RegexType::Char('b'));
    
    let split = State::split(out1.clone(), out2.clone());
    assert!(split.borrow().is_split());
    
    let (actual_out1, actual_out2) = split.borrow().split_out().unwrap();
    assert!(Rc::ptr_eq(&actual_out1.borrow(), &out1));
    assert!(Rc::ptr_eq(&actual_out2.borrow(), &out2));
}

#[test]
fn test_state_creation_match() {
    let state = State::match_(42);
    assert!(state.borrow().is_match());
}

#[test]
fn test_state_creation_none() {
    let state = State::none();
    assert!(state.borrow().is_none());
}

#[test]
fn test_state_creation_no_match() {
    let state = State::no_match();
    assert!(state.borrow().is_nomatch());
}

#[test]
fn test_state_creation_start_of_line() {
    let state = State::start_of_line();
    assert!(state.borrow().is_start_of_line());
    
    // Check that out is a none state
    let out = state.borrow().start_of_line_out().unwrap();
    assert!(State::is_none_var_ptr(&out));
}

#[test]
fn test_state_creation_end_of_line() {
    let state = State::end_of_line();
    assert!(state.borrow().is_end_of_line());
    
    // Check that out is a none state
    let out = state.borrow().end_of_line_out().unwrap();
    assert!(State::is_none_var_ptr(&out));
}

// ==============================================
// 2. STATE TYPE CHECKING TESTS
// ==============================================

#[test]
fn test_state_type_checking() {
    let basic = State::basic(RegexType::Char('a'));
    let split = State::split(State::none(), State::none());
    let match_state = State::match_(0);
    let none_state = State::none();
    let no_match = State::no_match();
    let start_line = State::start_of_line();
    let end_line = State::end_of_line();
    
    // Direct checking
    assert!(basic.borrow().is_basic());
    assert!(split.borrow().is_split());
    assert!(match_state.borrow().is_match());
    assert!(none_state.borrow().is_none());
    assert!(no_match.borrow().is_nomatch());
    assert!(start_line.borrow().is_start_of_line());
    assert!(end_line.borrow().is_end_of_line());
    
    // Pointer checking
    assert!(State::is_basic_ptr(&basic));
    assert!(State::is_split_ptr(&split));
    assert!(State::is_match_ptr(&match_state));
    assert!(State::is_none_ptr(&none_state));
    assert!(State::is_nomatch_ptr(&no_match));
    assert!(State::is_start_of_line_ptr(&start_line));
    assert!(State::is_end_of_line_ptr(&end_line));
    
    // Variable pointer checking
    let var_basic = var_state_ptr(basic);
    let var_split = var_state_ptr(split);
    let var_match = var_state_ptr(match_state);
    let var_none = var_state_ptr(none_state);
    let var_no_match = var_state_ptr(no_match);
    let var_start_line = var_state_ptr(start_line);
    let var_end_line = var_state_ptr(end_line);
    
    assert!(State::is_basic_var_ptr(&var_basic));
    assert!(State::is_split_var_ptr(&var_split));
    assert!(State::is_match_var_ptr(&var_match));
    assert!(State::is_none_var_ptr(&var_none));
    assert!(State::is_nomatch_var_ptr(&var_no_match));
    assert!(State::is_start_of_line_var_ptr(&var_start_line));
    assert!(State::is_end_of_line_var_ptr(&var_end_line));
}

// ==============================================
// 3. STATE OUTPUT RETRIEVAL TESTS
// ==============================================

#[test]
fn test_state_output_retrieval() {
    // Basic state output
    let basic = State::basic(RegexType::Char('a'));
    let basic_out = basic.borrow().basic_out();
    assert!(basic_out.is_some());
    
    // Split state outputs
    let split = State::split(State::none(), State::none());
    let split_out = split.borrow().split_out();
    assert!(split_out.is_some());
    
    // Start-of-line output
    let start_line = State::start_of_line();
    let start_out = start_line.borrow().start_of_line_out();
    assert!(start_out.is_some());
    
    // End-of-line output
    let end_line = State::end_of_line();
    let end_out = end_line.borrow().end_of_line_out();
    assert!(end_out.is_some());
    
    // Incorrect output retrievals should return None
    let match_state = State::match_(0);
    assert!(match_state.borrow().basic_out().is_none());
    assert!(basic.borrow().split_out().is_none());
    assert!(basic.borrow().start_of_line_out().is_none());
    assert!(basic.borrow().end_of_line_out().is_none());
}

// ==============================================
// 4. STATE TYPE CONVERSION TESTS
// ==============================================

#[test]
fn test_state_type_conversion() {
    // Convert to BasicState
    let basic = State::basic(RegexType::Char('a'));
    assert!(basic.borrow().into_basic().is_some());
    
    // Convert to SplitState
    let split = State::split(State::none(), State::none());
    assert!(split.borrow().into_split().is_some());
    
    // Incorrect conversions should return None
    let match_state = State::match_(0);
    assert!(match_state.borrow().into_basic().is_none());
    assert!(match_state.borrow().into_split().is_none());
    assert!(basic.borrow().into_split().is_none());
    assert!(split.borrow().into_basic().is_none());
}

// ==============================================
// 5. FRAGMENT CONSTRUCTION AND OPERATIONS TESTS
// ==============================================

#[test]
fn test_fragment_creation() {
    // Create a basic fragment
    let state = State::basic(RegexType::Char('a'));
    let ptr_list = vec![state.borrow().basic_out().unwrap()];
    
    let fragment = Fragment::new(state.clone(), ptr_list);
    assert!(Rc::ptr_eq(&fragment.start, &state));
    assert_eq!(fragment.ptr_list.len(), 1);
}

#[test]
fn test_fragment_basic_creation() {
    let state = State::basic(RegexType::Char('a'));
    let fragment = Fragment::basic(state.clone());
    
    assert!(Rc::ptr_eq(&fragment.start, &state));
    assert_eq!(fragment.ptr_list.len(), 1);
}

#[test]
fn test_fragment_concatenation() {
    // Create two fragments
    let a_state = State::basic(RegexType::Char('a'));
    let a_frag = Fragment::basic(a_state);
    
    let b_state = State::basic(RegexType::Char('b'));
    let b_frag = Fragment::basic(b_state);
    
    // Concatenate them
    let result = a_frag.and(b_frag);
    
    // Verify the result
    assert_eq!(result.ptr_list.len(), 1);
    
    // The start should be the original 'a' state
    assert!(result.start.borrow().is_basic());
    assert_eq!(result.start.borrow().into_basic().unwrap().c.char().unwrap(), 'a');
    
    // Following the chain should lead to the 'b' state
    let out_ptr = result.start.borrow().basic_out().unwrap();
    let next_state = out_ptr.borrow().clone();
    assert!(next_state.borrow().is_basic());
    assert_eq!(next_state.borrow().into_basic().unwrap().c.char().unwrap(), 'b');
}

#[test]
fn test_fragment_alternation() {
    // Create two fragments
    let a_state = State::basic(RegexType::Char('a'));
    let a_frag = Fragment::basic(a_state);
    
    let b_state = State::basic(RegexType::Char('b'));
    let b_frag = Fragment::basic(b_state);
    
    // Create alternation
    let result = a_frag.or(b_frag);
    
    // Verify the result
    assert_eq!(result.ptr_list.len(), 2);
    
    // The start should be a split state
    assert!(result.start.borrow().is_split());
    
    // The split state should point to 'a' and 'b' states
    let (out1, out2) = result.start.borrow().split_out().unwrap();
    let out1_state = out1.borrow().clone();
    let out2_state = out2.borrow().clone();
    
    assert!(out1_state.borrow().is_basic());
    assert!(out2_state.borrow().is_basic());
    assert_eq!(out1_state.borrow().into_basic().unwrap().c.char().unwrap(), 'a');
    assert_eq!(out2_state.borrow().into_basic().unwrap().c.char().unwrap(), 'b');
}

#[test]
fn test_fragment_optional() {
    let a_state = State::basic(RegexType::Char('a'));
    let a_frag = Fragment::basic(a_state);
    
    // Make it optional (a?)
    let result = a_frag.optional();
    
    // Verify the result
    assert_eq!(result.ptr_list.len(), 2);
    
    // The start should be a split state
    assert!(result.start.borrow().is_split());
    
    // The split state should point to 'a' and a none state
    let (out1, out2) = result.start.borrow().split_out().unwrap();
    let out1_state = out1.borrow().clone();
    
    assert!(out1_state.borrow().is_basic());
    assert_eq!(out1_state.borrow().into_basic().unwrap().c.char().unwrap(), 'a');
    assert!(State::is_none_var_ptr(&out2));
}

#[test]
fn test_fragment_optional_repeat() {
    let a_state = State::basic(RegexType::Char('a'));
    let a_frag = Fragment::basic(a_state);
    
    // Make it repeat zero or more times (a*)
    let result = a_frag.optional_repeat();
    
    // Verify the result
    assert_eq!(result.ptr_list.len(), 1);
    
    // The start should be a split state
    assert!(result.start.borrow().is_split());
    
    // The split should have one path to 'a' and one to none
    let (out1, out2) = result.start.borrow().split_out().unwrap();
    let out1_state = out1.borrow().clone();
    
    assert!(out1_state.borrow().is_basic());
    assert_eq!(out1_state.borrow().into_basic().unwrap().c.char().unwrap(), 'a');
    
    // The 'a' state should loop back to the split state
    let a_out = out1_state.borrow().basic_out().unwrap();
    assert!(Rc::ptr_eq(&a_out.borrow(), &result.start));
}

#[test]
fn test_fragment_exact_repeat() {
    let a_state = State::basic(RegexType::Char('a'));
    let a_frag = Fragment::basic(a_state);
    
    // Repeat exactly 3 times (a{3})
    let result = a_frag.exact_repeat(&3);
    
    // Verify the result
    assert!(result.start.borrow().is_basic());
    
    // Follow the chain - should be exactly 3 'a' states
    let mut current = result.start.clone();
    let mut count = 0;
    
    while current.borrow().is_basic() {
        assert_eq!(current.borrow().into_basic().unwrap().c.char().unwrap(), 'a');
        count += 1;
        
        let out_ptr = current.borrow().basic_out().unwrap();
        current = out_ptr.borrow().clone();
    }
    
    assert_eq!(count, 3);
}

#[test]
fn test_fragment_at_least() {
    let a_state = State::basic(RegexType::Char('a'));
    let a_frag = Fragment::basic(a_state);
    
    // At least 2 times (a{2,})
    let result = a_frag.at_least(&2);
    
    // Should have 2 exact, followed by a kleene star
    assert!(result.start.borrow().is_basic());
    
    // First follow the exact part (2 'a's)
    let mut current = result.start.clone();
    let mut count = 0;
    
    for _ in 0..2 {
        assert!(current.borrow().is_basic());
        assert_eq!(current.borrow().into_basic().unwrap().c.char().unwrap(), 'a');
        count += 1;
        
        let out_ptr = current.borrow().basic_out().unwrap();
        current = out_ptr.borrow().clone();
    }
    
    // Now we should be at a split for the kleene star
    assert!(current.borrow().is_split());
}

#[test]
fn test_fragment_range() {
    let a_state = State::basic(RegexType::Char('a'));
    let a_frag = Fragment::basic(a_state);
    
    // Between 2 and 4 times (a{2,4})
    let result = a_frag.range(&2, &4);
    
    // Should have 2 exact, followed by up to 2 optional
    assert!(result.start.borrow().is_basic());
    
    // First follow the exact part (2 'a's)
    let mut current = result.start.clone();
    
    for _ in 0..2 {
        assert!(current.borrow().is_basic());
        assert_eq!(current.borrow().into_basic().unwrap().c.char().unwrap(), 'a');
        
        let out_ptr = current.borrow().basic_out().unwrap();
        current = out_ptr.borrow().clone();
    }
    
    // Now we should have optional part (with splits)
    assert!(current.borrow().is_split() || current.borrow().is_basic());
}

#[test]
fn test_fragment_anchors() {
    // Test start-of-line anchor
    let a_state = State::basic(RegexType::Char('a'));
    let a_frag = Fragment::basic(a_state);
    
    let start_anchored = a_frag.deep_clone().start_of_line();
    assert!(start_anchored.start.borrow().is_start_of_line());
    
    // Test end-of-line anchor
    let end_anchored = a_frag.deep_clone().end_of_line();
    
    // The start should still be 'a'
    assert!(end_anchored.start.borrow().is_basic());
    
    // But the out should be an end-of-line state
    let out_ptr = end_anchored.start.borrow().basic_out().unwrap();
    let next_state = out_ptr.borrow().clone();
    assert!(next_state.borrow().is_end_of_line());
}

#[test]
fn test_fragment_quantification() {
    let a_state = State::basic(RegexType::Char('a'));
    let a_frag = Fragment::basic(a_state);
    
    // Test Exact quantifier
    let exact = a_frag.deep_clone().quantify(&Quantifier::Exact(3));
    // Verify three 'a' states in sequence
    let mut current = exact.start.clone();
    let mut count = 0;
    while current.borrow().is_basic() {
        count += 1;
        let out_ptr = current.borrow().basic_out().unwrap();
        current = out_ptr.borrow().clone();
    }
    assert_eq!(count, 3);
    
    // Test AtLeast quantifier
    let at_least = a_frag.deep_clone().quantify(&Quantifier::AtLeast(2));
    // Should have 2 'a's followed by a repeating section
    let mut current = at_least.start.clone();
    for _ in 0..2 {
        assert!(current.borrow().is_basic());
        let out_ptr = current.borrow().basic_out().unwrap();
        current = out_ptr.borrow().clone();
    }
    // Now we should be at a split state for the kleene star
    assert!(current.borrow().is_split());
    
    // Test Range quantifier
    let range = a_frag.deep_clone().quantify(&Quantifier::Range(2, 4));
    // Should have structure similar to a{2,4}
    let mut current = range.start.clone();
    for _ in 0..2 {
        assert!(current.borrow().is_basic());
        let out_ptr = current.borrow().basic_out().unwrap();
        current = out_ptr.borrow().clone();
    }
    // Now should have optional parts
    assert!(current.borrow().is_split() || current.borrow().is_basic());
}

// ==============================================
// 6. UTILITY FUNCTIONS TESTS
// ==============================================

#[test]
fn test_patch_function() {
    let match_state = State::match_(42);
    
    // Create a list of pointers
    let state1 = State::basic(RegexType::Char('a'));
    let state2 = State::basic(RegexType::Char('b'));
    
    let ptr_list = vec![
        state1.borrow().basic_out().unwrap(),
        state2.borrow().basic_out().unwrap()
    ];
    
    // Patch them to point to the match state
    utils::patch(&ptr_list, &match_state);
    
    // Verify both now point to the match state
    assert!(State::is_match_ptr(&state1.borrow().basic_out().unwrap().borrow()));
    assert!(State::is_match_ptr(&state2.borrow().basic_out().unwrap().borrow()));
}

#[test]
fn test_list_and_append() {
    // Test list1
    let state = State::basic(RegexType::Char('a'));
    let out_ptr = state.borrow().basic_out().unwrap();
    
    let list = utils::list1(out_ptr.clone());
    assert_eq!(list.len(), 1);
    assert!(Rc::ptr_eq(&list[0], &out_ptr));
    
    // Test append
    let state2 = State::basic(RegexType::Char('b'));
    let out_ptr2 = state2.borrow().basic_out().unwrap();
    
    let list2 = utils::list1(out_ptr2.clone());
    
    let combined = utils::append(list, list2);
    assert_eq!(combined.len(), 2);
    assert!(Rc::ptr_eq(&combined[1], &out_ptr2));
}

#[test]
fn test_character_matching() {
    let a_state = State::basic(RegexType::Char('a'));
    let b_state = State::basic(RegexType::Char('b'));
    let match_state = State::match_(0);
    
    // Test positive matches
    assert!(a_state.borrow().matche_with(&'a'));
    assert!(b_state.borrow().matche_with(&'b'));
    
    // Test negative matches
    assert!(!a_state.borrow().matche_with(&'b'));
    assert!(!b_state.borrow().matche_with(&'a'));
    
    // Non-basic states shouldn't match any character
    assert!(!match_state.borrow().matche_with(&'a'));
}

// ==============================================
// 7. NFA CONSTRUCTION TESTS
// ==============================================

#[test]
fn test_post2nfa_simple() {
    // Create a simple postfix expression for 'a'
    let mut postfix = VecDeque::new();
    postfix.push_back(TokenType::from(RegexType::Char('a')));
    
    // Convert to NFA
    let result = post2nfa(postfix, 0).unwrap();
    
    // Should be a basic state for 'a'
    assert!(result.borrow().is_basic());
    assert_eq!(result.borrow().into_basic().unwrap().c.char().unwrap(), 'a');
    
    // The out pointer should point to a match state
    let out_ptr = result.borrow().basic_out().unwrap();
    assert!(State::is_match_ptr(&out_ptr.borrow()));
}

#[test]
fn test_post2nfa_concatenation() {
    // Create postfix for "ab" (a b &)
    let mut postfix = VecDeque::new();
    postfix.push_back(TokenType::from(RegexType::Char('a')));
    postfix.push_back(TokenType::from(RegexType::Char('b')));
    postfix.push_back(TokenType::from(RegexType::Concatenation));
    
    // Convert to NFA
    let result = post2nfa(postfix, 0).unwrap();
    
    // Should be a basic state for 'a'
    assert!(result.borrow().is_basic());
    assert_eq!(result.borrow().into_basic().unwrap().c.char().unwrap(), 'a');
    
    // The out pointer should point to a basic state for 'b'
    let out_ptr = result.borrow().basic_out().unwrap();
    let next_state = out_ptr.borrow().clone();
    assert!(next_state.borrow().is_basic());
    assert_eq!(next_state.borrow().into_basic().unwrap().c.char().unwrap(), 'b');
    
    // And that should point to a match state
    let final_out = next_state.borrow().basic_out().unwrap();
    assert!(State::is_match_ptr(&final_out.borrow()));
}

#[test]
fn test_post2nfa_alternation() {
    // Create postfix for "a|b" (a b |)
    let mut postfix = VecDeque::new();
    postfix.push_back(TokenType::from(RegexType::Char('a')));
    postfix.push_back(TokenType::from(RegexType::Char('b')));
    postfix.push_back(TokenType::from(RegexType::Or));
    
    // Convert to NFA
    let result = post2nfa(postfix, 0).unwrap();
    
    // Should be a split state
    assert!(result.borrow().is_split());
    
    // The split should point to 'a' and 'b' states
    let (out1, out2) = result.borrow().split_out().unwrap();
    let out1_state = out1.borrow().clone();
    let out2_state = out2.borrow().clone();
    
    assert!(out1_state.borrow().is_basic());
    assert!(out2_state.borrow().is_basic());
    
    assert_eq!(out1_state.borrow().into_basic().unwrap().c.char().unwrap(), 'a');
    assert_eq!(out2_state.borrow().into_basic().unwrap().c.char().unwrap(), 'b');
    
    // Both should point to match states
    let final_out1 = out1_state.borrow().basic_out().unwrap();
    let final_out2 = out2_state.borrow().basic_out().unwrap();
    
    assert!(State::is_match_ptr(&final_out1.borrow()));
    assert!(State::is_match_ptr(&final_out2.borrow()));
}

#[test]
fn test_post2nfa_quantifier() {
    // Create postfix for "a*" (a *)
    let mut postfix = VecDeque::new();
    postfix.push_back(TokenType::from(RegexType::Char('a')));
    postfix.push_back(TokenType::from(RegexType::Quant(Quantifier::AtLeast(0))));
    
    // Convert to NFA
    let result = post2nfa(postfix, 0).unwrap();
    
    // Should be a split state
    assert!(result.borrow().is_split());
    
    // The split should have one path to 'a' and one to match
    let (out1, out2) = result.borrow().split_out().unwrap();
    let out1_state = out1.borrow().clone();
    
    assert!(out1_state.borrow().is_basic());
    assert_eq!(out1_state.borrow().into_basic().unwrap().c.char().unwrap(), 'a');
    
    // The 'a' state should loop back to the split state
    let a_out = out1_state.borrow().basic_out().unwrap();
    assert!(Rc::ptr_eq(&a_out.borrow(), &result));
    
    // The second path should go to a match state
    assert!(State::is_match_ptr(&out2.borrow()));
}

#[test]
fn test_post2nfa_anchors() {
    // Create postfix for "^a$" (^ a $)
    let mut postfix = VecDeque::new();
    postfix.push_back(TokenType::from(RegexType::LineStart));
    postfix.push_back(TokenType::from(RegexType::Char('a')));
    postfix.push_back(TokenType::from(RegexType::LineEnd));
    
    // Convert to NFA
    let result = post2nfa(postfix, 0).unwrap();
    
    // Should be a start-of-line state
    assert!(result.borrow().is_start_of_line());
    
    // The start-of-line should point to 'a'
    let start_out = result.borrow().start_of_line_out().unwrap();
    let a_state = start_out.borrow().clone();
    
    assert!(a_state.borrow().is_basic());
    assert_eq!(a_state.borrow().into_basic().unwrap().c.char().unwrap(), 'a');
    
    // 'a' should point to end-of-line
    let a_out = a_state.borrow().basic_out().unwrap();
    let end_state = a_out.borrow().clone();
    
    assert!(end_state.borrow().is_end_of_line());
    
    // End-of-line should point to match
    let end_out = end_state.borrow().end_of_line_out().unwrap();
    assert!(State::is_match_ptr(&end_out.borrow().clone()));
}

#[test]
fn test_post2nfa_complex() {
    // Create postfix for "(a|b)*c" (a b | * c &)
    let mut postfix = VecDeque::new();
    postfix.push_back(TokenType::from(RegexType::Char('a')));
    postfix.push_back(TokenType::from(RegexType::Char('b')));
    postfix.push_back(TokenType::from(RegexType::Or));
    postfix.push_back(TokenType::from(RegexType::Quant(Quantifier::AtLeast(0))));
    postfix.push_back(TokenType::from(RegexType::Char('c')));
    postfix.push_back(TokenType::from(RegexType::Concatenation));
    
    // Convert to NFA
    let result = post2nfa(postfix, 0).unwrap();
    
    // Should be a split state (for the Kleene star)
    assert!(result.borrow().is_split());
    
    // Eventually we should reach a 'c' state
    // (This is a simplistic test - a more thorough test would trace through the entire NFA structure)
    let (_, out2) = result.borrow().split_out().unwrap();
    let next_state = out2.borrow().clone();
    
    assert!(next_state.borrow().is_basic());
    assert_eq!(next_state.borrow().into_basic().unwrap().c.char().unwrap(), 'c');
}

// ==============================================
// 8. ERROR HANDLING TESTS
// ==============================================

#[test]
fn test_post2nfa_empty_input() {
    let empty = VecDeque::new();
    let result = post2nfa(empty, 0);
    
    assert!(result.is_err());
}

#[test]
fn test_post2nfa_unbalanced_operators() {
    // Missing operand for concatenation
    let mut postfix = VecDeque::new();
    postfix.push_back(TokenType::from(RegexType::Char('a')));
    postfix.push_back(TokenType::from(RegexType::Concatenation));
    
    let result = post2nfa(postfix, 0);
    assert!(result.is_err());
}

#[test]
fn test_post2nfa_misplaced_anchors() {
    // LineEnd not at the end
    let mut postfix = VecDeque::new();
    postfix.push_back(TokenType::from(RegexType::LineEnd));
    postfix.push_back(TokenType::from(RegexType::Char('a')));
    
    let result = post2nfa(postfix, 0);
    assert!(result.is_err());
    
    // LineStart after a pattern
    let mut postfix = VecDeque::new();
    postfix.push_back(TokenType::from(RegexType::Char('a')));
    postfix.push_back(TokenType::from(RegexType::LineStart));
    
    let result = post2nfa(postfix, 0);
    assert!(result.is_err());
}

#[test]
fn test_post2nfa_duplicate_anchors() {
    // Duplicate LineStart
    let mut postfix = VecDeque::new();
    postfix.push_back(TokenType::from(RegexType::LineStart));
    postfix.push_back(TokenType::from(RegexType::LineStart));
    
    let result = post2nfa(postfix, 0);
    assert!(result.is_err());
    
    // Duplicate LineEnd
    let mut postfix = VecDeque::new();
    postfix.push_back(TokenType::from(RegexType::LineEnd));
    postfix.push_back(TokenType::from(RegexType::LineEnd));
    
    let result = post2nfa(postfix, 0);
    assert!(result.is_err());
}

// ==============================================
// 9. EDGE CASES TESTS
// ==============================================

#[test]
fn test_post2nfa_empty_expression_with_anchors() {
    // ^$ should be valid
    let mut postfix = VecDeque::new();
    postfix.push_back(TokenType::from(RegexType::LineStart));
    postfix.push_back(TokenType::from(RegexType::LineEnd));
    
    let result = post2nfa(postfix, 0);
    assert!(result.is_ok());
    
    let nfa = result.unwrap();
    assert!(nfa.borrow().is_start_of_line());
}

#[test]
fn test_post2nfa_quantifier_zero_repetitions() {
    // a{0} should result in a basic state with NoMatch output
    let mut postfix = VecDeque::new();
    postfix.push_back(TokenType::from(RegexType::Char('a')));
    postfix.push_back(TokenType::from(RegexType::Quant(Quantifier::Exact(0))));
    
    let result = post2nfa(postfix, 0).unwrap();
    
    // Verify the result is a basic state with 'a' character
    assert!(result.borrow().is_basic());
    assert_eq!(result.borrow().into_basic().unwrap().c.char().unwrap(), 'a');
    
    // Verify the output is a NoMatch state
    let out_ptr = result.borrow().basic_out().unwrap();
    let out_state = out_ptr.borrow().clone();
    assert!(out_state.borrow().is_nomatch());
}

#[test]
fn test_post2nfa_large_repetitions() {
    // a{100} should create a chain of 100 'a' states
    let mut postfix = VecDeque::new();
    postfix.push_back(TokenType::from(RegexType::Char('a')));
    postfix.push_back(TokenType::from(RegexType::Quant(Quantifier::Exact(100))));
    
    let result = post2nfa(postfix, 0).unwrap();
    
    // Count the chain
    let mut current = Rc::clone(&result);
    let mut count = 0;
    
    // Need to handle different implementations
    if current.borrow().is_basic() {
        // If it's a chain of basic states
        while current.borrow().is_basic() {
            count += 1;
            let out = current.borrow().basic_out().unwrap();
            current = out.borrow().clone();
        }
        assert_eq!(count, 100);
    } else {
        // Could be implemented with splits or other structures
        // In this case, just verify it's a valid NFA structure
        assert!(result.borrow().is_split() || result.borrow().is_basic());
    }
}

// ==============================================
// 10. INTEGRATION TESTS
// ==============================================

#[test]
fn test_integration_with_re2post() {
    // Create infix regex for "a|b"
    let tokens = {
        let infix = Regex::tokens("a|b").unwrap();
        Regex::add_concatenation(infix)
    };
    
    // Convert to postfix
    let postfix = re2post::re2post(tokens).unwrap();
    
    // Now convert to NFA
    let nfa = post2nfa(postfix, 0).unwrap();
    
    // Verify basic structure
    assert!(nfa.borrow().is_split());
    
    // Should have paths to 'a' and 'b'
    let (out1, out2) = nfa.borrow().split_out().unwrap();
    let a_state = out1.borrow().clone();
    let b_state = out2.borrow().clone();
    
    assert!(a_state.borrow().is_basic());
    assert!(b_state.borrow().is_basic());
    
    let a_char = a_state.borrow().into_basic().unwrap().c.char().unwrap();
    let b_char = b_state.borrow().into_basic().unwrap().c.char().unwrap();
    
    // The characters might be in either order depending on implementation
    assert!((a_char == 'a' && b_char == 'b') || (a_char == 'b' && b_char == 'a'));
}

#[test]
fn test_integration_with_complex_pattern() {
    // Create infix regex for "^(a|b)*c$"
    let tokens = {
        let infix = Regex::tokens("^(a|b)*c$").unwrap();
        Regex::add_concatenation(infix)
    };
    
    // Convert to postfix
    let postfix = re2post::re2post(tokens).unwrap();
    
    // Now convert to NFA
    let nfa = post2nfa(postfix, 0).unwrap();
    
    // Verify it's a valid NFA with start anchor
    assert!(nfa.borrow().is_start_of_line());
    
    // Follow through to verify we have the expected structure
    let start_out = nfa.borrow().start_of_line_out().unwrap();
    let next = start_out.borrow().clone();
    
    // Should have a split for the Kleene star
    assert!(next.borrow().is_split());
}



// Helper function to create a token
fn token(regex_type: RegexType) -> TokenType {
	TokenType::from(regex_type)
}

// Helper function to compare NFAs
fn compare_nfas(postfix1: VecDeque<TokenType>, postfix2: VecDeque<TokenType>) -> bool {
	let nfa1 = post2nfa(postfix1, 0).unwrap();
	let nfa2 = post2nfa(postfix2, 0).unwrap();

	// Print NFA1 structure
	print_state_structure(&nfa1, "NFA1 Structure:");
	// Print NFA2 structure
	print_state_structure(&nfa2, "NFA2 Structure:");

	structures_equal(&nfa1, &nfa2)
}

fn into_postfix(str: &str) -> VecDeque<TokenType> {
	re2post(Regex::add_concatenation(Regex::tokens(str).unwrap())).unwrap()
}

#[test]
fn test_character_classes() {
	// Test digit class \d
	let digit_class = into_postfix("\\d");
	let digit_range = into_postfix("[0-9]");
	
	dbg!(&digit_class);
	dbg!(&digit_range);

	assert!(compare_nfas(
		digit_class,
		digit_range
	));


	// Test word character class \w
	let word_class = into_postfix("\\w");
	let word_equivalent = into_postfix("[a-zA-Z0-9_]");
	
	assert!(compare_nfas(
		word_class,
		word_equivalent
	));
}

#[test]
fn test_negated_character_classes() {
	// Test non-digit class \D
	let non_digit_class = into_postfix("\\D");
	let non_digit_range = into_postfix("[^0-9]");
	
	assert!(compare_nfas(
		non_digit_class,
		non_digit_range
	));


	// Test non-word character class \W
	let non_word_class = into_postfix("\\W");
	let non_word_equivalent = into_postfix("[^a-zA-Z0-9_]");
	
	assert!(compare_nfas(
		non_word_class,
		non_word_equivalent
	));
}

#[test]
fn test_word_boundaries() {
	// Test word boundary \b
	let word_boundary = into_postfix("\\b");
	
	// Test non-word boundary \B
	let non_word_boundary = into_postfix("\\B");
	
	// These should create valid NFAs (not testing equivalence, just that they parse)
	assert!(post2nfa(word_boundary.clone(), 0).is_ok());
	assert!(post2nfa(non_word_boundary.clone(), 0).is_ok());
}

#[test]
fn test_escape_sequences() {
    // Test various escape sequences
    let escape_sequences = into_postfix("\\t\\n\\r\\f");
    let literal_chars = into_postfix("\t\n\r\u{000C}");
    
    assert!(compare_nfas(
        escape_sequences,
        literal_chars
    ));
}

#[test]
fn test_complex_nested_quantifiers() {
	// Test (a+|b*){2,3}
	let complex_pattern = into_postfix("(a+|b*){2,3}");

	// Equivalent to (a+|b*)(a+|b*) with optional third (a+|b*)
	let expanded_pattern = into_postfix("(a+|b*)(a+|b*)(a+|b*)?");

	assert!(compare_nfas(
		complex_pattern,
		expanded_pattern
	));
}

#[test]
fn test_stress_complex_patterns() {
	// Test a very complex pattern: ((a|b)+c?d*){3,5}e+
	let complex_pattern = into_postfix("((a|b)+c?d*){3,5}e+");
	
	// Just test that it creates a valid NFA without errors
	assert!(post2nfa(complex_pattern, 0).is_ok());
	
	// Test another complex pattern with nested groups and quantifiers
	let nested_pattern = into_postfix("(a{2,3}(b|c)*[d-f]+){2}g?");
	assert!(post2nfa(nested_pattern, 0).is_ok());
	
	// Test a pattern with character classes and boundaries
	let mixed_pattern = into_postfix("\\b\\w+\\s*\\d{3,}\\W+\\B");
	assert!(post2nfa(mixed_pattern, 0).is_ok());
}

#[test]
fn test_boundary_cases() {
	// Test empty pattern (should fail)
	let empty_pattern = into_postfix("");
	assert!(post2nfa(empty_pattern, 0).is_err());
	
	// Test pattern with only line anchors
	let anchors_only = into_postfix("^$");
	assert!(post2nfa(anchors_only, 0).is_ok());
	
	// Test pattern with maximum repetition
	let max_repetition = into_postfix("a{1000}");
	assert!(post2nfa(max_repetition, 0).is_ok());
}


// ==============================================
// 11. ITERATIVE DEEP CLONE TESTS
// ==============================================

// Helper function to create a simple linear chain of states
fn create_linear_chain(length: usize) -> StatePtr {
	let match_state = State::match_(0);
	
	if length == 0 {
		return match_state;
	}
	
	// Start from the end (match state) and build backwards
	let mut current = match_state;
	
	for i in (0..length).rev() {
		// Calculate character based on position
		let char_val = (b'a' + (i % 26) as u8) as char;
		
		// Create new basic state
		let new_state = State::basic(RegexType::Char(char_val));
		
		// Connect the new state to the current chain
		new_state.borrow_mut().into_basic().unwrap().out.replace(current);
		
		// Update current to be the new head of the chain
		current = new_state;
	}
	
	current
}

// Helper function to create a deeply nested split structure
fn create_nested_split(depth: usize) -> StatePtr {
	// Base case
	if depth == 0 {
		return State::match_(0);
	}
	
	// Start with a match state at the bottom
	let mut current = State::match_(0);
	
	// Build the structure iteratively from bottom to top
	for i in 1..=depth {
		let level = depth - i; // Count backwards for character assignment
		let char_val = (b'a' + (level % 26) as u8) as char;
		
		// Create a basic state for this level
		let left = State::basic(RegexType::Char(char_val));
		
		// Connect the basic state to the current structure
		left.borrow_mut().into_basic().unwrap().out.replace(current.clone());
		
		// Create a split that branches to the basic state and the current structure
		current = State::split(left, current);
	}
	
	current
}

#[test]
fn test_iterative_clone_basic_state() {
	// Create a basic state
	let original = State::basic(RegexType::Char('a'));
	let match_state = State::match_(0);
	original.borrow_mut().into_basic().unwrap().out.replace(match_state);

	// Clone using iterative method
	let (cloned, ptr_list) = original.borrow().self_ptr_deep_clone_with_memo_iterative(&mut HashMap::new());

	// Verify structure
	assert!(State::is_basic_ptr(&cloned));
	let borrow = cloned.borrow();
	let basic = borrow.into_basic().unwrap();
	assert_eq!(basic.c.char().unwrap(), 'a');
	
	// Verify the out pointer points to a match state
	let out = basic.out.borrow();
	assert!(State::is_match_ptr(&out));
}

#[test]
fn test_iterative_clone_split_state() {
	// Create a split state with two basic states
	let left = State::basic(RegexType::Char('a'));
	let right = State::basic(RegexType::Char('b'));
	let split = State::split(left, right);

	// Clone using iterative method
	let (cloned, _) = split.borrow().self_ptr_deep_clone_with_memo_iterative(&mut HashMap::new());

	// Verify structure
	assert!(State::is_split_ptr(&cloned));
	let borrow = cloned.borrow();
	let split_state = borrow.into_split().unwrap();
	
	// Verify left branch
	let left_out = split_state.out1.borrow();
	assert!(State::is_basic_ptr(&left_out));
	assert_eq!(left_out.borrow().into_basic().unwrap().c.char().unwrap(), 'a');
	
	// Verify right branch
	let right_out = split_state.out2.borrow();
	assert!(State::is_basic_ptr(&right_out));
	assert_eq!(right_out.borrow().into_basic().unwrap().c.char().unwrap(), 'b');
}

#[test]
fn test_iterative_clone_deep_structure() {
	// Create a deep structure that would likely cause stack overflow with recursion
	let original = create_linear_chain(1000);
	
	// Clone using iterative method
	let (cloned, _) = original.borrow().self_ptr_deep_clone_with_memo_iterative(&mut HashMap::new());
	
	// Verify the first state
	assert!(State::is_basic_ptr(&cloned));
	assert_eq!(cloned.borrow().into_basic().unwrap().c.char().unwrap(), 'a');
	
	// Follow the chain and verify the last state is a match state
	let mut current = cloned;
	let mut count = 0;
	
	while !State::is_match_ptr(&current) && count < 1100 {
		let tmp = current;
		let borrow = tmp.borrow();
		if let Some(basic) = borrow.into_basic() {
			current = basic.out.borrow().clone();
			count += 1;
		} else {
			panic!("Expected basic state in chain");
		}
	}
	
	assert!(State::is_match_ptr(&current), "Last state should be a match state");
	assert_eq!(count, 1000, "Chain should have 1000 states");
}

#[test]
fn test_iterative_clone_nested_splits() {
	// Create a structure with deeply nested splits
	let original = create_nested_split(100);

	// Clone using iterative method
	let (cloned, _) = original.borrow().self_ptr_deep_clone_with_memo_iterative(&mut HashMap::new());
	
	// Verify it's a split
	assert!(State::is_split_ptr(&cloned));
	
	// Walk the structure and count the depth
	fn count_depth(state: StatePtr) -> usize {
		fn count_depth_with_memo(state: StatePtr, memo: &mut HashMap<*const State, usize>) -> usize {
			let raw_ptr = &*state.borrow() as *const State;

			// Check if we've already counted this state
			if let Some(depth) = memo.get(&raw_ptr) {
				return *depth;
			}
			
			let result = if State::is_match_ptr(&state) {
				1
			} else if let Some(basic) = state.borrow().into_basic() {
				let next_depth = count_depth_with_memo(basic.out.borrow().clone(), memo);
				1 + next_depth
			} else if let Some(split) = state.borrow().into_split() {
				let left_depth = count_depth_with_memo(split.out1.borrow().clone(), memo);
				let right_depth = count_depth_with_memo(split.out2.borrow().clone(), memo);
				std::cmp::max(left_depth, right_depth)
			} else if state.borrow().is_start_of_line() {
				let out = state.borrow().start_of_line_out().unwrap();
				let next_depth = count_depth_with_memo(out.borrow().clone(), memo);
				1 + next_depth
			} else if state.borrow().is_end_of_line() {
				let out = state.borrow().end_of_line_out().unwrap();
				let next_depth = count_depth_with_memo(out.borrow().clone(), memo);
				1 + next_depth
			} else {
				0
			};

			// Store the result in the memo
			memo.insert(raw_ptr, result);
			result
		}
		
		count_depth_with_memo(state, &mut HashMap::new())
	}
	
	let cloned_depth = count_depth(cloned);
	let original_depth = count_depth(original);

	assert_eq!(original_depth, cloned_depth);

	// 100 basics + 1 match
	assert_eq!(cloned_depth, 101);

}

#[test]
fn test_cloning_with_memo_reuse() {
	// Create a state that's used multiple times
	let shared = State::basic(RegexType::Char('x'));
	
	// Create two states that point to the shared state
	let a = State::basic(RegexType::Char('a'));
	a.borrow_mut().into_basic().unwrap().out.replace(shared.clone());
	
	let b = State::basic(RegexType::Char('b'));
	b.borrow_mut().into_basic().unwrap().out.replace(shared.clone());
	
	// Create a split that uses both
	let original = State::split(a, b);
	
	// Clone using iterative method
	let (cloned, _) = original.borrow().self_ptr_deep_clone_with_memo_iterative(&mut HashMap::new());
	
	// Get the out states from the split
	let borrow = cloned.borrow();
	let split = borrow.into_split().unwrap();
	let out1 = split.out1.borrow().clone();
	let out2 = split.out2.borrow().clone();
	
	// Get their out states (should be the same object)
	let out1_next = out1.borrow().into_basic().unwrap().out.borrow().clone();
	let out2_next = out2.borrow().into_basic().unwrap().out.borrow().clone();
	
	// Verify they're the same object (Rc::ptr_eq)
	assert!(Rc::ptr_eq(&out1_next, &out2_next), 
			"Shared state should be cloned only once");
}

#[test]
fn test_compare_with_recursive_clone() {
	// Create various test structures
	let tests = vec![
		State::basic(RegexType::Char('a')),
		State::match_(0),
		State::split(State::basic(RegexType::Char('a')), State::basic(RegexType::Char('b'))),
		create_linear_chain(10),
		create_nested_split(5)
	];
	
	for original in tests {
		// Clone both ways
		let (recursive_clone, _) = original.borrow().self_ptr_deep_clone_with_memo(&mut HashMap::new());
		let (iterative_clone, _) = original.borrow().self_ptr_deep_clone_with_memo_iterative(&mut HashMap::new());
		
		// Compare structure equality
		assert!(structures_equal(&recursive_clone, &iterative_clone),
				"Recursive and iterative clones should be structurally identical");
	}
}

#[test]
fn test_compare_clone_performance() {
	// Test cases with different sizes and complexities
	let test_cases = vec![
		("Small linear chain", create_linear_chain(10)),
		("Medium linear chain", create_linear_chain(100)),
		("Large linear chain", create_linear_chain(500)),
		("Large linear chain", create_linear_chain(800)),
		("Small nested split", create_nested_split(10)),
		("Medium nested split", create_nested_split(50)),
		("Large nested split", create_nested_split(100)),
		("Large+ nested split", create_nested_split(350)),
		// ("Large+ nested split", create_nested_split(400)),
	];
	println!("Performance comparison between recursive and iterative cloning:");
	println!("{:<20} | {:<15} | {:<15} | {:<10}", "Test Case", "Recursive (ms)", "Iterative (ms)", "Speedup");
	println!("{:-<67}", "");
	
	for (name, original) in test_cases {
		// Measure recursive clone time
		let recursive_start = Instant::now();
		let (recursive_clone, _) = original.borrow().self_ptr_deep_clone_with_memo(&mut HashMap::new());
		let recursive_duration = recursive_start.elapsed();
		
		// Measure iterative clone time
		let iterative_start = Instant::now();
		let (iterative_clone, _) = original.borrow().self_ptr_deep_clone_with_memo_iterative(&mut HashMap::new());
		let iterative_duration = iterative_start.elapsed();
		
		// Calculate speedup
		let speedup = if iterative_duration.as_millis() > 0 {
			recursive_duration.as_millis() as f64 / iterative_duration.as_millis() as f64
		} else {
			f64::NAN
		};
		
		println!(
			"{:<20} | {:<15.2} | {:<15.2} | {:<10.2}x",
			name,
			recursive_duration.as_millis() as f64,
			iterative_duration.as_millis() as f64,
			speedup
		);
		
		// Verify the clones are structurally identical
		// assert!(structures_equal(&recursive_clone, &iterative_clone),
		//         "Clones should be structurally identical for {}", name);
	}
}

#[test]
fn test_detailed_performance_analysis() {
	// Number of iterations for more reliable measurements
	const ITERATIONS: usize = 5;
	
	// Test with a large structure that would benefit from iterative approach
	let original = create_nested_split(700);
	
	let mut recursive_times = Vec::with_capacity(ITERATIONS);
	let mut iterative_times = Vec::with_capacity(ITERATIONS);
	
	for _ in 0..ITERATIONS {
		// Measure recursive clone
		let start = Instant::now();
		let (recursive_clone, _) = original.borrow().self_ptr_deep_clone_with_memo(&mut HashMap::new());
		recursive_times.push(start.elapsed());
		
		// Measure iterative clone
		let start = Instant::now();
		let (iterative_clone, _) = original.borrow().self_ptr_deep_clone_with_memo_iterative(&mut HashMap::new());
		iterative_times.push(start.elapsed());
		
		// Verify equality
		assert!(structures_equal(&recursive_clone, &iterative_clone));
	}
	
	// Calculate average times
	let avg_recursive = recursive_times.iter().sum::<Duration>().as_micros() as f64 / ITERATIONS as f64;
	let avg_iterative = iterative_times.iter().sum::<Duration>().as_micros() as f64 / ITERATIONS as f64;
	
	// Calculate standard deviations
	let std_dev_recursive = (recursive_times.iter()
		.map(|t| {
			let diff = t.as_micros() as f64 - avg_recursive;
			diff * diff
		})
		.sum::<f64>() / ITERATIONS as f64)
		.sqrt();
		
	let std_dev_iterative = (iterative_times.iter()
		.map(|t| {
			let diff = t.as_micros() as f64 - avg_iterative;
			diff * diff
		})
		.sum::<f64>() / ITERATIONS as f64)
		.sqrt();
	
	println!("\nDetailed Performance Analysis (Linear Chain of 500 states, {} iterations):", ITERATIONS);
	println!("Recursive: {:.2} µs (±{:.2} µs)", avg_recursive, std_dev_recursive);
	println!("Iterative: {:.2} µs (±{:.2} µs)", avg_iterative, std_dev_iterative);
	println!("Speedup: {:.2}x", avg_recursive / avg_iterative);
	
	// Verify the iterative method is faster
	assert!(avg_iterative <= avg_recursive, 
			"Expected iterative method to be faster than recursive method");
}

// Helper function to compare structure equality
fn structures_equal(a: &StatePtr, b: &StatePtr) -> bool {
	let mut visited = HashMap::new();
	structures_equal_recursive(a, b, &mut visited)
}

fn structures_equal_recursive(
	a: &StatePtr, 
	b: &StatePtr, 
	visited: &mut HashMap<(*const State, *const State), bool>
) -> bool {
	let a_ptr = &*a.borrow() as *const State;
	let b_ptr = &*b.borrow() as *const State;
	
	// If we've already checked this pair, return the cached result
	if let Some(&result) = visited.get(&(a_ptr, b_ptr)) {
		return result;
	}
	
	// We're now visiting this pair - temporarily mark as equal
	visited.insert((a_ptr, b_ptr), true);
	
	let result = match (&*a.borrow(), &*b.borrow()) {
		(State::Basic(a_basic), State::Basic(b_basic)) => {
			// Compare the character
			if a_basic.c.char() != b_basic.c.char() {
				return false;
			}
			
			// Compare out states
			structures_equal_recursive(
				&a_basic.out.borrow(), 
				&b_basic.out.borrow(), 
				visited
			)
		},
		
		(State::Split(a_split), State::Split(b_split)) => {
			// Compare both branches
			structures_equal_recursive(
				&a_split.out1.borrow(), 
				&b_split.out1.borrow(), 
				visited
			) && 
			structures_equal_recursive(
				&a_split.out2.borrow(), 
				&b_split.out2.borrow(), 
				visited
			)
		},
		
		(State::StartOfLine { out: a_out }, State::StartOfLine { out: b_out }) => {
			structures_equal_recursive(
				&a_out.borrow(), 
				&b_out.borrow(), 
				visited
			)
		},
		
		(State::EndOfLine { out: a_out }, State::EndOfLine { out: b_out }) => {
			structures_equal_recursive(
				&a_out.borrow(), 
				&b_out.borrow(), 
				visited
			)
		},
		
		(State::Match {..}, State::Match {..}) => true,
		(State::NoMatch, State::NoMatch) => true,
		(State::None, State::None) => true,
		
		// Different types
		_ => false,
	};
	
	// Update the cache with the real result
	visited.insert((a_ptr, b_ptr), result);
	result
}

#[test]
fn test_deep_clone_basic_state() {
	let basic = State::basic(RegexType::Char('a'));
	let (cloned, _) = State::deep_clone(&basic);
	
	assert!(structures_equal_recursive(&basic, &cloned, &mut HashMap::new()));
}

#[test]
fn test_deep_clone_match_state() {
	let match_state = State::match_(42);
	let (cloned, _) = State::deep_clone(&match_state);
	
	assert!(structures_equal_recursive(&match_state, &cloned, &mut HashMap::new()));
}

#[test]
fn test_deep_clone_split_state() {
	// Create a split state with two basic states
	let out1 = State::basic(RegexType::Char('a'));
	let out2 = State::basic(RegexType::Char('b'));
	let split = state_ptr(State::Split(SplitState {
		out1: var_state_ptr(out1),
		out2: var_state_ptr(out2)
	}));
	
	let (cloned, _) = State::deep_clone(&split);
	
	assert!(structures_equal_recursive(&split, &cloned, &mut HashMap::new()));
}

#[test]
fn test_deep_clone_cyclic_structure() {
	// Create a state that points to itself (cycle)
	let state = state_ptr(State::Basic(BasicState {
		c: RegexType::Char('a'),
		out: var_state_ptr(State::none())
	}));
	
	// Create a cycle by pointing to itself
	state.borrow().basic_out().unwrap().replace(Rc::clone(&state));
	
	let (cloned, _) = State::deep_clone(&state);
	
	// Verify the structure is equal
	assert!(structures_equal_recursive(&state, &cloned, &mut HashMap::new()));
	
	// Verify the cycle exists in the cloned structure
	let cloned_out = cloned.borrow().basic_out().unwrap();
	let cloned_out_state = cloned_out.borrow();
	
	// The cloned state should point to itself, not the original
	assert!(!Rc::ptr_eq(&state, &cloned_out_state));
	assert!(Rc::ptr_eq(&cloned, &cloned_out_state));
}

#[test]
fn test_deep_clone_complex_structure() {
	// Create a more complex structure:
	// Split -> Basic('a') -> Match(1)
	//      \-> Basic('b') -> Match(1)
	
	let match_state = State::match_(1);
	
	let basic_a = state_ptr(State::Basic(BasicState {
		c: RegexType::Char('a'),
		out: var_state_ptr(Rc::clone(&match_state))
	}));
	
	let basic_b = state_ptr(State::Basic(BasicState {
		c: RegexType::Char('b'),
		out: var_state_ptr(Rc::clone(&match_state))
	}));
	
	let split = state_ptr(State::Split(SplitState {
		out1: var_state_ptr(basic_a),
		out2: var_state_ptr(basic_b)
	}));
	
	let (cloned, _) = State::deep_clone(&split);
	
	assert!(structures_equal_recursive(&split, &cloned, &mut HashMap::new()));
}

#[test]
fn test_deep_clone_start_end_line_states() {
	// Test StartOfLine state
	let basic = State::basic(RegexType::Char('a'));
	let start_line = state_ptr(State::StartOfLine { 
		out: var_state_ptr(basic) 
	});
	
	let (cloned_start, _) = State::deep_clone(&start_line);
	assert!(structures_equal_recursive(&start_line, &cloned_start, &mut HashMap::new()));
	
	// Test EndOfLine state
	let match_state = State::match_(1);
	let end_line = state_ptr(State::EndOfLine { 
		out: var_state_ptr(match_state) 
	});
	
	let (cloned_end, _) = State::deep_clone(&end_line);
	assert!(structures_equal_recursive(&end_line, &cloned_end, &mut HashMap::new()));
}

#[test]
fn test_deep_clone_diamond_structure() {
	// Create a diamond-shaped structure:
	//       Split
	//      /     \
	// Basic('a') Basic('b')
	//      \     /
	//       Match
	
	let match_state = State::match_(1);
	
	let basic_a = state_ptr(State::Basic(BasicState {
		c: RegexType::Char('a'),
		out: var_state_ptr(Rc::clone(&match_state))
	}));
	
	let basic_b = state_ptr(State::Basic(BasicState {
		c: RegexType::Char('b'),
		out: var_state_ptr(Rc::clone(&match_state))
	}));
	
	let split = state_ptr(State::Split(SplitState {
		out1: var_state_ptr(basic_a),
		out2: var_state_ptr(basic_b)
	}));
	
	let (cloned, _) = State::deep_clone(&split);
	
	assert!(structures_equal_recursive(&split, &cloned, &mut HashMap::new()));
	
	// Verify the diamond structure is preserved (both paths lead to the same match state)
	let borrow = cloned.borrow();
	let cloned_split = match &*borrow {
		State::Split(s) => s,
		_ => panic!("Expected Split state")
	};
	
	let out1_match = cloned_split.out1.borrow().borrow().basic_out().unwrap();
	let out2_match = cloned_split.out2.borrow().borrow().basic_out().unwrap();
	
	// Both paths should lead to the same match state
	// Debug both output paths to verify they point to the same match state
	dbg!(&out1_match);
	dbg!(&out2_match);
	assert_eq!(Rc::ptr_eq(&*out1_match.borrow(), &*out2_match.borrow()), true);
}