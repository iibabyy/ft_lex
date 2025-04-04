use crate::regex::*;
use crate::regex::post2nfa::*;
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
        let frag = Fragment::char(s);
        
        // Test deep clone
        let cloned = frag.deep_clone();
        
        // Test concatenation (and)
        let s2 = State::basic(RegexType::Char('b'));
        let frag2 = Fragment::char(s2);
        let concat = frag.and(frag2);
        
        // We can't easily check the structure, but we can verify it doesn't panic
    }

    #[test]
    fn test_fragment_quantifiers() {
        // Test optional fragment
        let s = State::basic(RegexType::Char('a'));
        let frag = Fragment::char(s);
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
        assert!(State::is_basic_ptr(&nfa));
        
        // Test concatenation
        let tokens = create_token_queue(vec![
            RegexType::Char('a'),
            RegexType::Char('b'),
            RegexType::Concatenation
        ]);
        let nfa = post2nfa(tokens);
        assert!(nfa.is_ok());
        
        // Test alternation (or)
        let tokens = create_token_queue(vec![
            RegexType::Char('a'),
            RegexType::Char('b'),
            RegexType::Or
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
            RegexType::QuestionMark
        ]);
        let nfa = post2nfa(tokens);
        assert!(nfa.is_ok());
        
        // Test a{2,3}
        let tokens = create_token_queue(vec![
            RegexType::Char('a'),
            RegexType::Quant(Quantifier::Range(2, 3))
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
}
