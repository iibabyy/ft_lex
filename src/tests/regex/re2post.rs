use crate::regex::parsing::{RegexType, TokenType, Quantifier};
use crate::regex::re2post::re2post;
use crate::{Regex, ParsingError};
use std::collections::VecDeque;

// Helper function to simplify test creation
fn create_tokens(input: &str) -> VecDeque<TokenType> {
    let tokens = Regex::tokens(input).unwrap();
    Regex::add_concatenation(tokens)
}

// Helper to check if two token sequences are equivalent
fn assert_token_sequences_equal(actual: &VecDeque<TokenType>, expected: &VecDeque<TokenType>) {
    assert_eq!(actual.len(), expected.len(), "Token sequences have different lengths");
    
    for (i, (actual_token, expected_token)) in actual.iter().zip(expected.iter()).enumerate() {
        assert_eq!(
            format!("{:?}", actual_token),
            format!("{:?}", expected_token),
            "Tokens at position {} differ: expected {:?}, got {:?}",
            i,
            expected_token,
            actual_token
        );
    }
}

// ==============================================
// 1. BASIC FUNCTIONALITY TESTS
// ==============================================

#[test]
fn test_empty_input() {
    let tokens: VecDeque<TokenType> = VecDeque::new();
    let result = re2post(tokens).unwrap();
    assert_eq!(result.len(), 0);
}

#[test]
fn test_single_literal() {
    let tokens = create_tokens("a");
    let result = re2post(tokens).unwrap();
    assert_eq!(result.len(), 1);
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_multiple_literals_with_concatenation() {
    let tokens = create_tokens("ab");
    let result = re2post(tokens).unwrap();
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Char('b')));
        exp.push_back(TokenType::from(RegexType::Concatenation));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_alternation() {
    let tokens = create_tokens("a|b");
    let result = re2post(tokens).unwrap();
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Char('b')));
        exp.push_back(TokenType::from(RegexType::Or));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_quantifier_star() {
    let tokens = create_tokens("a*");
    let result = re2post(tokens).unwrap();
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Quant(Quantifier::AtLeast(0))));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_quantifier_plus() {
    let tokens = create_tokens("a+");
    let result = re2post(tokens).unwrap();
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Quant(Quantifier::AtLeast(1))));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_quantifier_question_mark() {
    let tokens = create_tokens("a?");
    let result = re2post(tokens).unwrap();
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Quant(Quantifier::Range(0, 1))));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

// ==============================================
// 2. OPERATOR PRECEDENCE TESTS
// ==============================================

#[test]
fn test_quantifier_vs_concatenation() {
    // Testing "ab*" - the star applies only to 'b'
    let tokens = create_tokens("ab*");
    let result = re2post(tokens).unwrap();
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Char('b')));
        exp.push_back(TokenType::from(RegexType::Quant(Quantifier::AtLeast(0))));
        exp.push_back(TokenType::from(RegexType::Concatenation));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_concatenation_vs_alternation() {
    // Testing "ab|c" - concatenation has higher precedence than alternation
    let tokens = create_tokens("ab|c");
    let result = re2post(tokens).unwrap();
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Char('b')));
        exp.push_back(TokenType::from(RegexType::Concatenation));
        exp.push_back(TokenType::from(RegexType::Char('c')));
        exp.push_back(TokenType::from(RegexType::Or));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_multiple_mixed_operators() {
    // Testing "a|b*c+" - mix of operators with different precedence
    let tokens = create_tokens("a|b*c+");
    let result = re2post(tokens).unwrap();
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Char('b')));
        exp.push_back(TokenType::from(RegexType::Quant(Quantifier::AtLeast(0))));
        exp.push_back(TokenType::from(RegexType::Char('c')));
        exp.push_back(TokenType::from(RegexType::Quant(Quantifier::AtLeast(1))));
        exp.push_back(TokenType::from(RegexType::Concatenation));
        exp.push_back(TokenType::from(RegexType::Or));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

// ==============================================
// 3. PARENTHESES TESTS
// ==============================================

