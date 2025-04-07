use crate::regex::parsing::*;
use crate::regex::re2post::*;
use crate::regex::*;
use crate::parsing::error::ParsingResult;
use std::collections::VecDeque;

// Helper function to create a vector of tokens from a regex pattern
fn create_tokens(pattern: &str) -> ParsingResult<VecDeque<TokenType>> {
    let tokens = Regex::tokens(pattern)?;
    Ok(Regex::add_concatenation(tokens))
}

// ==============================================
// 1. BASIC CONVERSION FUNCTIONALITY TESTS
// ==============================================

#[test]
fn test_simple_literal_conversion() {
    let infix = create_tokens("abc").unwrap();
    let postfix = re2post(infix).unwrap();
    
    // Expected: ab&c& (a concatenated with b, then concatenated with c)
    assert_eq!(postfix.len(), 5);
    assert!(matches!(postfix[0].into_inner(), RegexType::Char('a')));
    assert!(matches!(postfix[1].into_inner(), RegexType::Char('b')));
    assert!(matches!(postfix[2].into_inner(), RegexType::Concatenation));
    assert!(matches!(postfix[3].into_inner(), RegexType::Char('c')));
    assert!(matches!(postfix[4].into_inner(), RegexType::Concatenation));
}

#[test]
fn test_basic_alternation() {
    let infix = create_tokens("a|b").unwrap();
    let postfix = re2post(infix).unwrap();
    
    // Expected: ab| (a or b)
    assert_eq!(postfix.len(), 3);
    assert!(matches!(postfix[0].into_inner(), RegexType::Char('a')));
    assert!(matches!(postfix[1].into_inner(), RegexType::Char('b')));
    assert!(matches!(postfix[2].into_inner(), RegexType::Or));
}

#[test]
fn test_basic_concatenation_and_alternation() {
    let infix = create_tokens("ab|c").unwrap();
    let postfix = re2post(infix).unwrap();
    
    // Expected: ab&c| (a concatenated with b, or c)
    assert_eq!(postfix.len(), 5);
    assert!(matches!(postfix[0].into_inner(), RegexType::Char('a')));
    assert!(matches!(postfix[1].into_inner(), RegexType::Char('b')));
    assert!(matches!(postfix[2].into_inner(), RegexType::Concatenation));
    assert!(matches!(postfix[3].into_inner(), RegexType::Char('c')));
    assert!(matches!(postfix[4].into_inner(), RegexType::Or));
}

// ==============================================
// 2. OPERATOR PRECEDENCE HANDLING TESTS
// ==============================================

#[test]
fn test_operator_precedence_concatenation_over_alternation() {
    let infix = create_tokens("a|bc").unwrap();
    let postfix = re2post(infix).unwrap();
    
    // Expected: abc&| (a or (b concatenated with c))
    assert_eq!(postfix.len(), 5);
    assert!(matches!(postfix[0].into_inner(), RegexType::Char('a')));
    assert!(matches!(postfix[1].into_inner(), RegexType::Char('b')));
    assert!(matches!(postfix[2].into_inner(), RegexType::Char('c')));
    assert!(matches!(postfix[3].into_inner(), RegexType::Concatenation));
    assert!(matches!(postfix[4].into_inner(), RegexType::Or));
}

#[test]
fn test_quantifier_precedence() {
    let infix = create_tokens("a+b").unwrap();
    let postfix = re2post(infix).unwrap();
    
    // Expected: a+b& (a+ concatenated with b)
    assert_eq!(postfix.len(), 4);
    assert!(matches!(postfix[0].into_inner(), RegexType::Char('a')));
    assert!(matches!(postfix[1].into_inner(), RegexType::Quant(Quantifier::AtLeast(1))));
    assert!(matches!(postfix[2].into_inner(), RegexType::Char('b')));
    assert!(matches!(postfix[3].into_inner(), RegexType::Concatenation));
}

#[test]
fn test_multiple_operators_with_precedence() {
    let infix = create_tokens("a+b|cd*").unwrap();
    let postfix = re2post(infix).unwrap();
    
    // Expected: a+b&cd*&|
    assert_eq!(postfix.len(), 9);
    assert!(matches!(postfix[0].into_inner(), RegexType::Char('a')));
    assert!(matches!(postfix[1].into_inner(), RegexType::Quant(Quantifier::AtLeast(1))));
    assert!(matches!(postfix[2].into_inner(), RegexType::Char('b')));
    assert!(matches!(postfix[3].into_inner(), RegexType::Concatenation));
    assert!(matches!(postfix[4].into_inner(), RegexType::Char('c')));
    assert!(matches!(postfix[5].into_inner(), RegexType::Char('d')));
    assert!(matches!(postfix[6].into_inner(), RegexType::Quant(Quantifier::AtLeast(0))));
    assert!(matches!(postfix[7].into_inner(), RegexType::Concatenation));
    assert!(matches!(postfix[8].into_inner(), RegexType::Or));
}

