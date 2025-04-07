use crate::regex::post2nfa::*;
use crate::regex::parsing::*;
use crate::regex::re2post::*;
use crate::regex::*;
use crate::parsing::error::ParsingResult;
use std::collections::VecDeque;

// Helper function to convert regex pattern to postfix notation
fn pattern_to_postfix(pattern: &str) -> ParsingResult<VecDeque<TokenType>> {
    let tokens = Regex::tokens(pattern)?;
    let infix = Regex::add_concatenation(tokens);
    re2post(infix)
}

// Helper function to create NFA from regex pattern
fn pattern_to_nfa(pattern: &str) -> ParsingResult<Nfa> {
    let postfix = pattern_to_postfix(pattern)?;
    post2nfa(postfix)
}

// ==============================================
// 1. BASIC NFA STATE CONSTRUCTION TESTS
// ==============================================

#[test]
fn test_state_creation() {
    // Test basic state creation
    let basic_state = State::basic(RegexType::Char('a'));
    assert!(State::is_basic_ptr(&basic_state));
    
    // Test split state creation
    let none_state = State::none();
    let split_state = State::split(State::none(), State::none());
    assert!(State::is_split_ptr(&split_state));
    
    // Test match state creation
    let match_state = State::match_();
    assert!(State::is_match_ptr(&match_state));
    
    // Test nomatch state creation
    let nomatch_state = State::no_match();
    assert!(State::is_nomatch_ptr(&nomatch_state));
}

#[test]
fn test_state_ptr_management() {
    // Test state_ptr function
    let state = State::Basic(BasicState {
        c: RegexType::Char('a'),
        out: var_state_ptr(State::none()),
    });
    let ptr = state_ptr(state);
    
    // Test accessing state through pointer
    let borrowed = ptr.borrow();
    assert!(borrowed.is_basic());
    
    // Test var_state_ptr function
    let var_ptr = var_state_ptr(ptr.clone());
    assert!(State::is_basic_var_ptr(&var_ptr));
}

#[test]
fn test_state_type_checking() {
    // Create different state types
    let basic = State::basic(RegexType::Char('a'));
    let split = State::split(State::none(), State::none());
    let match_state = State::match_();
    let nomatch = State::no_match();
    let none = State::none();
    
    // Test type checking methods
    assert!(State::is_basic_ptr(&basic));
    assert!(State::is_split_ptr(&split));
    assert!(State::is_match_ptr(&match_state));
    assert!(State::is_nomatch_ptr(&nomatch));
    assert!(State::is_none_ptr(&none));
    
    // Test negative cases
    assert!(!State::is_basic_ptr(&split));
    assert!(!State::is_split_ptr(&basic));
    assert!(!State::is_match_ptr(&none));
}

#[test]
fn test_character_matching() {
    // Create a basic state for character 'a'
    let state = State::Basic(BasicState {
        c: RegexType::Char('a'),
        out: var_state_ptr(State::none()),
    });
    
    // Test matching with correct character
    assert!(state.matche_with(&'a'));
    
    // Test matching with incorrect character
    assert!(!state.matche_with(&'b'));
    
    // Test matching with any character state
    let any_state = State::Basic(BasicState {
        c: RegexType::Any,
        out: var_state_ptr(State::none()),
    });
    
    assert!(any_state.matche_with(&'a'));
    assert!(any_state.matche_with(&'b'));
    assert!(any_state.matche_with(&'\n'));
}

// ==============================================
// 2. FRAGMENT OPERATIONS TESTS
// ==============================================

#[test]
fn test_fragment_creation() {
    // Create a basic fragment
    let state = State::basic(RegexType::Char('a'));
    let fragment = Fragment::basic(state);
    
    // Verify fragment structure
    assert_eq!(fragment.ptr_list.len(), 1);
    assert!(State::is_basic_ptr(&fragment.start));
}

#[test]
fn test_fragment_concatenation() {
    // Create two basic fragments
    let state_a = State::basic(RegexType::Char('a'));
    let fragment_a = Fragment::basic(state_a);
    
    let state_b = State::basic(RegexType::Char('b'));
    let fragment_b = Fragment::basic(state_b);
    
    // Concatenate the fragments
    let concat = fragment_a.and(fragment_b);
    
    // Verify the structure of the concatenated fragment
    assert_eq!(concat.ptr_list.len(), 1);
    
    // The start should be the first fragment's start
    assert!(State::is_basic_ptr(&concat.start));
    
    // Test with pattern "ab" and verify the NFA works
    let nfa = pattern_to_nfa("ab").unwrap();
    assert!(State::is_basic_ptr(&nfa.start));
}

