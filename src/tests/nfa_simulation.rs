use crate::regex::nfa_simulation::*;
use crate::regex::post2nfa::*;
use crate::regex::*;
use std::collections::VecDeque;
use std::iter::Peekable;
use std::rc::Rc;
use std::str::Chars;

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a basic NFA
    fn create_basic_nfa(c: char) -> Nfa {
        let token = TokenType::Literal(RegexType::Char(c));
        post2nfa(VecDeque::from([token])).expect("Failed to build nfa")
    }

    // Helper function to create simple patterns
    fn create_simple_pattern(pattern: &str) -> Nfa {
        let mut tokens = VecDeque::new();

        // Create a token for each character and add concatenation
        let mut chars = pattern.chars();
        if let Some(first) = chars.next() {
            tokens.push_back(TokenType::Literal(RegexType::Char(first)));

            for c in chars {
                tokens.push_back(TokenType::Literal(RegexType::Char(c)));
                tokens.push_back(TokenType::BinaryOperator(RegexType::Concatenation));
            }
        }

        post2nfa(tokens).expect("Failed to build pattern nfa")
    }

    // Helper function to create an alternation NFA (a|b)
    fn create_alt_nfa(a: char, b: char) -> Nfa {
        let tokens = VecDeque::from([
            TokenType::Literal(RegexType::Char(a)),
            TokenType::Literal(RegexType::Char(b)),
            TokenType::BinaryOperator(RegexType::Or),
        ]);
        post2nfa(tokens).expect("Failed to build alternation nfa")
    }

    // Helper function to create a repetition NFA (a*)
    fn create_star_nfa(c: char) -> Nfa {
        let tokens = VecDeque::from([
            TokenType::Literal(RegexType::Char(c)),
            TokenType::UnaryOperator(RegexType::Quant(Quantifier::AtLeast(0))),
        ]);
        post2nfa(tokens).expect("Failed to build star nfa")
    }

    // Helper function to create a plus NFA (a+)
    fn create_plus_nfa(c: char) -> Nfa {
        let tokens = VecDeque::from([
            TokenType::Literal(RegexType::Char(c)),
            TokenType::UnaryOperator(RegexType::Quant(Quantifier::AtLeast(1))),
        ]);
        post2nfa(tokens).expect("Failed to build plus nfa")
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

        let mut current_list = List::from(&nfa.start);
        let mut next_list = List::new();

        // Test step with matching character
        let mut chars = "a".chars().peekable();
        let start_of_line = true;
        step(
            &mut chars,
            &current_list,
            &mut next_list,
            &nfa,
            start_of_line,
        );

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
        let mut match_nfa = Nfa::new();
        match_nfa.start = State::match_();
        assert!(input_match(&match_nfa, ""));
        assert!(input_match(&match_nfa, "anything"));
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

    #[test]
    fn test_alternation_matching() {
        // Test (a|b) pattern
        let nfa = create_alt_nfa('a', 'b');

        assert!(input_match(&nfa, "a"), "Should match 'a'");
        assert!(input_match(&nfa, "b"), "Should match 'b'");
        assert!(!input_match(&nfa, "c"), "Should not match 'c'");
        assert!(!input_match(&nfa, "ab"), "Should not match 'ab'");
        assert!(!input_match(&nfa, ""), "Should not match empty string");

        // Test more complex alternation with concatenation
        let tokens = VecDeque::from([
            TokenType::Literal(RegexType::Char('a')),
            TokenType::Literal(RegexType::Char('b')),
            TokenType::BinaryOperator(RegexType::Concatenation),
            TokenType::Literal(RegexType::Char('c')),
            TokenType::Literal(RegexType::Char('d')),
            TokenType::BinaryOperator(RegexType::Concatenation),
            TokenType::BinaryOperator(RegexType::Or),
        ]);
        let nfa = post2nfa(tokens).expect("Failed to build complex alternation nfa");

        assert!(input_match(&nfa, "ab"), "Should match 'ab'");
        assert!(input_match(&nfa, "cd"), "Should match 'cd'");
        assert!(!input_match(&nfa, "ac"), "Should not match 'ac'");
        assert!(!input_match(&nfa, "abcd"), "Should not match 'abcd'");
    }

    #[test]
    fn test_repetition_matching() {
        // Test a* (0 or more 'a's)
        let nfa = create_star_nfa('a');

        assert!(input_match(&nfa, ""), "a* should match empty string");
        assert!(input_match(&nfa, "a"), "a* should match 'a'");
        assert!(input_match(&nfa, "aa"), "a* should match 'aa'");
        assert!(input_match(&nfa, "aaa"), "a* should match 'aaa'");
        assert!(!input_match(&nfa, "b"), "a* should not match 'b'");
        assert!(!input_match(&nfa, "ab"), "a* should not match 'ab'");

        // Test a+ (1 or more 'a's)
        let nfa = create_plus_nfa('a');

        assert!(!input_match(&nfa, ""), "a+ should not match empty string");
        assert!(input_match(&nfa, "a"), "a+ should match 'a'");
        assert!(input_match(&nfa, "aa"), "a+ should match 'aa'");
        assert!(!input_match(&nfa, "b"), "a+ should not match 'b'");
    }

    #[test]
    fn test_empty_pattern() {
        // Create an empty pattern (just a match state)
        let mut match_nfa = Nfa::new();
        match_nfa.start = State::match_();

        assert!(
            input_match(&match_nfa, ""),
            "Empty pattern should match empty string"
        );
        assert!(
            input_match(&match_nfa, "anything"),
            "Empty pattern should match any string"
        );
    }

    #[test]
    fn test_complex_patterns() {
        // Test (a|b)* pattern
        let tokens = VecDeque::from([
            TokenType::Literal(RegexType::Char('a')),
            TokenType::Literal(RegexType::Char('b')),
            TokenType::BinaryOperator(RegexType::Or),
            TokenType::UnaryOperator(RegexType::Quant(Quantifier::AtLeast(0))),
        ]);
        let nfa = post2nfa(tokens).expect("Failed to build (a|b)* nfa");

        assert!(input_match(&nfa, ""), "(a|b)* should match empty string");
        assert!(input_match(&nfa, "a"), "(a|b)* should match 'a'");
        assert!(input_match(&nfa, "b"), "(a|b)* should match 'b'");
        assert!(input_match(&nfa, "ab"), "(a|b)* should match 'ab'");
        assert!(input_match(&nfa, "aba"), "(a|b)* should match 'aba'");
        assert!(
            input_match(&nfa, "abababba"),
            "(a|b)* should match 'abababba'"
        );
        assert!(!input_match(&nfa, "abc"), "(a|b)* should not match 'abc'");

        // Test a(bc)+ pattern
        let tokens = VecDeque::from([
            TokenType::Literal(RegexType::Char('a')),
            TokenType::Literal(RegexType::Char('b')),
            TokenType::Literal(RegexType::Char('c')),
            TokenType::BinaryOperator(RegexType::Concatenation),
            TokenType::UnaryOperator(RegexType::Quant(Quantifier::AtLeast(1))),
            TokenType::BinaryOperator(RegexType::Concatenation),
        ]);

        let nfa = post2nfa(tokens).expect("Failed to build a(bc)+ nfa");

        assert!(input_match(&nfa, "abc"), "a(bc)+ should match 'abc'");
        assert!(input_match(&nfa, "abcbc"), "a(bc)+ should match 'abcbc'");
        assert!(!input_match(&nfa, "a"), "a(bc)+ should not match 'a'");
        assert!(!input_match(&nfa, "ab"), "a(bc)+ should not match 'ab'");
    }

    #[test]
    fn test_edge_cases() {
        // Test none state
        let mut none_nfa = Nfa::new();
        none_nfa.start = State::none();
        assert!(
            !input_match(&none_nfa, ""),
            "None state should not match empty string"
        );
        assert!(
            !input_match(&none_nfa, "a"),
            "None state should not match any string"
        );

        // Test no_match state
        let mut no_match_nfa = Nfa::new();
        no_match_nfa.start = State::no_match();
        assert!(
            !input_match(&no_match_nfa, ""),
            "No match state should not match empty string"
        );
        assert!(
            !input_match(&no_match_nfa, "a"),
            "No match state should not match any string"
        );

        // Test with very long input
        let nfa = create_star_nfa('a');
        let long_string = "a".repeat(1000);
        assert!(input_match(&nfa, &long_string), "a* should match 1000 'a's");
    }

    #[test]
    fn test_step_function_edge_cases() {
        // Test step with empty input
        let nfa = create_basic_nfa('a');
        let mut current_list = List::from(&nfa.start);
        let mut next_list = List::new();
        let mut chars = "".chars().peekable();
        let start_of_line = true;

        step(
            &mut chars,
            &current_list,
            &mut next_list,
            &nfa,
            start_of_line,
        );
        assert!(
            next_list.states.is_empty(),
            "Step with empty input should result in empty next list"
        );

        // Test step with non-matching character
        let mut current_list = List::from(&nfa.start);
        let mut next_list = List::new();
        let mut chars = "b".chars().peekable();

        step(
            &mut chars,
            &current_list,
            &mut next_list,
            &nfa,
            start_of_line,
        );
        assert!(
            next_list.states.is_empty(),
            "Step with non-matching character should result in empty next list"
        );
    }
}