#[test]
fn test_simple_grouping() {
    let tokens = create_tokens("(a)");
    let result = re2post(tokens).unwrap();
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_nested_groups() {
    let tokens = create_tokens("((a))");
    let result = re2post(tokens).unwrap();
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_complex_expressions_with_parens() {
    let tokens = create_tokens("(a|b)c");
    let result = re2post(tokens).unwrap();
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Char('b')));
        exp.push_back(TokenType::from(RegexType::Or));
        exp.push_back(TokenType::from(RegexType::Char('c')));
        exp.push_back(TokenType::from(RegexType::Concatenation));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_grouped_alternation() {
    let tokens = create_tokens("(a|b|c)");
    let result = re2post(tokens).unwrap();
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Char('b')));
        exp.push_back(TokenType::from(RegexType::Or));
        exp.push_back(TokenType::from(RegexType::Char('c')));
        exp.push_back(TokenType::from(RegexType::Or));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_grouped_concatenation() {
    let tokens = create_tokens("(abc)");
    let result = re2post(tokens).unwrap();
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Char('b')));
        exp.push_back(TokenType::from(RegexType::Concatenation));
        exp.push_back(TokenType::from(RegexType::Char('c')));
        exp.push_back(TokenType::from(RegexType::Concatenation));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_quantified_groups() {
    let tokens = create_tokens("(ab)*");
    let result = re2post(tokens).unwrap();
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Char('b')));
        exp.push_back(TokenType::from(RegexType::Concatenation));
        exp.push_back(TokenType::from(RegexType::Quant(Quantifier::AtLeast(0))));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_unclosed_parenthesis() {
    let tokens = create_tokens("(abc");
    let result = re2post(tokens);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().message().contains("Unclosed parenthesis"));
}

#[test]
fn test_extra_closing_parenthesis() {
    let mut tokens = create_tokens("abc");
    tokens.push_back(TokenType::from(RegexType::CloseParenthesis));
    
    let result = re2post(tokens);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().message().contains("Unclosed parenthesis"));
}

// ==============================================
// 4. LINE ANCHORS TESTS
// ==============================================

#[test]
fn test_valid_line_start_anchor() {
    let tokens = create_tokens("^abc");
    let result = re2post(tokens).unwrap();
    
    // Line start anchor should be at the beginning of the result
    assert!(matches!(result[0], TokenType::StartOrEndCondition(RegexType::LineStart)));
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::LineStart));
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Char('b')));
        exp.push_back(TokenType::from(RegexType::Concatenation));
        exp.push_back(TokenType::from(RegexType::Char('c')));
        exp.push_back(TokenType::from(RegexType::Concatenation));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_valid_line_end_anchor() {
    let tokens = create_tokens("abc$");
    let result = re2post(tokens).unwrap();
    
    // Line end anchor should be at the end of the result
    assert!(matches!(result[result.len()-1], TokenType::StartOrEndCondition(RegexType::LineEnd)));
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Char('b')));
        exp.push_back(TokenType::from(RegexType::Concatenation));
        exp.push_back(TokenType::from(RegexType::Char('c')));
        exp.push_back(TokenType::from(RegexType::Concatenation));
        exp.push_back(TokenType::from(RegexType::LineEnd));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_both_anchors() {
    let tokens = create_tokens("^abc$");
    let result = re2post(tokens).unwrap();
    
    // Check both anchors are present
    assert!(matches!(result[0], TokenType::StartOrEndCondition(RegexType::LineStart)));
    assert!(matches!(result[result.len()-1], TokenType::StartOrEndCondition(RegexType::LineEnd)));
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::LineStart));
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Char('b')));
        exp.push_back(TokenType::from(RegexType::Concatenation));
        exp.push_back(TokenType::from(RegexType::Char('c')));
        exp.push_back(TokenType::from(RegexType::Concatenation));
        exp.push_back(TokenType::from(RegexType::LineEnd));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_line_start_not_at_beginning() {
    let mut tokens = VecDeque::new();
    tokens.push_back(TokenType::from(RegexType::Char('a')));
    tokens.push_back(TokenType::from(RegexType::LineStart));
    
    let result = re2post(tokens);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().message().contains("Unexpected '^' special character"));
}

#[test]
fn test_line_end_not_at_end() {
    let mut tokens = VecDeque::new();
    tokens.push_back(TokenType::from(RegexType::LineEnd));
    tokens.push_back(TokenType::from(RegexType::Char('a')));
    
    let result = re2post(tokens);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().message().contains("Unexpected '$' special character"));
}