// ==============================================
// 3. PARENTHESES HANDLING TESTS
// ==============================================

#[test]
fn test_simple_parentheses() {
    let infix = create_tokens("(ab)").unwrap();
    let postfix = re2post(infix).unwrap();
    
    // Expected: ab& (a concatenated with b)
    assert_eq!(postfix.len(), 3);
    assert!(matches!(postfix[0].into_inner(), RegexType::Char('a')));
    assert!(matches!(postfix[1].into_inner(), RegexType::Char('b')));
    assert!(matches!(postfix[2].into_inner(), RegexType::Concatenation));
}

#[test]
fn test_parentheses_with_alternation() {
    let infix = create_tokens("(a|b)c").unwrap();
    let postfix = re2post(infix).unwrap();
    
    // Expected: ab|c& ((a or b) concatenated with c)
    assert_eq!(postfix.len(), 5);
    assert!(matches!(postfix[0].into_inner(), RegexType::Char('a')));
    assert!(matches!(postfix[1].into_inner(), RegexType::Char('b')));
    assert!(matches!(postfix[2].into_inner(), RegexType::Or));
    assert!(matches!(postfix[3].into_inner(), RegexType::Char('c')));
    assert!(matches!(postfix[4].into_inner(), RegexType::Concatenation));
}

#[test]
fn test_nested_parentheses() {
    let infix = create_tokens("((a|b)c)").unwrap();
    let postfix = re2post(infix).unwrap();
    
    // Expected: ab|c& ((a or b) concatenated with c)
    assert_eq!(postfix.len(), 5);
    assert!(matches!(postfix[0].into_inner(), RegexType::Char('a')));
    assert!(matches!(postfix[1].into_inner(), RegexType::Char('b')));
    assert!(matches!(postfix[2].into_inner(), RegexType::Or));
    assert!(matches!(postfix[3].into_inner(), RegexType::Char('c')));
    assert!(matches!(postfix[4].into_inner(), RegexType::Concatenation));
}

#[test]
fn test_complex_parentheses() {
    let infix = create_tokens("(a|b)(c|d)").unwrap();
    let postfix = re2post(infix).unwrap();
    
    // Expected: ab|cd|& ((a or b) concatenated with (c or d))
    assert_eq!(postfix.len(), 7);
    assert!(matches!(postfix[0].into_inner(), RegexType::Char('a')));
    assert!(matches!(postfix[1].into_inner(), RegexType::Char('b')));
    assert!(matches!(postfix[2].into_inner(), RegexType::Or));
    assert!(matches!(postfix[3].into_inner(), RegexType::Char('c')));
    assert!(matches!(postfix[4].into_inner(), RegexType::Char('d')));
    assert!(matches!(postfix[5].into_inner(), RegexType::Or));
    assert!(matches!(postfix[6].into_inner(), RegexType::Concatenation));
}

// ==============================================
// 4. LINE ANCHOR HANDLING TESTS
// ==============================================

#[test]
fn test_line_start_anchor() {
    let infix = create_tokens("^abc").unwrap();
    let postfix = re2post(infix).unwrap();
    
    // Expected: ^ab&c& (line start, followed by a concatenated with b, then concatenated with c)
    assert_eq!(postfix.len(), 6);
    assert!(matches!(postfix[0].into_inner(), RegexType::LineStart));
    assert!(matches!(postfix[1].into_inner(), RegexType::Char('a')));
    assert!(matches!(postfix[2].into_inner(), RegexType::Char('b')));
    assert!(matches!(postfix[3].into_inner(), RegexType::Concatenation));
    assert!(matches!(postfix[4].into_inner(), RegexType::Char('c')));
    assert!(matches!(postfix[5].into_inner(), RegexType::Concatenation));
}

#[test]
fn test_line_end_anchor() {
    let infix = create_tokens("abc$").unwrap();
    let postfix = re2post(infix).unwrap();
    
    // Expected: ab&c&$ (a concatenated with b, then concatenated with c, followed by line end)
    assert_eq!(postfix.len(), 6);
    assert!(matches!(postfix[0].into_inner(), RegexType::Char('a')));
    assert!(matches!(postfix[1].into_inner(), RegexType::Char('b')));
    assert!(matches!(postfix[2].into_inner(), RegexType::Concatenation));
    assert!(matches!(postfix[3].into_inner(), RegexType::Char('c')));
    assert!(matches!(postfix[4].into_inner(), RegexType::Concatenation));
    assert!(matches!(postfix[5].into_inner(), RegexType::LineEnd));
}

