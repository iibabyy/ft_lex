use crate::parsing::error::ParsingResult;
use crate::regex::{post2nfa, TokenType, RegexType, Quantifier, StatePtr, State, input_match};
use std::collections::VecDeque;
use std::rc::Rc;

// Helper function to create a VecDeque of TokenTypes for testing
fn create_token_deque(tokens: Vec<TokenType>) -> VecDeque<TokenType> {
    let mut deque = VecDeque::new();
    for token in tokens {
        deque.push_back(token);
    }
    deque
}

#[test]
fn test_post2nfa_single_char() -> ParsingResult<()> {
    // Test a simple 'a' character
    let tokens = vec![
        TokenType::Literal(RegexType::Char('a')),
    ];

    
    let result = post2nfa(create_token_deque(tokens))?;
    
    // Verify we got a valid state
    assert!(matches!(result.as_ref(), State::Basic(_)));
    
    // Test matching
    assert!(input_match(Some(result.clone()), "a"));
    assert!(!input_match(Some(result), "b"));
    
    Ok(())
}

#[test]
fn test_post2nfa_concatenation() -> ParsingResult<()> {
    // Test 'ab' (a·b in postfix notation)
    let tokens = vec![
        TokenType::Literal(RegexType::Char('a')),
        TokenType::Literal(RegexType::Char('b')),
        TokenType::BinaryOperator(RegexType::Concatenation),
    ];
    
    let result = post2nfa(create_token_deque(tokens))?;
    
    // Test matching
    assert!(input_match(Some(result.clone()), "ab"));
    assert!(!input_match(Some(result.clone()), "a"));
    assert!(!input_match(Some(result.clone()), "b"));
    assert!(!input_match(Some(result), "abc"));
    
    Ok(())
}

#[test]
fn test_post2nfa_alternation() -> ParsingResult<()> {
    // Test 'a|b' (ab| in postfix notation)
    let tokens = vec![
        TokenType::Literal(RegexType::Char('a')),
        TokenType::Literal(RegexType::Char('b')),
        TokenType::BinaryOperator(RegexType::Or),
    ];

    let result = post2nfa(create_token_deque(tokens))?;
    
    // Test matching
    assert!(input_match(Some(result.clone()), "a"));
    assert!(input_match(Some(result.clone()), "b"));
    assert!(!input_match(Some(result), "c"));
    
    Ok(())
}

#[test]
fn test_post2nfa_optional() -> ParsingResult<()> {
    // Test 'a?' (a? in postfix notation)
    let tokens = vec![
        TokenType::Literal(RegexType::Char('a')),
        TokenType::UnaryOperator(RegexType::QuestionMark),
    ];
    
    let result = post2nfa(create_token_deque(tokens))?;
    
    // Test matching
    assert!(input_match(Some(result.clone()), "a"));
    assert!(input_match(Some(result.clone()), ""));
    assert!(!input_match(Some(result), "b"));
    
    Ok(())
}

#[test]
fn test_post2nfa_exact_quantifier() -> ParsingResult<()> {
    // Test 'a{3}' (a{3} in postfix notation)
    let tokens = vec![
        TokenType::Literal(RegexType::Char('a')),
        TokenType::UnaryOperator(RegexType::Quant(Quantifier::Exact(3))),
    ];
    
    let result = post2nfa(create_token_deque(tokens))?;
    
    // Test matching
    assert!(input_match(Some(result.clone()), "aaa"));
    assert!(!input_match(Some(result.clone()), "aa"));
    assert!(!input_match(Some(result.clone()), "aaaa"));
    assert!(!input_match(Some(result), ""));
    
    Ok(())
}

#[test]
fn test_post2nfa_at_least_quantifier() -> ParsingResult<()> {
    // Test 'a{2,}' (a{2,} in postfix notation)
    let tokens = vec![
        TokenType::Literal(RegexType::Char('a')),
        TokenType::UnaryOperator(RegexType::Quant(Quantifier::AtLeast(2))),
    ];
    
    let result = post2nfa(create_token_deque(tokens))?;
    
    // Test matching
    assert!(input_match(Some(result.clone()), "aa"));
    assert!(input_match(Some(result.clone()), "aaa"));
    assert!(input_match(Some(result.clone()), "aaaa"));
    assert!(!input_match(Some(result.clone()), "a"));
    assert!(!input_match(Some(result), ""));
    
    Ok(())
}

#[test]
fn test_post2nfa_range_quantifier() -> ParsingResult<()> {
    // Test 'a{2,4}' (a{2,4} in postfix notation)
    let tokens = vec![
        TokenType::Literal(RegexType::Char('a')),
        TokenType::UnaryOperator(RegexType::Quant(Quantifier::Range(2, 4))),
    ];
    
    let result = post2nfa(create_token_deque(tokens))?;
    
    // Test matching
    assert!(input_match(Some(result.clone()), "aa"));
    assert!(input_match(Some(result.clone()), "aaa"));
    assert!(input_match(Some(result.clone()), "aaaa"));
    assert!(!input_match(Some(result.clone()), "a"));
    assert!(!input_match(Some(result.clone()), "aaaaa"));
    assert!(!input_match(Some(result), ""));
    
    Ok(())
}

#[test]
fn test_post2nfa_complex_regex() -> ParsingResult<()> {
    // Test '(a|b)+c' which translates to 'ab|+c·' in postfix
    let tokens = vec![
        TokenType::Literal(RegexType::Char('a')),
        TokenType::Literal(RegexType::Char('b')),
        TokenType::BinaryOperator(RegexType::Or),
        TokenType::UnaryOperator(RegexType::Quant(Quantifier::AtLeast(1))),
        TokenType::Literal(RegexType::Char('c')),
        TokenType::BinaryOperator(RegexType::Concatenation),
    ];

    let result = post2nfa(create_token_deque(tokens))?;

    // Test matching
    assert!(input_match(Some(result.clone()), "ac"));
    assert!(input_match(Some(result.clone()), "bc"));
    assert!(input_match(Some(result.clone()), "aac"));
    assert!(input_match(Some(result.clone()), "abc"));
    assert!(input_match(Some(result.clone()), "ababc"));
    assert!(!input_match(Some(result.clone()), "c"));
    assert!(!input_match(Some(result.clone()), "ab"));
    assert!(!input_match(Some(result), ""));
    
    Ok(())
}

#[test]
fn test_post2nfa_error_case() {
    // Test invalid regex (empty expression)
    let tokens: Vec<TokenType> = vec![];
    
    let result = post2nfa(create_token_deque(tokens));
    
    // Should return an error
    assert!(result.is_err());
} 