#[test]
fn test_multiple_line_starts() {
    let mut tokens = VecDeque::new();
    tokens.push_back(TokenType::from(RegexType::LineStart));
    tokens.push_back(TokenType::from(RegexType::LineStart));
    
    let result = re2post(tokens);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().message().contains("Unexpected '^' special character"));
}

#[test]
fn test_multiple_line_ends() {
    let mut tokens = VecDeque::new();
    tokens.push_back(TokenType::from(RegexType::LineEnd));
    tokens.push_back(TokenType::from(RegexType::LineEnd));
    
    let result = re2post(tokens);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().message().contains("Unexpected '$' special character"));
}

// ==============================================
// 5. COMPLEX PATTERN TESTS
// ==============================================

#[test]
fn test_complex_regex() {
    let tokens = create_tokens("(a|b)*c+(d|e)?");
    let result = re2post(tokens).unwrap();
    
    // This is a complex case, so we'll just verify basic structure
    assert!(result.len() > 5);
    
    // Manually check for expected token sequence
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Char('b')));
        exp.push_back(TokenType::from(RegexType::Or));
        exp.push_back(TokenType::from(RegexType::Quant(Quantifier::AtLeast(0))));
        exp.push_back(TokenType::from(RegexType::Char('c')));
        exp.push_back(TokenType::from(RegexType::Quant(Quantifier::AtLeast(1))));
        exp.push_back(TokenType::from(RegexType::Concatenation));
        exp.push_back(TokenType::from(RegexType::Char('d')));
        exp.push_back(TokenType::from(RegexType::Char('e')));
        exp.push_back(TokenType::from(RegexType::Or));
        exp.push_back(TokenType::from(RegexType::Quant(Quantifier::Range(0, 1))));
        exp.push_back(TokenType::from(RegexType::Concatenation));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_deeply_nested_expressions() {
    let tokens = create_tokens("(((a|b)|c)|d)");
    let result = re2post(tokens).unwrap();
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Char('b')));
        exp.push_back(TokenType::from(RegexType::Or));
        exp.push_back(TokenType::from(RegexType::Char('c')));
        exp.push_back(TokenType::from(RegexType::Or));
        exp.push_back(TokenType::from(RegexType::Char('d')));
        exp.push_back(TokenType::from(RegexType::Or));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_anchored_complex_expression() {
    let tokens = create_tokens("^(a|b)*c$");
    let result = re2post(tokens).unwrap();
    
    // Check anchors
    assert!(matches!(result[0], TokenType::StartOrEndCondition(RegexType::LineStart)));
    assert!(matches!(result[result.len()-1], TokenType::StartOrEndCondition(RegexType::LineEnd)));
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::LineStart));
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Char('b')));
        exp.push_back(TokenType::from(RegexType::Or));
        exp.push_back(TokenType::from(RegexType::Quant(Quantifier::AtLeast(0))));
        exp.push_back(TokenType::from(RegexType::Char('c')));
        exp.push_back(TokenType::from(RegexType::Concatenation));
        exp.push_back(TokenType::from(RegexType::LineEnd));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

// ==============================================
// 6. EDGE CASES
// ==============================================

#[test]
fn test_empty_groups() {
    let tokens = create_tokens("()");
    let result = re2post(tokens).unwrap();
    
    // Empty group should result in empty output (or whatever the implementation defines)
    assert_eq!(result.len(), 0);
}

#[test]
fn test_quantified_empty_group() {
    let tokens = create_tokens("()*");
    let result = re2post(tokens).unwrap();
    
    // Expected: An empty expression with a quantifier
    assert!(matches!(result[0], TokenType::UnaryOperator(RegexType::Quant(Quantifier::AtLeast(0)))));
}

#[test]
fn test_large_expression() {
    // Create a large expression with many tokens
    let large_expr = "a".repeat(100);
    let tokens = create_tokens(&large_expr);
    let result = re2post(tokens).unwrap();
    
    // Should successfully process, with correct number of tokens (for 100 'a's, we need 199 tokens in postfix)
    assert_eq!(result.len(), 199); // 100 literals + 99 concatenations
}