#[test]
fn test_both_line_anchors() {
    let infix = create_tokens("^abc$").unwrap();
    let postfix = re2post(infix).unwrap();
    
    // Expected: ^ab&c&$ (line start, followed by a concatenated with b, then concatenated with c, followed by line end)
    assert_eq!(postfix.len(), 7);
    assert!(matches!(postfix[0].into_inner(), RegexType::LineStart));
    assert!(matches!(postfix[1].into_inner(), RegexType::Char('a')));
    assert!(matches!(postfix[2].into_inner(), RegexType::Char('b')));
    assert!(matches!(postfix[3].into_inner(), RegexType::Concatenation));
    assert!(matches!(postfix[4].into_inner(), RegexType::Char('c')));
    assert!(matches!(postfix[5].into_inner(), RegexType::Concatenation));
    assert!(matches!(postfix[6].into_inner(), RegexType::LineEnd));
}

// ==============================================
// 5. ERROR DETECTION AND HANDLING TESTS
// ==============================================

#[test]
fn test_unclosed_parenthesis() {
    let infix = create_tokens("(abc").unwrap();
    let result = re2post(infix);
    
    assert!(result.is_err());
    // Check error message contains "Unclosed parenthesis"
    assert!(result.unwrap_err().to_string().contains("Unclosed parenthesis"));
}

#[test]
fn test_misplaced_line_start_anchor() {
    let infix = create_tokens("a^bc").unwrap();
    let result = re2post(infix);
    
    assert!(result.is_err());
    // Check error message contains "Unexpected '^'"
    assert!(result.unwrap_err().to_string().contains("Unexpected '^'"));
}

#[test]
fn test_misplaced_line_end_anchor() {
    let infix = create_tokens("ab$c").unwrap();
    let result = re2post(infix);
    
    assert!(result.is_err());
    // Check error message contains "Unexpected '$'"
    assert!(result.unwrap_err().to_string().contains("Unexpected '$'"));
}

#[test]
fn test_duplicate_line_start_anchor() {
    let infix = create_tokens("^^abc").unwrap();
    let result = re2post(infix);
    
    assert!(result.is_err());
    // Check error message contains "Unexpected '^'"
    assert!(result.unwrap_err().to_string().contains("Unexpected '^'"));
}

#[test]
fn test_duplicate_line_end_anchor() {
    let infix = create_tokens("abc$$").unwrap();
    let result = re2post(infix);
    
    assert!(result.is_err());
    // Check error message contains "Unexpected '$'"
    assert!(result.unwrap_err().to_string().contains("Unexpected '$'"));
}

// ==============================================
// 6. EMPTY EXPRESSION HANDLING TESTS
// ==============================================

#[test]
fn test_empty_expression() {
    let infix = create_tokens("").unwrap();
    let postfix = re2post(infix).unwrap();
    
    // Empty expression should result in empty output
    assert_eq!(postfix.len(), 0);
}

#[test]
fn test_only_line_anchors() {
    let infix = create_tokens("^$").unwrap();
    let postfix = re2post(infix).unwrap();
    
    // Expected: ^$ (line start followed by line end)
    assert_eq!(postfix.len(), 2);
    assert!(matches!(postfix[0].into_inner(), RegexType::LineStart));
    assert!(matches!(postfix[1].into_inner(), RegexType::LineEnd));
}

// ==============================================
// 7. EDGE CASES TESTS
// ==============================================

#[test]
fn test_complex_expression() {
    let infix = create_tokens("(a|b)+c*(d|e)?").unwrap();
    let postfix = re2post(infix).unwrap();
    
    // Expected: ab|+c*&de|?&
    assert!(postfix.len() > 0);
    // Verify the first few tokens and last token
    assert!(matches!(postfix[0].into_inner(), RegexType::Char('a')));
    assert!(matches!(postfix[1].into_inner(), RegexType::Char('b')));
    assert!(matches!(postfix[2].into_inner(), RegexType::Or));
    assert!(matches!(postfix[3].into_inner(), RegexType::Quant(Quantifier::AtLeast(1))));
    // Last token should be a concatenation
    assert!(matches!(postfix[postfix.len()-1].into_inner(), RegexType::Concatenation));
}