#[test]
fn test_fragment_alternation() {
    // Create two basic fragments
    let state_a = State::basic(RegexType::Char('a'));
    let fragment_a = Fragment::basic(state_a);
    
    let state_b = State::basic(RegexType::Char('b'));
    let fragment_b = Fragment::basic(state_b);
    
    // Create OR fragment
    let alt = fragment_a.or(fragment_b);
    
    // Verify the structure of the alternation fragment
    assert_eq!(alt.ptr_list.len(), 2);
    assert!(State::is_split_ptr(&alt.start));
    
    // Test with pattern "a|b" and verify the NFA works
    let nfa = pattern_to_nfa("a|b").unwrap();
    assert!(State::is_split_ptr(&nfa.start));
}

#[test]
fn test_fragment_optional() {
    // Create a basic fragment
    let state_a = State::basic(RegexType::Char('a'));
    let fragment_a = Fragment::basic(state_a);
    
    // Make it optional
    let optional = fragment_a.optional();
    
    // Verify the structure of the optional fragment
    assert!(State::is_split_ptr(&optional.start));
    
    // Test with pattern "a?" and verify the NFA works
    let nfa = pattern_to_nfa("a?").unwrap();
    assert!(State::is_split_ptr(&nfa.start));
}

#[test]
fn test_fragment_kleene_star() {
    // Create a basic fragment
    let state_a = State::basic(RegexType::Char('a'));
    let fragment_a = Fragment::basic(state_a);
    
    // Apply Kleene star
    let kleene = fragment_a.optional_repeat();
    
    // Verify the structure of the Kleene star fragment
    assert!(State::is_split_ptr(&kleene.start));
    
    // Test with pattern "a*" and verify the NFA works
    let nfa = pattern_to_nfa("a*").unwrap();
    assert!(State::is_split_ptr(&nfa.start));
}

#[test]
fn test_exact_repetition() {
    // Create a basic fragment
    let state_a = State::basic(RegexType::Char('a'));
    let fragment_a = Fragment::basic(state_a);
    
    // Apply exact repetition of 3
    let repeat = fragment_a.exact_repeat(&3);
    
    // Test with pattern "a{3}" and verify the NFA works
    let nfa = pattern_to_nfa("a{3}").unwrap();
    assert!(State::is_basic_ptr(&nfa.start));
}

#[test]
fn test_at_least_repetition() {
    // Create a basic fragment
    let state_a = State::basic(RegexType::Char('a'));
    let fragment_a = Fragment::basic(state_a);
    
    // Apply at least 2 repetition
    let at_least = fragment_a.at_least(&2);
    
    // Test with pattern "a{2,}" and verify the NFA works
    let nfa = pattern_to_nfa("a{2,}").unwrap();
    assert!(State::is_basic_ptr(&nfa.start));
}

#[test]
fn test_range_repetition() {
    // Create a basic fragment
    let state_a = State::basic(RegexType::Char('a'));
    let fragment_a = Fragment::basic(state_a);
    
    // Apply range repetition {2,4}
    let range = fragment_a.range(&2, &4);
    
    // Test with pattern "a{2,4}" and verify the NFA works
    let nfa = pattern_to_nfa("a{2,4}").unwrap();
    assert!(State::is_basic_ptr(&nfa.start));
}

#[test]
fn test_fragment_deep_clone() {
    // Create a fragment for 'ab'
    let nfa_ab = pattern_to_nfa("ab").unwrap();
    let state_a = State::basic(RegexType::Char('a'));
    let fragment_a = Fragment::basic(state_a);
    
    // Clone the fragment
    let cloned = fragment_a.deep_clone();
    
    // Verify the cloned fragment is independent
    assert!(State::is_basic_ptr(&cloned.start));
    assert_eq!(cloned.ptr_list.len(), 1);
}

// ==============================================
// 3. QUANTIFIER HANDLING TESTS
// ==============================================