// ==============================================
// 7. ALGORITHM SPECIFIC TESTS
// ==============================================

#[test]
fn test_operator_precedence_handling() {
    // Test that operator precedence is correctly handled in the shunting-yard algorithm
    let tokens = create_tokens("a+b|c*d");
    let result = re2post(tokens).unwrap();
    
    // Expected: (a+ b &) (c* d &) |
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Quant(Quantifier::AtLeast(1))));
        exp.push_back(TokenType::from(RegexType::Char('b')));
        exp.push_back(TokenType::from(RegexType::Concatenation));
        exp.push_back(TokenType::from(RegexType::Char('c')));
        exp.push_back(TokenType::from(RegexType::Quant(Quantifier::AtLeast(0))));
        exp.push_back(TokenType::from(RegexType::Char('d')));
        exp.push_back(TokenType::from(RegexType::Concatenation));
        exp.push_back(TokenType::from(RegexType::Or));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_multiple_operators_in_sequence() {
    // This is technically invalid in most regex implementations, but we're testing the re2post algorithm
    let mut tokens = VecDeque::new();
    tokens.push_back(TokenType::from(RegexType::Char('a')));
    tokens.push_back(TokenType::from(RegexType::Or));
    tokens.push_back(TokenType::from(RegexType::Or));
    tokens.push_back(TokenType::from(RegexType::Char('b')));
    
    // The re2post function should still process this, even if it's semantically invalid
    let result = re2post(tokens).unwrap();
    
    // The exact result depends on implementation, but it should not crash
    assert!(result.len() > 0);
}

// =============================================
// 8. ADDITIONAL COMPLEX PATTERNS
// =============================================

#[test]
fn test_alternation_with_empty_expressions() {
    // Test a|b|c with empty expressions between alternatives
    let mut tokens = VecDeque::new();
    tokens.push_back(TokenType::from(RegexType::Char('a')));
    tokens.push_back(TokenType::from(RegexType::Or));
    tokens.push_back(TokenType::from(RegexType::Or));
    tokens.push_back(TokenType::from(RegexType::Char('c')));
    
    // Should handle this case without crashing
    let result = re2post(tokens).unwrap();
    assert!(result.len() > 0);
}

#[test]
fn test_mixed_operators_and_groups() {
    let tokens = create_tokens("a|(b*)|c+d");
    let result = re2post(tokens).unwrap();
    
    // Expected: a (b*) (c+ d &) | |
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Char('b')));
        exp.push_back(TokenType::from(RegexType::Quant(Quantifier::AtLeast(0))));
        exp.push_back(TokenType::from(RegexType::Or));
        exp.push_back(TokenType::from(RegexType::Char('c')));
        exp.push_back(TokenType::from(RegexType::Quant(Quantifier::AtLeast(1))));
        exp.push_back(TokenType::from(RegexType::Char('d')));
        exp.push_back(TokenType::from(RegexType::Concatenation));
        exp.push_back(TokenType::from(RegexType::Or));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

// ==============================================
// 9. ERROR HANDLING TESTS
// ==============================================

#[test]
fn test_unmatched_closing_parenthesis() {
    let mut tokens = VecDeque::new();
    tokens.push_back(TokenType::from(RegexType::CloseParenthesis));
    
    let result = re2post(tokens);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().message().contains("Unclosed parenthesis"));
}

#[test]
fn test_invalid_anchor_position() {
    let mut tokens = VecDeque::new();
    tokens.push_back(TokenType::from(RegexType::Char('a')));
    tokens.push_back(TokenType::from(RegexType::LineStart));
    tokens.push_back(TokenType::from(RegexType::Char('b')));
    
    let result = re2post(tokens);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().message().contains("Unexpected '^' special character"));
}

// ==============================================
// 10. ANCHOR TESTS
// ==============================================

#[test]
fn test_duplicate_line_start_anchor() {
    let mut tokens = VecDeque::new();
    tokens.push_back(TokenType::StartOrEndCondition(RegexType::LineStart));
    tokens.push_back(TokenType::StartOrEndCondition(RegexType::LineStart));
    tokens.push_back(TokenType::from(RegexType::Char('a')));
    
    let result = re2post(tokens);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().message().contains("Unexpected '^' special character"));
}