#[test]
fn test_deeply_nested_parentheses() {
    let infix = create_tokens("(((a)))").unwrap();
    let postfix = re2post(infix).unwrap();
    
    // Should just be 'a' in postfix notation
    assert_eq!(postfix.len(), 1);
    assert!(matches!(postfix[0].into_inner(), RegexType::Char('a')));
}

#[test]
fn test_multiple_alternations() {
    let infix = create_tokens("a|b|c|d").unwrap();
    let postfix = re2post(infix).unwrap();
    
    // Expected: ab|c|d| (a or b or c or d)
    assert_eq!(postfix.len(), 7);
    assert!(matches!(postfix[0].into_inner(), RegexType::Char('a')));
    assert!(matches!(postfix[1].into_inner(), RegexType::Char('b')));
    assert!(matches!(postfix[2].into_inner(), RegexType::Or));
    assert!(matches!(postfix[3].into_inner(), RegexType::Char('c')));
    assert!(matches!(postfix[4].into_inner(), RegexType::Or));
    assert!(matches!(postfix[5].into_inner(), RegexType::Char('d')));
    assert!(matches!(postfix[6].into_inner(), RegexType::Or));
}

// ==============================================
// 8. INTEGRATION WITH TOKENTYPE TESTS
// ==============================================

#[test]
fn test_character_class_handling() {
    // Create a token sequence representing alternation of characters (a|b|c)
    // which is how character classes are now implemented
    let mut tokens = VecDeque::new();
    tokens.push_back(TokenType::OpenParenthesis(RegexType::OpenParenthesis));
    tokens.push_back(TokenType::Literal(RegexType::Char('a')));
    tokens.push_back(TokenType::BinaryOperator(RegexType::Or));
    tokens.push_back(TokenType::Literal(RegexType::Char('b')));
    tokens.push_back(TokenType::BinaryOperator(RegexType::Or));
    tokens.push_back(TokenType::Literal(RegexType::Char('c')));
    tokens.push_back(TokenType::CloseParenthesis(RegexType::CloseParenthesis));
    tokens.push_back(TokenType::Literal(RegexType::Char('d')));
    tokens.push_back(TokenType::BinaryOperator(RegexType::Concatenation));
    
    let postfix = re2post(tokens).unwrap();
    
    // Expected: abc||d& (alternation of a, b, c concatenated with d)
    assert!(postfix.len() > 0);
    assert!(matches!(postfix[0].into_inner(), RegexType::Char('a')));
    assert!(matches!(postfix[1].into_inner(), RegexType::Char('b')));
    assert!(matches!(postfix[2].into_inner(), RegexType::Or));
    assert!(matches!(postfix[3].into_inner(), RegexType::Char('c')));
    assert!(matches!(postfix[4].into_inner(), RegexType::Or));
    assert!(matches!(postfix[5].into_inner(), RegexType::Char('d')));
    assert!(matches!(postfix[6].into_inner(), RegexType::Concatenation));
}

#[test]
fn test_any_character_handling() {
    // For the "." wildcard, we now expect an alternation of all possible characters
    // Let's test with a simple pattern "a.b" which should be parsed to something like "a(char1|char2|...)b"
    let infix = create_tokens("a.b").unwrap();
    let postfix = re2post(infix).unwrap();
    
    // Expected: a<alternation_structure>b& (a concatenated with alternation, then concatenated with b)
    // Just verify the first and last part since we don't know the exact alternation structure
    assert!(postfix.len() > 0);
    assert!(matches!(postfix[0].into_inner(), RegexType::Char('a')));
    // Last two tokens should be 'b' and concatenation
    assert!(matches!(postfix[postfix.len()-2].into_inner(), RegexType::Char('b')));
    assert!(matches!(postfix[postfix.len()-1].into_inner(), RegexType::Concatenation));
}

#[test]
fn test_mixed_token_types() {
    // Create a regex with various token types: literals, groups, quantifiers, etc.
    let infix = create_tokens("a(b|c)*d").unwrap();
    let postfix = re2post(infix).unwrap();
    
    // Expected: abc|*&d& (a concatenated with (b or c)*, then concatenated with d)
    assert!(postfix.len() > 0);
    assert!(matches!(postfix[0].into_inner(), RegexType::Char('a')));
    assert!(matches!(postfix[1].into_inner(), RegexType::Char('b')));
    assert!(matches!(postfix[2].into_inner(), RegexType::Char('c')));
    assert!(matches!(postfix[3].into_inner(), RegexType::Or));
    assert!(matches!(postfix[4].into_inner(), RegexType::Quant(Quantifier::AtLeast(0))));
    // Last token should be a concatenation
    assert!(matches!(postfix[postfix.len()-1].into_inner(), RegexType::Concatenation));
}