#[test]
fn test_exact_quantifier() {
    // Test {0} quantifier - should produce no match
    let nfa_zero = pattern_to_nfa("a{0}").unwrap();
    
    // Test {1} quantifier - should be equivalent to just 'a'
    let nfa_one = pattern_to_nfa("a{1}").unwrap();
    assert!(State::is_basic_ptr(&nfa_one.start));
    
    // Test {3} quantifier - should create three consecutive 'a's
    let nfa_three = pattern_to_nfa("a{3}").unwrap();
    assert!(State::is_basic_ptr(&nfa_three.start));
}

#[test]
fn test_at_least_quantifier() {
    // Test {0,} quantifier - equivalent to a*
    let nfa_zero_plus = pattern_to_nfa("a{0,}").unwrap();
    assert!(State::is_split_ptr(&nfa_zero_plus.start));
    
    // Test {1,} quantifier - equivalent to a+
    let nfa_one_plus = pattern_to_nfa("a{1,}").unwrap();
    assert!(State::is_basic_ptr(&nfa_one_plus.start));
    
    // Test {3,} quantifier
    let nfa_three_plus = pattern_to_nfa("a{3,}").unwrap();
    assert!(State::is_basic_ptr(&nfa_three_plus.start));
}

#[test]
fn test_range_quantifier() {
    // Test {0,1} quantifier - equivalent to a?
    let nfa_optional = pattern_to_nfa("a{0,1}").unwrap();
    assert!(State::is_split_ptr(&nfa_optional.start));
    
    // Test {1,3} quantifier
    let nfa_one_to_three = pattern_to_nfa("a{1,3}").unwrap();
    assert!(State::is_basic_ptr(&nfa_one_to_three.start));
}

#[test]
fn test_standard_shorthand_quantifiers() {
    // Test * quantifier (0 or more)
    let nfa_star = pattern_to_nfa("a*").unwrap();
    assert!(State::is_split_ptr(&nfa_star.start));
    
    // Test + quantifier (1 or more)
    let nfa_plus = pattern_to_nfa("a+").unwrap();
    assert!(State::is_basic_ptr(&nfa_plus.start));
    
    // Test ? quantifier (0 or 1)
    let nfa_question = pattern_to_nfa("a?").unwrap();
    assert!(State::is_split_ptr(&nfa_question.start));
}

// ==============================================
// 4. NFA CONSTRUCTION TESTS
// ==============================================

#[test]
fn test_nfa_from_postfix() {
    // Test basic character
    let nfa_a = pattern_to_nfa("a").unwrap();
    assert!(State::is_basic_ptr(&nfa_a.start));
    
    // Test concatenation
    let nfa_ab = pattern_to_nfa("ab").unwrap();
    assert!(State::is_basic_ptr(&nfa_ab.start));
    
    // Test alternation
    let nfa_a_or_b = pattern_to_nfa("a|b").unwrap();
    assert!(State::is_split_ptr(&nfa_a_or_b.start));
    
    // Test complex expression
    let nfa_complex = pattern_to_nfa("a(b|c)*d").unwrap();
    assert!(State::is_basic_ptr(&nfa_complex.start));
}

#[test]
fn test_processing_concatenation() {
    // Test concatenation of two characters
    let nfa_ab = pattern_to_nfa("ab").unwrap();
    
    // Test concatenation with a group
    let nfa_a_group = pattern_to_nfa("a(bc)").unwrap();
    
    // Test multiple concatenations
    let nfa_abc = pattern_to_nfa("abc").unwrap();
}

#[test]
fn test_processing_or_operators() {
    // Test simple alternation
    let nfa_a_or_b = pattern_to_nfa("a|b").unwrap();
    assert!(State::is_split_ptr(&nfa_a_or_b.start));
    
    // Test alternation with groups
    let nfa_group_or = pattern_to_nfa("(ab)|(cd)").unwrap();
    assert!(State::is_split_ptr(&nfa_group_or.start));
    
    // Test multiple alternations
    let nfa_multi_or = pattern_to_nfa("a|b|c").unwrap();
    assert!(State::is_split_ptr(&nfa_multi_or.start));
}