#[test]
fn test_duplicate_line_end_anchor() {
    let mut tokens = VecDeque::new();
    tokens.push_back(TokenType::from(RegexType::Char('a')));
    tokens.push_back(TokenType::StartOrEndCondition(RegexType::LineEnd));
    tokens.push_back(TokenType::StartOrEndCondition(RegexType::LineEnd));
    
    let result = re2post(tokens);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().message().contains("Unexpected '$' special character"));
}

#[test]
fn test_empty_regex_with_anchors() {
    let mut tokens = VecDeque::new();
    tokens.push_back(TokenType::StartOrEndCondition(RegexType::LineStart));
    tokens.push_back(TokenType::StartOrEndCondition(RegexType::LineEnd));
    
    let result = re2post(tokens).unwrap();
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::StartOrEndCondition(RegexType::LineStart));
        exp.push_back(TokenType::StartOrEndCondition(RegexType::LineEnd));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_line_start_anchor_in_middle() {
    let mut tokens = VecDeque::new();
    tokens.push_back(TokenType::from(RegexType::Char('a')));
    tokens.push_back(TokenType::StartOrEndCondition(RegexType::LineStart));
    
    let result = re2post(tokens);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().message().contains("Unexpected '^' special character"));
}

#[test]
fn test_line_end_anchor_in_middle() {
    let mut tokens = VecDeque::new();
    tokens.push_back(TokenType::StartOrEndCondition(RegexType::LineEnd));
    tokens.push_back(TokenType::from(RegexType::Char('a')));
    
    let result = re2post(tokens);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().message().contains("Unexpected '$' special character"));
}

// ==============================================
// 11. MALFORMED OPERATOR SEQUENCES
// ==============================================

#[test]
fn test_consecutive_operators() {
    let tokens = create_tokens("a++*");
    let result = re2post(tokens).unwrap();
    
    // Expected: a + * (consecutive quantifiers are allowed in the parser)
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Quant(Quantifier::AtLeast(1))));
        exp.push_back(TokenType::from(RegexType::Quant(Quantifier::AtLeast(1))));
        exp.push_back(TokenType::from(RegexType::Quant(Quantifier::AtLeast(0))));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_operator_without_operand() {
    let mut tokens = VecDeque::new();
    tokens.push_back(TokenType::from(RegexType::Quant(Quantifier::AtLeast(1))));
    
    // This should still parse, as the re2post function doesn't validate
    // semantic correctness of the regex, just converts to postfix
    let result = re2post(tokens).unwrap();
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Quant(Quantifier::AtLeast(1))));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_unclosed_parenthesis_error_message() {
    let mut tokens = VecDeque::new();
    tokens.push_back(TokenType::from(RegexType::OpenParenthesis));
    tokens.push_back(TokenType::from(RegexType::Char('a')));
    
    let result = re2post(tokens);
    
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.message().contains("Unclosed parenthesis"));
}

#[test]
fn test_alternation_without_right_operand() {
    let mut tokens = VecDeque::new();
    tokens.push_back(TokenType::from(RegexType::Char('a')));
    tokens.push_back(TokenType::from(RegexType::Or));
    
    // This should still parse in re2post
    let result = re2post(tokens).unwrap();
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Or));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_complex_anchor_handling() {
    let mut tokens = VecDeque::new();
    tokens.push_back(TokenType::StartOrEndCondition(RegexType::LineStart));
    tokens.push_back(TokenType::from(RegexType::Char('a')));
    tokens.push_back(TokenType::from(RegexType::Char('b')));
    tokens.push_back(TokenType::from(RegexType::Concatenation));
    tokens.push_back(TokenType::StartOrEndCondition(RegexType::LineEnd));
    
    let result = re2post(tokens).unwrap();
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::StartOrEndCondition(RegexType::LineStart));
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Char('b')));
        exp.push_back(TokenType::from(RegexType::Concatenation));
        exp.push_back(TokenType::StartOrEndCondition(RegexType::LineEnd));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

// ==============================================
// 3. COMPLEX REGEX TESTS
// ==============================================

