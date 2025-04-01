use crate::parsing::error::ParsingResult;
use crate::regex::{RegexType, TokenType};
use crate::regex::re2post::re2post;
use std::collections::VecDeque;

// Helper function to create token sequences for testing
fn create_token_sequence(tokens: Vec<TokenType>) -> VecDeque<TokenType> {
    let mut queue = VecDeque::new();
    for token in tokens {
        queue.push_back(token);
    }
    queue
}

#[test]
fn test_re2post_simple_literal() -> ParsingResult<()> {
    // Test with a simple literal 'a'
    let input = create_token_sequence(vec![
        TokenType::Literal(RegexType::Char('a')),
    ]);
    
    let result = re2post(input)?;
    
    assert_eq!(result.len(), 1);
    assert!(matches!(result[0], TokenType::Literal(RegexType::Char('a'))));
    
    Ok(())
}

#[test]
fn test_re2post_binary_operator() -> ParsingResult<()> {
    // Test with 'a|b' which should become 'a b |' in postfix
    let input = create_token_sequence(vec![
        TokenType::Literal(RegexType::Char('a')),
        TokenType::BinaryOperator(RegexType::Or),
        TokenType::Literal(RegexType::Char('b')),
    ]);
    
    let result = re2post(input)?;
    
    assert_eq!(result.len(), 3);
    assert!(matches!(result[0], TokenType::Literal(RegexType::Char('a'))));
    assert!(matches!(result[1], TokenType::Literal(RegexType::Char('b'))));
    assert!(matches!(result[2], TokenType::BinaryOperator(RegexType::Or)));
    
    Ok(())
}

#[test]
fn test_re2post_parentheses() -> ParsingResult<()> {
    // Test with '(a|b)' which should also become 'a b |' in postfix
    // since parentheses only affect the order of operations
    let input = create_token_sequence(vec![
        TokenType::OpenParenthesis(RegexType::OpenParenthesis),
        TokenType::Literal(RegexType::Char('a')),
        TokenType::BinaryOperator(RegexType::Or),
        TokenType::Literal(RegexType::Char('b')),
        TokenType::CloseParenthesis(RegexType::CloseParenthesis),
    ]);
    
    let result = re2post(input)?;
    
    assert_eq!(result.len(), 3);
    assert!(matches!(result[0], TokenType::Literal(RegexType::Char('a'))));
    assert!(matches!(result[1], TokenType::Literal(RegexType::Char('b'))));
    assert!(matches!(result[2], TokenType::BinaryOperator(RegexType::Or)));
    
    Ok(())
}

#[test]
fn test_re2post_complex_expression() -> ParsingResult<()> {
    // Test with '(a|b)c' which should become 'a b | c' in postfix
    // followed by an implicit concatenation
    let input = create_token_sequence(vec![
        TokenType::OpenParenthesis(RegexType::OpenParenthesis),
        TokenType::Literal(RegexType::Char('a')),
        TokenType::BinaryOperator(RegexType::Or),
        TokenType::Literal(RegexType::Char('b')),
        TokenType::CloseParenthesis(RegexType::CloseParenthesis),
        TokenType::Literal(RegexType::Char('c')),
    ]);
    
    let result = re2post(input)?;
    
    assert_eq!(result.len(), 5);
    assert!(matches!(result[0], TokenType::Literal(RegexType::Char('a'))));
    assert!(matches!(result[1], TokenType::Literal(RegexType::Char('b'))));
    assert!(matches!(result[2], TokenType::BinaryOperator(RegexType::Or)));
    assert!(matches!(result[3], TokenType::Literal(RegexType::Char('c'))));
    assert!(matches!(result[4], TokenType::BinaryOperator(RegexType::Concatenation)));
    
    Ok(())
}

#[test]
fn test_re2post_unary_operator() -> ParsingResult<()> {
    // Test with 'a?' which should become 'a ?' in postfix
    let input = create_token_sequence(vec![
        TokenType::Literal(RegexType::Char('a')),
        TokenType::UnaryOperator(RegexType::QuestionMark),
    ]);
    
    let result = re2post(input)?;
    
    assert_eq!(result.len(), 2);
    assert!(matches!(result[0], TokenType::Literal(RegexType::Char('a'))));
    assert!(matches!(result[1], TokenType::UnaryOperator(RegexType::QuestionMark)));
    
    Ok(())
}

#[test]
fn test_re2post_precedence() -> ParsingResult<()> {
    // Test with 'a|b?' which should become 'a b ? |' in postfix
    // because ? has higher precedence than |
    let input = create_token_sequence(vec![
        TokenType::Literal(RegexType::Char('a')),
        TokenType::BinaryOperator(RegexType::Or),
        TokenType::Literal(RegexType::Char('b')),
        TokenType::UnaryOperator(RegexType::QuestionMark),
    ]);
    
    let result = re2post(input)?;
    
    assert_eq!(result.len(), 4);
    assert!(matches!(result[0], TokenType::Literal(RegexType::Char('a'))));
    assert!(matches!(result[1], TokenType::Literal(RegexType::Char('b'))));
    assert!(matches!(result[2], TokenType::UnaryOperator(RegexType::QuestionMark)));
    assert!(matches!(result[3], TokenType::BinaryOperator(RegexType::Or)));
    
    Ok(())
}

#[test]
fn test_re2post_nested_parentheses() -> ParsingResult<()> {
    // Test with '((a|b)c)' which should become 'a b | c •' in postfix
    let input = create_token_sequence(vec![
        TokenType::OpenParenthesis(RegexType::OpenParenthesis),
        TokenType::OpenParenthesis(RegexType::OpenParenthesis),
        TokenType::Literal(RegexType::Char('a')),
        TokenType::BinaryOperator(RegexType::Or),
        TokenType::Literal(RegexType::Char('b')),
        TokenType::CloseParenthesis(RegexType::CloseParenthesis),
        TokenType::Literal(RegexType::Char('c')),
        TokenType::CloseParenthesis(RegexType::CloseParenthesis),
    ]);
    
    let result = re2post(input)?;
    
    assert_eq!(result.len(), 5);
    assert!(matches!(result[0], TokenType::Literal(RegexType::Char('a'))));
    assert!(matches!(result[1], TokenType::Literal(RegexType::Char('b'))));
    assert!(matches!(result[2], TokenType::BinaryOperator(RegexType::Or)));
    assert!(matches!(result[3], TokenType::Literal(RegexType::Char('c'))));
    assert!(matches!(result[4], TokenType::BinaryOperator(RegexType::Concatenation)));
    
    Ok(())
}

#[test]
fn test_re2post_mismatched_parentheses() {
    // Test with '(a|b' which should return an error
    let input = create_token_sequence(vec![
        TokenType::OpenParenthesis(RegexType::OpenParenthesis),
        TokenType::Literal(RegexType::Char('a')),
        TokenType::BinaryOperator(RegexType::Or),
        TokenType::Literal(RegexType::Char('b')),
    ]);
    
    let result = re2post(input);
    
    assert!(result.is_err());
}

#[test]
fn test_re2post_empty_input() -> ParsingResult<()> {
    // Test with empty input
    let input = create_token_sequence(vec![]);
    
    let result = re2post(input)?;
    
    assert_eq!(result.len(), 0);
    
    Ok(())
} 