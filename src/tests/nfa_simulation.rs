use std::collections::VecDeque;
use crate::regex::{
    input_match,
    post2nfa,
    RegexType,
    State,
    TokenType,
    Quantifier,
    StatePtr,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_tokens(tokens: Vec<RegexType>) -> VecDeque<TokenType> {
        tokens.into_iter().map(TokenType::from).collect()
    }

    fn create_nfa(tokens: Vec<RegexType>) -> StatePtr {
        let postfix_tokens = create_tokens(tokens);
        post2nfa(postfix_tokens).unwrap()
    }

    #[test]
    fn test_simple_match() {
        // NFA for the regex "a"
        let nfa = create_nfa(vec![RegexType::Char('a')]);
        
        // Should match "a"
        assert!(input_match(nfa.clone(), "a"));
        
        // Should not match "b" or other strings
        assert!(!input_match(nfa.clone(), "b"));
        assert!(!input_match(nfa.clone(), ""));
        assert!(!input_match(nfa.clone(), "aa"));
    }

    #[test]
    fn test_concatenation_match() {
        // NFA for the regex "ab"
        let nfa = create_nfa(vec![
            RegexType::Char('a'),
            RegexType::Char('b'),
            RegexType::Concatenation,
        ]);
        
        // Should match "ab"
        assert!(input_match(nfa.clone(), "ab"));
        
        // Should not match other strings
        assert!(!input_match(nfa.clone(), "a"));
        assert!(!input_match(nfa.clone(), "b"));
        assert!(!input_match(nfa.clone(), "ba"));
        assert!(!input_match(nfa.clone(), "abc"));
    }

    #[test]
    fn test_alternation_match() {
        // NFA for the regex "a|b"
        let nfa = create_nfa(vec![
            RegexType::Char('a'),
            RegexType::Char('b'),
            RegexType::Or,
        ]);
        
        // Should match either "a" or "b"
        assert!(input_match(nfa.clone(), "a"));
        assert!(input_match(nfa.clone(), "b"));
        
        // Should not match other strings
        assert!(!input_match(nfa.clone(), ""));
        assert!(!input_match(nfa.clone(), "ab"));
        assert!(!input_match(nfa.clone(), "c"));
    }

    #[test]
    fn test_optional_match() {
        // NFA for the regex "a?"
        let nfa = create_nfa(vec![
            RegexType::Char('a'),
            RegexType::QuestionMark,
        ]);
        
        // Should match both "" and "a"
        assert!(input_match(nfa.clone(), "a"));
        assert!(input_match(nfa.clone(), ""));
        
        // Should not match other strings
        assert!(!input_match(nfa.clone(), "aa"));
        assert!(!input_match(nfa.clone(), "b"));
    }

    #[test]
    fn test_exact_repetition() {
        // NFA for the regex "a{3}" (exactly 3 'a's)
        let nfa = create_nfa(vec![
            RegexType::Char('a'),
            RegexType::Quant(Quantifier::Exact(3)),
        ]);
        
        // Should match "aaa"
        assert!(input_match(nfa.clone(), "aaa"));
        
        // Should not match anything else
        assert!(!input_match(nfa.clone(), ""));
        assert!(!input_match(nfa.clone(), "a"));
        assert!(!input_match(nfa.clone(), "aa"));
        assert!(!input_match(nfa.clone(), "aaaa"));
    }

    #[test]
    fn test_at_least_repetition() {
        // NFA for the regex "a{2,}" (at least 2 'a's)
        let nfa = create_nfa(vec![
            RegexType::Char('a'),
            RegexType::Quant(Quantifier::AtLeast(2)),
        ]);
        
        // Should match "aa", "aaa", "aaaa", etc.
        assert!(input_match(nfa.clone(), "aa"));
        assert!(input_match(nfa.clone(), "aaa"));
        assert!(input_match(nfa.clone(), "aaaa"));
        
        // Should not match "", "a"
        assert!(!input_match(nfa.clone(), ""));
        assert!(!input_match(nfa.clone(), "a"));
    }

    #[test]
    fn test_range_repetition() {
        // NFA for the regex "a{2,4}" (between 2 and 4 'a's)
        let nfa = create_nfa(vec![
            RegexType::Char('a'),
            RegexType::Quant(Quantifier::Range(2, 4)),
        ]);
        
        // Should match "aa", "aaa", "aaaa"
        assert!(input_match(nfa.clone(), "aa"));
        assert!(input_match(nfa.clone(), "aaa"));
        assert!(input_match(nfa.clone(), "aaaa"));
        
        // Should not match "", "a", "aaaaa"
        assert!(!input_match(nfa.clone(), ""));
        assert!(!input_match(nfa.clone(), "a"));
        assert!(!input_match(nfa.clone(), "aaaaa"));
    }

    #[test]
    fn test_complex_pattern() {
        // NFA for the regex "a(b|c)d"
        let nfa = create_nfa(vec![
            RegexType::Char('a'),
            RegexType::Char('b'),
            RegexType::Char('c'),
            RegexType::Or,
            RegexType::Concatenation,
            RegexType::Char('d'),
            RegexType::Concatenation,
        ]);
        
        // Should match "abd" and "acd"
        assert!(input_match(nfa.clone(), "abd"));
        assert!(input_match(nfa.clone(), "acd"));
        
        // Should not match other strings
        assert!(!input_match(nfa.clone(), ""));
        assert!(!input_match(nfa.clone(), "ad"));
        assert!(!input_match(nfa.clone(), "abcd"));
    }
}