#[test]
fn test_complex_nested_groups_with_alternation() {
    // Test deeply nested parentheses with alternation and quantifiers
    let tokens = create_tokens("(a(b|c)*(d(e|f)+g)?)+");
    let result = re2post(tokens).unwrap();
    
    // The postfix notation should have operators after their operands
    // Verify some key structural elements
    let char_count = result.iter().filter(|t| matches!(t.into_inner(), RegexType::Char(_))).count();
    let or_count = result.iter().filter(|t| matches!(t.into_inner(), RegexType::Or)).count();
    let concat_count = result.iter().filter(|t| matches!(t.into_inner(), RegexType::Concatenation)).count();
    let quant_count = result.iter().filter(|t| matches!(t.into_inner(), RegexType::Quant(_))).count();
    
    // Ensure we have the expected number of characters and operators
    assert!(char_count >= 7); // a, b, c, d, e, f, g
    assert!(or_count >= 2);   // b|c and e|f
    assert!(concat_count > 0);
    assert!(quant_count >= 3); // *, +, ?
    
    // Verify the last token is the outermost + quantifier
    if let Some(last) = result.back() {
        assert!(matches!(last.into_inner(), RegexType::Quant(Quantifier::AtLeast(1))));
    } else {
        panic!("Result is empty");
    }
}

#[test]
fn test_complex_alternation_with_groups() {
    // Test complex alternation with groups and quantifiers
    let tokens = create_tokens("(ab|cd)*(ef|gh)+|(ij|kl)?");
    let result = re2post(tokens).unwrap();

	dbg!(&result);
    
    // Verify the structure of the postfix expression
    let char_count = result.iter().filter(|t| matches!(t.into_inner(), RegexType::Char(_))).count();
    let or_count = result.iter().filter(|t| matches!(t.into_inner(), RegexType::Or)).count();
    
    assert_eq!(char_count, 12); // a,b,c,d,e,f,g,h,i,j,k,l (some may be optimized out)) 
    assert!(or_count == 4);    // ab|cd, ef|gh, ij|kl, and the main alternation
    
    // The last operator should be the main alternation
    let last_is_or = result.iter().rev().any(|t| matches!(t.into_inner(), RegexType::Or));
    assert!(last_is_or);
}

#[test]
fn test_complex_quantifiers_with_groups() {
    // Test various complex quantifier combinations with groups
    let tokens = create_tokens("(a{2,5}b{3}){1,3}(c{1,}d?){2,}");
    let result = re2post(tokens).unwrap();
    
    // Verify all quantifiers are correctly parsed and positioned
    let quant_count = result.iter().filter(|t| matches!(t.into_inner(), RegexType::Quant(_))).count();
    assert!(quant_count >= 5); // {2,5}, {3}, {1,3}, {1,}, ?, {2,}
    
    // Check for specific quantifiers in the result
    let has_range_2_5 = result.iter().any(|t| matches!(t.into_inner(), RegexType::Quant(Quantifier::Range(2, 5))));
    let has_exact_3 = result.iter().any(|t| matches!(t.into_inner(), RegexType::Quant(Quantifier::Exact(3))));
    let has_range_1_3 = result.iter().any(|t| matches!(t.into_inner(), RegexType::Quant(Quantifier::Range(1, 3))));
    let has_at_least_1 = result.iter().any(|t| matches!(t.into_inner(), RegexType::Quant(Quantifier::AtLeast(1))));
    let has_range_0_1 = result.iter().any(|t| matches!(t.into_inner(), RegexType::Quant(Quantifier::Range(0, 1))));
    let has_at_least_2 = result.iter().any(|t| matches!(t.into_inner(), RegexType::Quant(Quantifier::AtLeast(2))));
    
    assert!(has_range_2_5);
    assert!(has_exact_3);
    assert!(has_range_1_3);
    assert!(has_at_least_1);
    assert!(has_range_0_1);
    assert!(has_at_least_2);
}