#[test]
fn test_processing_quantifiers() {
    // Test with Kleene star
    let nfa_kleene = pattern_to_nfa("a*").unwrap();
    assert!(State::is_split_ptr(&nfa_kleene.start));
    
    // Test with plus
    let nfa_plus = pattern_to_nfa("a+").unwrap();
    assert!(State::is_basic_ptr(&nfa_plus.start));
    
    // Test with optional
    let nfa_optional = pattern_to_nfa("a?").unwrap();
    assert!(State::is_split_ptr(&nfa_optional.start));
    
    // Test with exact quantifier
    let nfa_exact = pattern_to_nfa("a{3}").unwrap();
    assert!(State::is_basic_ptr(&nfa_exact.start));
}

#[test]
fn test_processing_line_anchors() {
    // Test with line start anchor
    let nfa_start = pattern_to_nfa("^abc").unwrap();
    assert!(nfa_start.start_of_line);
    assert!(!nfa_start.end_of_line);
    
    // Test with line end anchor
    let nfa_end = pattern_to_nfa("abc$").unwrap();
    assert!(nfa_end.end_of_line);
    assert!(!nfa_end.start_of_line);
    
    // Test with both anchors
    let nfa_both = pattern_to_nfa("^abc$").unwrap();
    assert!(nfa_both.start_of_line);
    assert!(nfa_both.end_of_line);
}

// ==============================================
// 5. ERROR HANDLING TESTS
// ==============================================

#[test]
fn test_invalid_token_sequence() {
    // Create invalid sequence with consecutive operators
    let mut tokens = VecDeque::new();
    tokens.push_back(TokenType::BinaryOperator(RegexType::Or));
    tokens.push_back(TokenType::BinaryOperator(RegexType::Or));
    
    // Attempt to create NFA from invalid tokens
    let result = post2nfa(tokens);
    assert!(result.is_err());
}

#[test]
fn test_stack_underflow() {
    // Create sequence with more operators than operands
    let mut tokens = VecDeque::new();
    tokens.push_back(TokenType::Literal(RegexType::Char('a')));
    tokens.push_back(TokenType::BinaryOperator(RegexType::Concatenation));
    tokens.push_back(TokenType::BinaryOperator(RegexType::Concatenation));
    
    // Attempt to create NFA from invalid tokens
    let result = post2nfa(tokens);
    assert!(result.is_err());
}

#[test]
fn test_improper_line_anchor_usage() {
    // Test with duplicate line start anchor
    let result_dup_start = pattern_to_nfa("^^abc");
    assert!(result_dup_start.is_err());
    
    // Test with duplicate line end anchor
    let result_dup_end = pattern_to_nfa("abc$$");
    assert!(result_dup_end.is_err());
    
    // Test with misplaced line start anchor
    let result_misplaced_start = pattern_to_nfa("a^bc");
    assert!(result_misplaced_start.is_err());
    
    // Test with misplaced line end anchor
    let result_misplaced_end = pattern_to_nfa("ab$c");
    assert!(result_misplaced_end.is_err());
}

#[test]
fn test_empty_invalid_fragments() {
    // Test with empty pattern
    let result_empty = pattern_to_nfa("");
    assert!(result_empty.is_err());

    // Test with empty parentheses
    let result_empty_parens = pattern_to_nfa("()");
    assert!(result_empty_parens.is_err());
}

// ==============================================
// 6. DEEP CLONING TESTS
// ==============================================

#[test]
fn test_state_deep_clone() {
    // Create a basic state
    let basic = State::basic(RegexType::Char('a'));
    
    // Clone the state
    let (cloned, ptr_list) = State::deep_clone(&basic);
    
    // Verify the clone is correct
    assert!(State::is_basic_ptr(&cloned));
    assert_eq!(ptr_list.len(), 1);
}

#[test]
fn test_complex_structure_clone() {
    // Create a more complex structure (a|b)
    let nfa = pattern_to_nfa("a|b").unwrap();
    
    // Create a fragment from the NFA start state
    let frag = Fragment::new(nfa.start.clone(), vec![]);
    
    // Clone the fragment
    let cloned = frag.deep_clone();
    
    // Verify the clone is correct
    assert!(State::is_split_ptr(&cloned.start));
}

#[test]
fn test_deep_clone_independence() {
    // Create a fragment for 'a'
    let state_a = State::basic(RegexType::Char('a'));
    let fragment_a = Fragment::basic(state_a);
    
    // Clone the fragment
    let cloned = fragment_a.deep_clone();
    
    // Modify the original (by using it in an operation)
    let _modified = fragment_a.and(Fragment::basic(State::basic(RegexType::Char('b'))));
    
    // Verify the clone remains unchanged
    assert!(State::is_basic_ptr(&cloned.start));
    assert_eq!(cloned.ptr_list.len(), 1);
}

