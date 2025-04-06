use crate::regex::post2nfa::*;
use crate::regex::*;
use std::collections::VecDeque;

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create tokens for testing
    fn create_token_queue(tokens: Vec<RegexType>) -> VecDeque<TokenType> {
        let mut queue = VecDeque::new();
        for token in tokens {
            queue.push_back(TokenType::from(token));
        }
        queue
    }

    #[test]
    fn test_state_creation() {
        // Test basic state creation
        let basic = State::basic(RegexType::Char('a'));
        assert!(State::is_basic_ptr(&basic));

        // Test split state creation
        let s1 = State::none();
        let s2 = State::none();
        let split = State::split(s1, s2);
        assert!(State::is_split_ptr(&split));

        // Test match, no_match, and none states
        let match_state = State::match_();
        assert!(State::is_match_ptr(&match_state));

        let no_match = State::no_match();
        assert!(State::is_nomatch_ptr(&no_match));

        let none = State::none();
        assert!(State::is_none_ptr(&none));
    }

    #[test]
    fn test_fragment_operations() {
        // Test fragment creation
        let s = State::basic(RegexType::Char('a'));
        let frag = Fragment::basic(s);

        // Test deep clone
        let cloned = frag.deep_clone();

        // Test concatenation (and)
        let s2 = State::basic(RegexType::Char('b'));
        let frag2 = Fragment::basic(s2);
        let concat = frag.and(frag2);

        // We can't easily check the structure, but we can verify it doesn't panic
    }

    #[test]
    fn test_fragment_quantifiers() {
        // Test optional fragment
        let s = State::basic(RegexType::Char('a'));
        let frag = Fragment::basic(s);
        let optional = frag.deep_clone().optional();

        // Test repeat
        let repeat = frag.deep_clone().optional_repeat();

        // Test exact repeat
        let exact = frag.deep_clone().exact_repeat(&2);

        // Test at least
        let at_least = frag.deep_clone().at_least(&2);

        // Test range
        let range = frag.deep_clone().range(&2, &4);
    }

    #[test]
    fn test_post2nfa_simple() {
        // Test simple character
        let tokens = create_token_queue(vec![RegexType::Char('a')]);
        let nfa = post2nfa(tokens).unwrap();
        assert!(State::is_basic_ptr(&nfa.start));

        // Test concatenation
        let tokens = create_token_queue(vec![
            RegexType::Char('a'),
            RegexType::Char('b'),
            RegexType::Concatenation,
        ]);
        let nfa = post2nfa(tokens);
        assert!(nfa.is_ok());

        // Test alternation (or)
        let tokens = create_token_queue(vec![
            RegexType::Char('a'),
            RegexType::Char('b'),
            RegexType::Or,
        ]);
        let nfa = post2nfa(tokens);
        assert!(nfa.is_ok());
    }

    #[test]
    fn test_post2nfa_complex() {
        // Test (a|b)*
        let tokens = create_token_queue(vec![
            RegexType::Char('a'),
            RegexType::Char('b'),
            RegexType::Or,
        ]);
        let nfa = post2nfa(tokens);
        assert!(nfa.is_ok());

        // Test a{2,3}
        let tokens = create_token_queue(vec![
            RegexType::Char('a'),
            RegexType::Quant(Quantifier::Range(2, 3)),
        ]);
        let nfa = post2nfa(tokens);
        assert!(nfa.is_ok());
    }

    #[test]
    fn test_post2nfa_errors() {
        // Test empty input
        let tokens = VecDeque::new();
        let nfa = post2nfa(tokens);
        assert!(nfa.is_err());

        // Test invalid expression (operator with no operands)
        let tokens = create_token_queue(vec![RegexType::Concatenation]);
        let nfa = post2nfa(tokens);
        assert!(nfa.is_err());
    }

    #[test]
    fn test_post2nfa_character_classes() {
        // Test character class [abc]
        let mut char_class = CharacterClass::new();
        char_class.add_char('a');
        char_class.add_char('b');
        char_class.add_char('c');

        let tokens = create_token_queue(vec![RegexType::Class(char_class)]);
        let nfa = post2nfa(tokens);
        assert!(nfa.is_ok());

        // Test negated character class [^abc]
        let mut char_class = CharacterClass::new();
        char_class.add_char('a');
        char_class.add_char('b');
        char_class.add_char('c');
        let char_class = char_class.negated();

        let tokens = create_token_queue(vec![RegexType::Class(char_class)]);
        let nfa = post2nfa(tokens);
        assert!(nfa.is_ok());

        // Test character range [a-z]
        let mut char_class = CharacterClass::new();
        char_class.add_range('a', 'z');

        let tokens = create_token_queue(vec![RegexType::Class(char_class)]);
        let nfa = post2nfa(tokens);
        assert!(nfa.is_ok());
    }

    #[test]
    fn test_post2nfa_dot() {
        // Test dot (any character)
        let tokens = create_token_queue(vec![RegexType::Any]);
        let nfa = post2nfa(tokens);
        assert!(nfa.is_ok());
    }

    #[test]
    fn test_post2nfa_quantifier_edge_cases() {
        // Test a{0,0} (matches empty string)
        let tokens = create_token_queue(vec![
            RegexType::Char('a'),
            RegexType::Quant(Quantifier::Range(0, 0)),
        ]);
        let nfa = post2nfa(tokens);
        assert!(nfa.is_ok());

        // Test a{0,1} (equivalent to a?)
        let tokens = create_token_queue(vec![
            RegexType::Char('a'),
            RegexType::Quant(Quantifier::Range(0, 1)),
        ]);
        let nfa = post2nfa(tokens);
        assert!(nfa.is_ok());

        // Test a{1,1} (equivalent to just 'a')
        let tokens = create_token_queue(vec![
            RegexType::Char('a'),
            RegexType::Quant(Quantifier::Exact(1)),
        ]);
        let nfa = post2nfa(tokens);
        assert!(nfa.is_ok());
    }

    #[test]
    fn test_post2nfa_complex_nested() {
        // Test (a(b|c))+
        let tokens = create_token_queue(vec![
            RegexType::Char('a'),
            RegexType::Char('b'),
            RegexType::Char('c'),
            RegexType::Or,
            RegexType::Concatenation,
            RegexType::Quant(Quantifier::AtLeast(1)),
        ]);
        let nfa = post2nfa(tokens);
        assert!(nfa.is_ok());

        // Test a|(b|c)
        let tokens = create_token_queue(vec![
            RegexType::Char('a'),
            RegexType::Char('b'),
            RegexType::Char('c'),
            RegexType::Or,
            RegexType::Or,
        ]);
        let nfa = post2nfa(tokens);
        assert!(nfa.is_ok());
    }

    #[test]
    fn test_post2nfa_more_errors() {
        // Test more illegal expressions

        // Binary operator with only one operand
        let tokens = create_token_queue(vec![RegexType::Char('a'), RegexType::Or]);
        let nfa = post2nfa(tokens);
        assert!(nfa.is_err());

        // Unary operator with no operand
        let tokens = create_token_queue(vec![RegexType::Quant(Quantifier::Range(0, 1))]);
        let nfa = post2nfa(tokens);
        assert!(nfa.is_err());

        // Nested illegal expression
        let tokens = create_token_queue(vec![
            RegexType::Char('a'),
            RegexType::Or,
            RegexType::Concatenation,
        ]);
        let nfa = post2nfa(tokens);
        assert!(nfa.is_err());
    }

    #[test]
    fn test_fragment_operations_advanced() {
        // Create two fragments
        let s1 = State::basic(RegexType::Char('a'));
        let frag1 = Fragment::basic(s1);

        let s2 = State::basic(RegexType::Char('b'));
        let frag2 = Fragment::basic(s2);

        // Test or operation
        let or_frag = frag1.deep_clone().or(frag2.deep_clone());

        // Test and operation with or
        let s3 = State::basic(RegexType::Char('c'));
        let frag3 = Fragment::basic(s3);

        let combined = frag3.and(or_frag);
    }

    #[test]
    fn test_quantifiers_direct_post2nfa() {
        // Test a*
        let tokens = create_token_queue(vec![
            RegexType::Char('a'),
            RegexType::Quant(Quantifier::AtLeast(0)),
        ]);
        let nfa = post2nfa(tokens);
        assert!(nfa.is_ok());

        // Test a+
        let tokens = create_token_queue(vec![
            RegexType::Char('a'),
            RegexType::Quant(Quantifier::AtLeast(1)),
        ]);
        let nfa = post2nfa(tokens);
        assert!(nfa.is_ok());

        // Test a?
        let tokens = create_token_queue(vec![
            RegexType::Char('a'),
            RegexType::Quant(Quantifier::Range(0, 1)),
        ]);
        let nfa = post2nfa(tokens);
        assert!(nfa.is_ok());
    }

    #[test]
    fn test_empty_pattern_handling() {
        // Test handling of an empty pattern through quantifiers
        let tokens = create_token_queue(vec![
            RegexType::Char('a'),
            RegexType::Quant(Quantifier::Range(0, 0)),
        ]);
        let nfa = post2nfa(tokens).unwrap();

        // The resulting NFA should effectively be a match state
        // We can't check this directly, but we can ensure it was created successfully
        assert!(!State::is_none_ptr(&nfa.start));
        assert!(!State::is_nomatch_ptr(&nfa.start));
    }
}
