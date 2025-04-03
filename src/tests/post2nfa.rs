use std::collections::VecDeque;
use crate::regex::{post2nfa, State, TokenType, RegexType, Quantifier};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_tokens(tokens: Vec<RegexType>) -> VecDeque<TokenType> {
        tokens.into_iter().map(TokenType::from).collect()
    }

    #[test]
    fn test_simple_character() {
        let tokens = create_tokens(vec![RegexType::Char('a')]);
        let nfa = post2nfa(tokens).unwrap();
        
        assert!(State::is_basic_ptr(&nfa));
        let state = State::from_ptr(&nfa);
        
        match state.into_basic() {
            Some(basic) => {
                match &basic.c {
                    RegexType::Char(c) => assert_eq!(*c, 'a'),
                    _ => panic!("Expected character"),
                }
                
                let out_state = State::from_ptr(&basic.out);
                assert!(out_state.is_match());
            },
            None => panic!("Expected basic state"),
        }
    }

    #[test]
    fn test_concatenation() {
        let tokens = create_tokens(vec![
            RegexType::Char('a'), 
            RegexType::Char('b'), 
            RegexType::Concatenation
        ]);
        
        let nfa = post2nfa(tokens).unwrap();
        
        assert!(State::is_basic_ptr(&nfa));
        let state = State::from_ptr(&nfa);
        
        match state.into_basic() {
            Some(basic) => {
                match &basic.c {
                    RegexType::Char(c) => assert_eq!(*c, 'a'),
                    _ => panic!("Expected character 'a'"),
                }
                
                let out_state = State::from_ptr(&basic.out);
                
                match out_state.into_basic() {
                    Some(next_basic) => {
                        match &next_basic.c {
                            RegexType::Char(c) => assert_eq!(*c, 'b'),
                            _ => panic!("Expected character 'b'"),
                        }
                        
                        let final_state = State::from_ptr(&next_basic.out);
                        assert!(final_state.is_match());
                    },
                    None => panic!("Expected basic state for 'b'"),
                }
            },
            None => panic!("Expected basic state for 'a'"),
        }
    }

    #[test]
    fn test_alternation() {
        let tokens = create_tokens(vec![
            RegexType::Char('a'), 
            RegexType::Char('b'), 
            RegexType::Or
        ]);
        
        let nfa = post2nfa(tokens).unwrap();
        
        assert!(State::is_split_ptr(&nfa));
        let state = State::from_ptr(&nfa);
        
        match state.into_split() {
            Some(split) => {
                // Check first branch (a)
                let out1_state = State::from_ptr(&split.out1);
                match out1_state.into_basic() {
                    Some(basic) => {
                        match &basic.c {
                            RegexType::Char(c) => assert_eq!(*c, 'a'),
                            _ => panic!("Expected character 'a' in first branch"),
                        }
                    },
                    None => panic!("Expected basic state in first branch"),
                }
                
                // Check second branch (b)
                let out2_state = State::from_ptr(&split.out2);
                match out2_state.into_basic() {
                    Some(basic) => {
                        match &basic.c {
                            RegexType::Char(c) => assert_eq!(*c, 'b'),
                            _ => panic!("Expected character 'b' in second branch"),
                        }
                    },
                    None => panic!("Expected basic state in second branch"),
                }
            },
            None => panic!("Expected split state"),
        }
    }

    #[test]
    fn test_optional() {
        let tokens = create_tokens(vec![
            RegexType::Char('a'), 
            RegexType::QuestionMark
        ]);
        
        let nfa = post2nfa(tokens).unwrap();
        
        assert!(State::is_split_ptr(&nfa));
    }

    #[test]
    fn test_quantifier_exact() {
        let tokens = create_tokens(vec![
            RegexType::Char('a'), 
            RegexType::Quant(Quantifier::Exact(3))
        ]);
        
        let nfa = post2nfa(tokens).unwrap();
        assert!(State::is_basic_ptr(&nfa));
    }

    #[test]
    fn test_error_on_invalid_postfix() {
        // Only OR operator without operands
        let tokens = create_tokens(vec![RegexType::Or]);
        let result = post2nfa(tokens);
        assert!(result.is_err());
        
        // Question mark without preceding expression
        let tokens = create_tokens(vec![RegexType::QuestionMark]);
        let result = post2nfa(tokens);
        assert!(result.is_err());
    }
}