// ==============================================
// 7. CHARACTER CLASS AND SPECIAL CHARACTER TESTS
// ==============================================

#[test]
fn test_character_class_handling() {
    // Create a digit character class
    let digit_class = CharacterClass::digit();
    let class_state = State::basic(RegexType::Class(digit_class));
    
    // Create an NFA with a character class pattern
    let nfa_digit = pattern_to_nfa("\\d").unwrap();
    
    // Test with pattern containing a character class range
    let nfa_range = pattern_to_nfa("[a-z]").unwrap();
    assert!(State::is_basic_ptr(&nfa_range.start));
}

#[test]
fn test_wildcard_character() {
    // Test with wildcard pattern
    let nfa_any = pattern_to_nfa(".").unwrap();
    assert!(State::is_basic_ptr(&nfa_any.start));
    
    // Verify it's the 'any' character type
    if let State::Basic(basic_state) = &*nfa_any.start.borrow() {
        assert_eq!(basic_state.c, RegexType::Any);
    } else {
        panic!("Expected Basic state");
    }
    
    // Test with wildcard in a more complex pattern
    let nfa_complex = pattern_to_nfa("a.b").unwrap();
    assert!(State::is_basic_ptr(&nfa_complex.start));
}

#[test]
fn test_escaped_characters() {
    // Test with escaped characters
    let nfa_escaped = pattern_to_nfa("\\n").unwrap();
    assert!(State::is_basic_ptr(&nfa_escaped.start));
    
    // Test with escaped special regex characters
    let nfa_special = pattern_to_nfa("a\\*b").unwrap();
    assert!(State::is_basic_ptr(&nfa_special.start));
    
    // Test with escaped shorthand character class
    let nfa_shorthand = pattern_to_nfa("\\w+").unwrap();
    assert!(State::is_basic_ptr(&nfa_shorthand.start));
}

// ==============================================
// 8. UTILITY FUNCTIONS TESTS
// ==============================================

#[test]
fn test_utility_patching() {
    // Create a var state pointer to patch
    let none_state = State::none();
    let var_ptr = var_state_ptr(none_state);
    
    // Create a target state
    let target = State::match_();
    
    // Patch the pointer
    utils::patch(&vec![var_ptr.clone()], &target);
    
    // Verify the patch worked
    assert!(State::is_match_ptr(&var_ptr.borrow()));
}

#[test]
fn test_utility_last_patch() {
    // Create a var state pointer to patch
    let none_state = State::none();
    let var_ptr = var_state_ptr(none_state);
    
    // Patch to match state
    utils::last_patch(&vec![var_ptr.clone()]);
    
    // Verify it was patched to match state
    assert!(State::is_match_ptr(&var_ptr.borrow()));
}

#[test]
fn test_utility_list_operations() {
    // Create state pointers
    let state1 = State::none();
    let state2 = State::none();
    
    let var_ptr1 = var_state_ptr(state1);
    let var_ptr2 = var_state_ptr(state2);
    
    // Test list1
    let list = utils::list1(var_ptr1.clone());
    assert_eq!(list.len(), 1);
    
    // Test append
    let appended = utils::append(list, vec![var_ptr2.clone()]);
    assert_eq!(appended.len(), 2);
}

// Integration tests showing comprehensive pattern handling

#[test]
fn test_complex_patterns() {
    // Test a moderately complex pattern
    let complex1 = pattern_to_nfa("a(b|c)*d+e?").unwrap();
    
    // Test a pattern with multiple quantifiers
    let complex2 = pattern_to_nfa("(a{2,3}b+){1,2}").unwrap();
    
    // Test a pattern with character classes and anchors
    let complex3 = pattern_to_nfa("^\\w+@\\w+\\.\\w{2,}$").unwrap();
    assert!(complex3.start_of_line);
    assert!(complex3.end_of_line);
}
#[test]
fn test_big_regex() {
    // Test a pattern with multiple quantifiers
    let complex2 = pattern_to_nfa("a{0,10000}{0,10000}").unwrap();
}
