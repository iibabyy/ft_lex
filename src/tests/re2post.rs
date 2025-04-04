use crate::parsing::error::ParsingResult;
use crate::regex::{re2post, TokenType, RegexType, Quantifier};
use std::collections::VecDeque;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_re2post_simple_characters() -> ParsingResult<()> {
        // Test "abc" -> "ab·c·"
        let mut tokens = VecDeque::new();
        tokens.push_back(TokenType::Literal(RegexType::Char('a')));
        tokens.push_back(TokenType::BinaryOperator(RegexType::Concatenation));
        tokens.push_back(TokenType::Literal(RegexType::Char('b')));
        tokens.push_back(TokenType::BinaryOperator(RegexType::Concatenation));
        tokens.push_back(TokenType::Literal(RegexType::Char('c')));
        
        let result = re2post(tokens)?;
        
        assert_eq!(result.len(), 5);
        assert!(matches!(result[0], TokenType::Literal(RegexType::Char('a'))));
        assert!(matches!(result[1], TokenType::Literal(RegexType::Char('b'))));
        assert!(matches!(result[2], TokenType::BinaryOperator(RegexType::Concatenation)));
        assert!(matches!(result[3], TokenType::Literal(RegexType::Char('c'))));
        assert!(matches!(result[4], TokenType::BinaryOperator(RegexType::Concatenation)));
        
        Ok(())
    }

    #[test]
    fn test_re2post_alternation() -> ParsingResult<()> {
        // Test "a|b" -> "ab|"
        let mut tokens = VecDeque::new();
        tokens.push_back(TokenType::Literal(RegexType::Char('a')));
        tokens.push_back(TokenType::BinaryOperator(RegexType::Or));
        tokens.push_back(TokenType::Literal(RegexType::Char('b')));
        
        let result = re2post(tokens)?;
        
        assert_eq!(result.len(), 3);
        assert!(matches!(result[0], TokenType::Literal(RegexType::Char('a'))));
        assert!(matches!(result[1], TokenType::Literal(RegexType::Char('b'))));
        assert!(matches!(result[2], TokenType::BinaryOperator(RegexType::Or)));
        
        Ok(())
    }

    #[test]
    fn test_re2post_parentheses() -> ParsingResult<()> {
        // Test "(a|b)c" -> "ab|c·"
        let mut tokens = VecDeque::new();
        tokens.push_back(TokenType::OpenParenthesis(RegexType::OpenParenthesis));
        tokens.push_back(TokenType::Literal(RegexType::Char('a')));
        tokens.push_back(TokenType::BinaryOperator(RegexType::Or));
        tokens.push_back(TokenType::Literal(RegexType::Char('b')));
        tokens.push_back(TokenType::CloseParenthesis(RegexType::CloseParenthesis));
        tokens.push_back(TokenType::BinaryOperator(RegexType::Concatenation));
        tokens.push_back(TokenType::Literal(RegexType::Char('c')));
        
        let result = re2post(tokens)?;
        
        assert_eq!(result.len(), 5);
        assert!(matches!(result[0], TokenType::Literal(RegexType::Char('a'))));
        assert!(matches!(result[1], TokenType::Literal(RegexType::Char('b'))));
        assert!(matches!(result[2], TokenType::BinaryOperator(RegexType::Or)));
        assert!(matches!(result[3], TokenType::Literal(RegexType::Char('c'))));
        assert!(matches!(result[4], TokenType::BinaryOperator(RegexType::Concatenation)));
        
        Ok(())
    }

    #[test]
    fn test_re2post_quantifiers() -> ParsingResult<()> {
        // Test "a+b*" -> "a+b*·"
        let mut tokens = VecDeque::new();
        tokens.push_back(TokenType::Literal(RegexType::Char('a')));
        tokens.push_back(TokenType::UnaryOperator(RegexType::Quant(Quantifier::AtLeast(1))));
        tokens.push_back(TokenType::BinaryOperator(RegexType::Concatenation));
        tokens.push_back(TokenType::Literal(RegexType::Char('b')));
        tokens.push_back(TokenType::UnaryOperator(RegexType::Quant(Quantifier::AtLeast(0))));
        
        let result = re2post(tokens)?;
        
        assert_eq!(result.len(), 5);
        assert!(matches!(result[0], TokenType::Literal(RegexType::Char('a'))));
        assert!(matches!(result[1], TokenType::UnaryOperator(RegexType::Quant(Quantifier::AtLeast(1)))));
        assert!(matches!(result[2], TokenType::Literal(RegexType::Char('b'))));
        assert!(matches!(result[3], TokenType::UnaryOperator(RegexType::Quant(Quantifier::AtLeast(0)))));
        assert!(matches!(result[4], TokenType::BinaryOperator(RegexType::Concatenation)));
        
        Ok(())
    }

    #[test]
    fn test_re2post_complex_expression() -> ParsingResult<()> {
        // Test "(a|b)+c" -> "ab|+c·"
        let mut tokens = VecDeque::new();
        tokens.push_back(TokenType::OpenParenthesis(RegexType::OpenParenthesis));
        tokens.push_back(TokenType::Literal(RegexType::Char('a')));
        tokens.push_back(TokenType::BinaryOperator(RegexType::Or));
        tokens.push_back(TokenType::Literal(RegexType::Char('b')));
        tokens.push_back(TokenType::CloseParenthesis(RegexType::CloseParenthesis));
        tokens.push_back(TokenType::UnaryOperator(RegexType::Quant(Quantifier::AtLeast(1))));
        tokens.push_back(TokenType::BinaryOperator(RegexType::Concatenation));
        tokens.push_back(TokenType::Literal(RegexType::Char('c')));
        
        let result = re2post(tokens)?;
        
        assert_eq!(result.len(), 6);
        assert!(matches!(result[0], TokenType::Literal(RegexType::Char('a'))));
        assert!(matches!(result[1], TokenType::Literal(RegexType::Char('b'))));
        assert!(matches!(result[2], TokenType::BinaryOperator(RegexType::Or)));
        assert!(matches!(result[3], TokenType::UnaryOperator(RegexType::Quant(Quantifier::AtLeast(1)))));
        assert!(matches!(result[4], TokenType::Literal(RegexType::Char('c'))));
        assert!(matches!(result[5], TokenType::BinaryOperator(RegexType::Concatenation)));
        
        Ok(())
    }

    #[test]
    fn test_re2post_multiple_alternations() -> ParsingResult<()> {
        // Test "a|b|c" -> "ab|c|"
        let mut tokens = VecDeque::new();
        tokens.push_back(TokenType::Literal(RegexType::Char('a')));
        tokens.push_back(TokenType::BinaryOperator(RegexType::Or));
        tokens.push_back(TokenType::Literal(RegexType::Char('b')));
        tokens.push_back(TokenType::BinaryOperator(RegexType::Or));
        tokens.push_back(TokenType::Literal(RegexType::Char('c')));
        
        let result = re2post(tokens)?;
        
        assert_eq!(result.len(), 5);
        assert!(matches!(result[0], TokenType::Literal(RegexType::Char('a'))));
        assert!(matches!(result[1], TokenType::Literal(RegexType::Char('b'))));
        assert!(matches!(result[2], TokenType::BinaryOperator(RegexType::Or)));
        assert!(matches!(result[3], TokenType::Literal(RegexType::Char('c'))));
        assert!(matches!(result[4], TokenType::BinaryOperator(RegexType::Or)));
        
        Ok(())
    }

    #[test]
    fn test_re2post_nested_parentheses() -> ParsingResult<()> {
        // Test "(a|(b|c))" -> "abc||"
        let mut tokens = VecDeque::new();
        tokens.push_back(TokenType::OpenParenthesis(RegexType::OpenParenthesis));
        tokens.push_back(TokenType::Literal(RegexType::Char('a')));
        tokens.push_back(TokenType::BinaryOperator(RegexType::Or));
        tokens.push_back(TokenType::OpenParenthesis(RegexType::OpenParenthesis));
        tokens.push_back(TokenType::Literal(RegexType::Char('b')));
        tokens.push_back(TokenType::BinaryOperator(RegexType::Or));
        tokens.push_back(TokenType::Literal(RegexType::Char('c')));
        tokens.push_back(TokenType::CloseParenthesis(RegexType::CloseParenthesis));
        tokens.push_back(TokenType::CloseParenthesis(RegexType::CloseParenthesis));
        
        let result = re2post(tokens)?;
        
        assert_eq!(result.len(), 5);
        assert!(matches!(result[0], TokenType::Literal(RegexType::Char('a'))));
        assert!(matches!(result[1], TokenType::Literal(RegexType::Char('b'))));
        assert!(matches!(result[2], TokenType::Literal(RegexType::Char('c'))));
        assert!(matches!(result[3], TokenType::BinaryOperator(RegexType::Or)));
        assert!(matches!(result[4], TokenType::BinaryOperator(RegexType::Or)));
        
        Ok(())
    }

    #[test]
    fn test_re2post_unbalanced_parentheses() {
        // Test "(a|b" (missing closing parenthesis)
        let mut tokens = VecDeque::new();
        tokens.push_back(TokenType::OpenParenthesis(RegexType::OpenParenthesis));
        tokens.push_back(TokenType::Literal(RegexType::Char('a')));
        tokens.push_back(TokenType::BinaryOperator(RegexType::Or));
        tokens.push_back(TokenType::Literal(RegexType::Char('b')));
        
        let result = re2post(tokens);
        
        // Should return an error
        assert!(result.is_err());
    }
}