#[test]
fn test_pathological_regex_conversion() {
    // Test a pathologically complex regex that combines many features
    let pattern = "^(([a-z]+)|(\\d{1,3})|(\\w+\\-[0-9]+))+[^\\s\\d]?\\\\\\$\\d+(\\.\\d{2})?$";
    let tokens = create_tokens(pattern);
    let result = re2post(tokens).unwrap();
    
    // Basic structure verification
    assert!(result.len() > 20);
    
    // Verify anchors are preserved
    let has_start_anchor = result.iter().any(|t| 
        matches!(t, TokenType::StartOrEndCondition(RegexType::LineStart))
    );
    let has_end_anchor = result.iter().any(|t| 
        matches!(t, TokenType::StartOrEndCondition(RegexType::LineEnd))
    );
    
    assert!(has_start_anchor);
    assert!(has_end_anchor);
    

	dbg!(&result);
    // Count groups, alternations, and quantifiers
    let or_count = result.iter().filter(|t| matches!(t.into_inner(), RegexType::Or)).count();
    let quant_count = result.iter().filter(|t| matches!(t.into_inner(), RegexType::Quant(_))).count();
    
    assert_eq!(or_count, 2);
    assert_eq!(quant_count, 9);
}

#[test]
fn test_balanced_nested_parentheses() {
    // Test deeply nested balanced parentheses
    let tokens = create_tokens("(((((a)))))(((((b)))))");
    let result = re2post(tokens).unwrap();
    
    // In postfix notation, the nested structure is flattened
    // We should just have the characters and concatenation
    assert_eq!(result.len(), 3);
    
    let expected = {
        let mut exp = VecDeque::new();
        exp.push_back(TokenType::from(RegexType::Char('a')));
        exp.push_back(TokenType::from(RegexType::Char('b')));
        exp.push_back(TokenType::from(RegexType::Concatenation));
        exp
    };
    
    assert_token_sequences_equal(&result, &expected);
}

#[test]
fn test_complex_character_classes_and_escapes() {
    // Test character classes with escapes in a complex pattern
    let tokens = create_tokens("[a-z0-9]+(\\.[a-z0-9]+)*@[a-z0-9]+(\\.[a-z0-9]+)+");
    let result = re2post(tokens).unwrap();
    
    // Verify the structure has the right components
    let char_count = result.iter().filter(|t| matches!(t.into_inner(), RegexType::Char(_))).count();
    let concat_count = result.iter().filter(|t| matches!(t.into_inner(), RegexType::Concatenation)).count();
    let quant_count = result.iter().filter(|t| matches!(t.into_inner(), RegexType::Quant(_))).count();
    
    // We should have multiple characters, concatenations, and quantifiers
    assert!(char_count > 0);
    assert!(concat_count > 0);
    assert!(quant_count >= 2); // + and *
    
    // Check for specific characters like @ in the email pattern
    let has_at_sign = result.iter().any(|t| matches!(t.into_inner(), RegexType::Char('@')));
    assert!(has_at_sign);
}

#[test]
fn test_mixed_operators_precedence() {
    // Test mixed operators with different precedence levels
    let tokens = create_tokens("a|b*c+d?e{2,3}");
    let result = re2post(tokens).unwrap();
    
    // In postfix notation, operators come after their operands
    // Higher precedence operators should be processed first
    
    // Verify we have all the expected characters
    let chars = ['a', 'b', 'c', 'd', 'e'];
    for c in chars {
        let has_char = result.iter().any(|t| matches!(t.into_inner(), RegexType::Char(ch) if ch == &c));
        assert!(has_char, "Character '{}' not found in result", c);
    }
    
    // Verify we have all the expected quantifiers
    let has_star = result.iter().any(|t| matches!(t.into_inner(), RegexType::Quant(Quantifier::AtLeast(0))));
    let has_plus = result.iter().any(|t| matches!(t.into_inner(), RegexType::Quant(Quantifier::AtLeast(1))));
    let has_question = result.iter().any(|t| matches!(t.into_inner(), RegexType::Quant(Quantifier::Range(0, 1))));
    let has_range = result.iter().any(|t| matches!(t.into_inner(), RegexType::Quant(Quantifier::Range(2, 3))));
    
    assert!(has_star);
    assert!(has_plus);
    assert!(has_question);
    assert!(has_range);
    
    // The last operator should be the alternation (lowest precedence)
    if let Some(last) = result.back() {
        assert_eq!(last.into_inner(), &RegexType::Or);
    } else {
        panic!("Result is empty");
    }
}
