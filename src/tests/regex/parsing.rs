use crate::regex::parsing::{RegexType, TokenType, CharacterClass, Quantifier};
use crate::{Regex, Utils, ParsingError};
use std::collections::{HashSet, VecDeque};

// ==============================================
// 1. REGEXTYPE TESTS
// ==============================================

#[test]
fn test_regex_type_char_creation() {
    let char_type = RegexType::Char('a');
    assert_eq!(char_type, RegexType::Char('a'));
    assert_ne!(char_type, RegexType::Char('b'));
}

#[test]
fn test_regex_type_char_matching() {
    let char_type = RegexType::Char('a');
    assert!(char_type.match_(&'a'));
    assert!(!char_type.match_(&'b'));
}

#[test]
fn test_regex_type_char_extraction() {
    let char_type = RegexType::Char('a');
    assert_eq!(char_type.char(), Some('a'));
    
    let other_type = RegexType::Or;
    assert_eq!(other_type.char(), None);
}

#[test]
fn test_regex_type_display() {
    assert_eq!(RegexType::Char('a').to_string(), "a");
    assert_eq!(RegexType::LineStart.to_string(), "^");
    assert_eq!(RegexType::LineEnd.to_string(), "$");
    assert_eq!(RegexType::OpenParenthesis.to_string(), "(");
    assert_eq!(RegexType::CloseParenthesis.to_string(), ")");
    assert_eq!(RegexType::Or.to_string(), "|");
    assert_eq!(RegexType::Concatenation.to_string(), "&");
    assert_eq!(RegexType::Quant(Quantifier::Exact(3)).to_string(), "{3}");
}

#[test]
fn test_regex_type_precedence() {
    assert_eq!(RegexType::Quant(Quantifier::Exact(3)).precedence(), 3);
    assert_eq!(RegexType::Concatenation.precedence(), 2);
    assert_eq!(RegexType::Or.precedence(), 1);
    assert_eq!(RegexType::Char('a').precedence(), 0);
    assert_eq!(RegexType::LineStart.precedence(), 0);
}

#[test]
fn test_regex_type_to_token_type() {
    assert!(matches!(RegexType::Char('a').type_(), TokenType::Literal(_)));
    assert!(matches!(RegexType::OpenParenthesis.type_(), TokenType::OpenParenthesis(_)));
    assert!(matches!(RegexType::CloseParenthesis.type_(), TokenType::CloseParenthesis(_)));
    assert!(matches!(RegexType::Quant(Quantifier::Exact(3)).type_(), TokenType::UnaryOperator(_)));
    assert!(matches!(RegexType::Or.type_(), TokenType::BinaryOperator(_)));
    assert!(matches!(RegexType::Concatenation.type_(), TokenType::BinaryOperator(_)));
    assert!(matches!(RegexType::LineStart.type_(), TokenType::StartOrEndCondition(_)));
    assert!(matches!(RegexType::LineEnd.type_(), TokenType::StartOrEndCondition(_)));
}

// ==============================================
// 2. TOKENTYPE TESTS
// ==============================================

#[test]
fn test_token_type_from_regex_type() {
    assert!(matches!(TokenType::from(RegexType::Char('a')), TokenType::Literal(_)));
    assert!(matches!(TokenType::from(RegexType::OpenParenthesis), TokenType::OpenParenthesis(_)));
    assert!(matches!(TokenType::from(RegexType::CloseParenthesis), TokenType::CloseParenthesis(_)));
    assert!(matches!(TokenType::from(RegexType::Quant(Quantifier::Exact(3))), TokenType::UnaryOperator(_)));
    assert!(matches!(TokenType::from(RegexType::Or), TokenType::BinaryOperator(_)));
    assert!(matches!(TokenType::from(RegexType::Concatenation), TokenType::BinaryOperator(_)));
}

#[test]
fn test_token_type_into_inner() {
    let token = TokenType::from(RegexType::Char('a'));
    assert_eq!(*token.into_inner(), RegexType::Char('a'));
}

#[test]
fn test_token_type_into_owned_inner() {
    let token = TokenType::from(RegexType::Char('a'));
    assert_eq!(token.into_owned_inner(), RegexType::Char('a'));
}

#[test]
fn test_token_type_precedence() {
    let token = TokenType::from(RegexType::Quant(Quantifier::Exact(3)));
    assert_eq!(token.precedence(), 3);
    
    let token = TokenType::from(RegexType::Concatenation);
    assert_eq!(token.precedence(), 2);
    
    let token = TokenType::from(RegexType::Or);
    assert_eq!(token.precedence(), 1);
}

#[test]
fn test_token_type_display() {
    let token = TokenType::from(RegexType::Char('a'));
    assert_eq!(token.to_string(), "a");
    
    let token = TokenType::from(RegexType::Or);
    assert_eq!(token.to_string(), "|");
}

#[test]
fn test_need_concatenation_with_literal_and_literal() {
    let token = TokenType::from(RegexType::Char('a'));
    assert!(token.need_concatenation_with(&RegexType::Char('b')));
}

#[test]
fn test_need_concatenation_with_literal_and_open_paren() {
    let token = TokenType::from(RegexType::Char('a'));
    assert!(token.need_concatenation_with(&RegexType::OpenParenthesis));
}

#[test]
fn test_need_concatenation_with_close_paren_and_literal() {
    let token = TokenType::from(RegexType::CloseParenthesis);
    assert!(token.need_concatenation_with(&RegexType::Char('a')));
}

#[test]
fn test_need_concatenation_with_close_paren_and_open_paren() {
    let token = TokenType::from(RegexType::CloseParenthesis);
    assert!(token.need_concatenation_with(&RegexType::OpenParenthesis));
}

#[test]
fn test_need_concatenation_with_unary_op_and_literal() {
    let token = TokenType::from(RegexType::Quant(Quantifier::Exact(3)));
    assert!(token.need_concatenation_with(&RegexType::Char('a')));
}

#[test]
fn test_need_concatenation_with_unary_op_and_open_paren() {
    let token = TokenType::from(RegexType::Quant(Quantifier::Exact(3)));
    assert!(token.need_concatenation_with(&RegexType::OpenParenthesis));
}

#[test]
fn test_need_concatenation_with_false_cases() {
    // Some cases where concatenation is not needed
    let binary_op = TokenType::from(RegexType::Or);
    assert!(!binary_op.need_concatenation_with(&RegexType::Char('a')));
    
    let open_paren = TokenType::from(RegexType::OpenParenthesis);
    assert!(!open_paren.need_concatenation_with(&RegexType::Char('a')));
    
    let literal = TokenType::from(RegexType::Char('a'));
    assert!(!literal.need_concatenation_with(&RegexType::Or));
}

// ==============================================
// 3. CHARACTER CLASS TESTS
// ==============================================

#[test]
fn test_character_class_new() {
    let class = CharacterClass::new();
    assert_eq!(class.chars.len(), 0);
    assert_eq!(class.negated, false);
}

#[test]
fn test_character_class_add_char() {
    let mut class = CharacterClass::new();
    class.add_char('a');
    assert!(class.contains(&'a'));
    
    // Adding same char again shouldn't duplicate
    class.add_char('a');
    assert_eq!(class.chars.len(), 1);
}

#[test]
fn test_character_class_add_range_valid() {
    let mut class = CharacterClass::new();
    let result = class.add_range('a', 'c');
    assert!(result.is_ok());
    assert!(class.contains(&'a'));
    assert!(class.contains(&'b'));
    assert!(class.contains(&'c'));
}

#[test]
fn test_character_class_add_range_invalid() {
    let mut class = CharacterClass::new();
    let result = class.add_range('c', 'a');
    assert!(result.is_err());

	let err = result.unwrap_err();
    assert!(err.message().contains("negative range in character class"));
}

#[test]
fn test_character_class_from_single() {
    let class = CharacterClass::from_single('a');
    assert!(class.contains(&'a'));
    assert_eq!(class.chars.len(), 1);
}

#[test]
fn test_character_class_from_range() {
    let class = CharacterClass::from_range('0', '9');
    assert!(class.contains(&'0'));
    assert!(class.contains(&'5'));
    assert!(class.contains(&'9'));
    assert_eq!(class.chars.len(), 10);
}

#[test]
fn test_character_class_negated() {
    let class = CharacterClass::from_single('a').negated();
    assert!(class.negated);
}

#[test]
fn test_character_class_from_negated() {
    let original = CharacterClass::from_single('a');
    let negated = CharacterClass::from_negated(original);
    assert!(negated.negated);
    assert!(negated.chars.contains(&'a'));
}

#[test]
fn test_character_class_predefined_digit() {
    let class = CharacterClass::digit();
    assert!(class.contains(&'0'));
    assert!(class.contains(&'9'));
    assert_eq!(class.chars.len(), 10);
}

#[test]
fn test_character_class_predefined_non_digit() {
    let class = CharacterClass::non_digit();
    assert!(!class.contains(&'0'));
    assert!(class.negated);
}

#[test]
fn test_character_class_predefined_word_char() {
    let class = CharacterClass::word_char();
    assert!(class.contains(&'a'));
    assert!(class.contains(&'Z'));
    assert!(class.contains(&'0'));
    assert!(class.contains(&'_'));
}

#[test]
fn test_character_class_predefined_non_word_char() {
    let class = CharacterClass::non_word_char();

	dbg!(&class);

    assert!(!class.contains(&'a'));
    assert!(class.negated);
}

#[test]
fn test_character_class_predefined_whitespace() {
    let class = CharacterClass::whitespace();
    assert!(class.contains(&' '));
    assert!(class.contains(&'\t'));
    assert!(class.contains(&'\n'));
}

#[test]
fn test_character_class_predefined_non_whitespace() {
    let class = CharacterClass::non_whitespace();
    assert!(!class.contains(&' '));
    assert!(class.negated);
}

#[test]
fn test_character_class_from_shorthand_valid() {
    assert!(CharacterClass::from_shorthand('d').is_ok());
    assert!(CharacterClass::from_shorthand('D').is_ok());
    assert!(CharacterClass::from_shorthand('w').is_ok());
    assert!(CharacterClass::from_shorthand('W').is_ok());
    assert!(CharacterClass::from_shorthand('s').is_ok());
    assert!(CharacterClass::from_shorthand('S').is_ok());
}

#[test]
fn test_character_class_from_shorthand_invalid() {
    let result = CharacterClass::from_shorthand('x');
    assert!(result.is_err());

	let err = result.unwrap_err();
    assert!(err.message().contains("Unknown shorthand class '\\x'"));
}

#[test]
fn test_character_class_all() {
    let all_chars = CharacterClass::all();
    assert_eq!(all_chars.len(), 128);
    assert!(all_chars.contains(&'a'));
    assert!(all_chars.contains(&'0'));
    assert!(all_chars.contains(&'\n'));
}

#[test]
fn test_character_class_push_into_tokens() {
    let mut tokens = VecDeque::new();
    let class = CharacterClass::from_single('a');
    
	assert!(class.push_into_tokens(&mut tokens).is_ok());
    
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0], RegexType::OpenParenthesis);
    assert_eq!(tokens[1], RegexType::Char('a'));
    assert_eq!(tokens[2], RegexType::CloseParenthesis);
}

#[test]
fn test_character_class_parse_simple() {
    let input = "abc]";
    let mut chars = input.chars();
    let result = CharacterClass::parse(&mut chars);
    
    assert!(result.is_ok());
    let class = result.unwrap();
    assert!(class.contains(&'a'));
    assert!(class.contains(&'b'));
    assert!(class.contains(&'c'));
    assert_eq!(class.chars.len(), 3);
}

#[test]
fn test_character_class_parse_negated() {
    let input = "^abc]";
    let mut chars = input.chars();
    let result = CharacterClass::parse(&mut chars);
    
    assert!(result.is_ok());

	let class = result.unwrap();
	assert!(class.negated);

	// Don't contains banned chars
    assert!(!class.contains(&'a'));
    assert!(!class.contains(&'b'));
    assert!(!class.contains(&'c'));

	// Contains other chars
	assert!(class.contains(&'d'));
	assert!(class.contains(&'1'));
}

#[test]
fn test_character_class_parse_range() {
    let input = "a-c]";
    let mut chars = input.chars();
    let result = CharacterClass::parse(&mut chars);
    
    assert!(result.is_ok());
    let class = result.unwrap();
    assert!(class.contains(&'a'));
    assert!(class.contains(&'b'));
    assert!(class.contains(&'c'));
}

#[test]
fn test_character_class_parse_dash_at_start() {
    let input = "-abc]";
    let mut chars = input.chars();
    let result = CharacterClass::parse(&mut chars);
    
    assert!(result.is_ok());
    let class = result.unwrap();
    assert!(class.contains(&'-'));
    assert!(class.contains(&'a'));
}

#[test]
fn test_character_class_parse_dash_at_end() {
    let input = "abc-]";
    let mut chars = input.chars();
    let result = CharacterClass::parse(&mut chars);
    
    assert!(result.is_ok());
    let class = result.unwrap();
    assert!(class.contains(&'-'));
    assert!(class.contains(&'a'));
}

#[test]
fn test_character_class_parse_escaped_chars() {
    let input = "\\]\\-\\^]";
    let mut chars = input.chars();
    let result = CharacterClass::parse(&mut chars);
    
    assert!(result.is_ok());
    let class = result.unwrap();
    assert!(class.contains(&']'));
    assert!(class.contains(&'-'));
    assert!(class.contains(&'^'));
}

#[test]
fn test_character_class_parse_unclosed() {
    let input = "abc";
    let mut chars = input.chars();
    let result = CharacterClass::parse(&mut chars);
    
    assert!(result.is_err());
	let err = result.unwrap_err();
    assert!(err.message().contains("Unclosed character class"));
}

#[test]
fn test_character_class_parse_escape_at_end() {
    let input = "abc\\";
    let mut chars = input.chars();
    let result = CharacterClass::parse(&mut chars);
    
    assert!(result.is_err());

	let err = result.unwrap_err();
    assert!(err.message().contains("Escape sequence at end of character class"));
}

// ==============================================
// 4. QUANTIFIER TESTS
// ==============================================

#[test]
fn test_quantifier_display_exact() {
    let quant = Quantifier::Exact(5);
    assert_eq!(quant.to_string(), "{5}");
}

#[test]
fn test_quantifier_display_at_least() {
    let quant = Quantifier::AtLeast(3);
    assert_eq!(quant.to_string(), "{3,}");
}

#[test]
fn test_quantifier_display_range() {
    let quant = Quantifier::Range(2, 5);
    assert_eq!(quant.to_string(), "{2,5}");
}

// ==============================================
// 5. REGEX PARSING METHODS TESTS
// ==============================================

#[test]
fn test_add_concatenation_empty() {
    let tokens: VecDeque<RegexType> = VecDeque::new();
    let result = Regex::add_concatenation(tokens);
    assert_eq!(result.len(), 0);
}

#[test]
fn test_add_concatenation_single_token() {
    let mut tokens = VecDeque::new();
    tokens.push_back(RegexType::Char('a'));
    
    let result = Regex::add_concatenation(tokens);
    assert_eq!(result.len(), 1);
    assert!(matches!(result[0], TokenType::Literal(_)));
}

#[test]
fn test_add_concatenation_literals() {
    let mut tokens = VecDeque::new();
    tokens.push_back(RegexType::Char('a'));
    tokens.push_back(RegexType::Char('b'));
    
    let result = Regex::add_concatenation(tokens);
    assert_eq!(result.len(), 3);
    assert!(matches!(result[0], TokenType::Literal(_)));
    assert!(matches!(result[1], TokenType::BinaryOperator(_)));
    assert!(matches!(result[2], TokenType::Literal(_)));
}

#[test]
fn test_add_concatenation_with_operators() {
    let mut tokens = VecDeque::new();
    tokens.push_back(RegexType::Char('a'));
    tokens.push_back(RegexType::Or);
    tokens.push_back(RegexType::Char('b'));
    
    let result = Regex::add_concatenation(tokens);
    assert_eq!(result.len(), 3);
    // No concatenation needed between literal and operator
    assert!(matches!(result[0], TokenType::Literal(_)));
    assert!(matches!(result[1], TokenType::BinaryOperator(_)));
    assert!(matches!(result[2], TokenType::Literal(_)));
}

#[test]
fn test_add_concatenation_with_groups() {
    let mut tokens = VecDeque::new();
    tokens.push_back(RegexType::Char('a'));
    tokens.push_back(RegexType::OpenParenthesis);
    tokens.push_back(RegexType::Char('b'));
    tokens.push_back(RegexType::CloseParenthesis);
    tokens.push_back(RegexType::Char('c'));
    
    let result = Regex::add_concatenation(tokens);
    assert_eq!(result.len(), 7);
    assert!(matches!(result[0], TokenType::Literal(_)));
    assert!(matches!(result[1], TokenType::BinaryOperator(_))); // Concat between 'a' and '('
    assert!(matches!(result[2], TokenType::OpenParenthesis(_)));
    assert!(matches!(result[3], TokenType::Literal(_)));
    assert!(matches!(result[4], TokenType::CloseParenthesis(_)));
    assert!(matches!(result[5], TokenType::BinaryOperator(_))); // Concat between ')' and 'c'
    assert!(matches!(result[6], TokenType::Literal(_)));
}

#[test]
fn test_tokens_simple_literal() {
    let result = Regex::tokens("abc").unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], RegexType::Char('a'));
    assert_eq!(result[1], RegexType::Char('b'));
    assert_eq!(result[2], RegexType::Char('c'));
}

#[test]
fn test_tokens_with_operators() {
    let result = Regex::tokens("a|b").unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], RegexType::Char('a'));
    assert_eq!(result[1], RegexType::Or);
    assert_eq!(result[2], RegexType::Char('b'));
}

#[test]
fn test_tokens_with_parentheses() {
    let result = Regex::tokens("(a)").unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], RegexType::OpenParenthesis);
    assert_eq!(result[1], RegexType::Char('a'));
    assert_eq!(result[2], RegexType::CloseParenthesis);
}

#[test]
fn test_tokens_with_quantifiers() {
    let result = Regex::tokens("a*b+c?").unwrap();

	dbg!(&result);
    assert_eq!(result.len(), 6);
    assert_eq!(result[0], RegexType::Char('a'));
    assert!(matches!(result[1], RegexType::Quant(Quantifier::AtLeast(0))));
    assert_eq!(result[2], RegexType::Char('b'));
    assert!(matches!(result[3], RegexType::Quant(Quantifier::AtLeast(1))));
    assert_eq!(result[4], RegexType::Char('c'));
    assert!(matches!(result[5], RegexType::Quant(Quantifier::Range(0, 1))));
}

#[test]
fn test_tokens_with_anchors() {
    let result = Regex::tokens("^abc$").unwrap();
    assert_eq!(result.len(), 5);
    assert_eq!(result[0], RegexType::LineStart);
    assert_eq!(result[1], RegexType::Char('a'));
    assert_eq!(result[2], RegexType::Char('b'));
    assert_eq!(result[3], RegexType::Char('c'));
    assert_eq!(result[4], RegexType::LineEnd);
}

#[test]
fn test_tokens_with_explicit_quantifier() {
    let result = Regex::tokens("a{3}").unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], RegexType::Char('a'));
    assert!(matches!(result[1], RegexType::Quant(Quantifier::Exact(3))));
}

#[test]
fn test_tokens_with_at_least_quantifier() {
    let result = Regex::tokens("a{2,}").unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], RegexType::Char('a'));
    assert!(matches!(result[1], RegexType::Quant(Quantifier::AtLeast(2))));
}

#[test]
fn test_tokens_with_range_quantifier() {
    let result = Regex::tokens("a{2,5}").unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], RegexType::Char('a'));
    assert!(matches!(result[1], RegexType::Quant(Quantifier::Range(2, 5))));
}

#[test]
fn test_tokens_with_invalid_quantifier_range() {
    let result = Regex::tokens("a{5,2}");
    assert!(result.is_err());
	
	let err = result.unwrap_err();
    assert!(err.message().contains("Invalid quantifier range: min > max"));
}

#[test]
fn test_tokens_with_unclosed_quantifier() {
    let result = Regex::tokens("a{5");
    assert!(result.is_err());
	
	let err = result.unwrap_err();
    assert!(err.message().contains("Unclosed quantifier"));
}

#[test]
fn test_tokens_with_invalid_character_in_quantifier() {
    let result = Regex::tokens("a{5a}");
    assert!(result.is_err());
	
	let err = result.unwrap_err();
    assert!(err.message().contains("Invalid character in quantifier"));
}

#[test]
fn test_tokens_with_character_class() {
    let result = Regex::tokens("[abc]").unwrap();

    assert_eq!(result.len(), 7); // Open, a, |, b, |, c, Close
    assert_eq!(result[0], RegexType::OpenParenthesis);
    // The middle will have characters with OR operators
    assert_eq!(result[result.len()-1], RegexType::CloseParenthesis);
}

#[test]
fn test_tokens_with_negated_character_class() {
    let result = Regex::tokens("[^abc]").unwrap();
    // This will create a complex structure with all ASCII chars except a, b, c
    assert!(result.len() > 5);
    assert_eq!(result[0], RegexType::OpenParenthesis);
    assert_eq!(result[result.len()-1], RegexType::CloseParenthesis);
}

#[test]
fn test_tokens_with_character_range() {
    let result = Regex::tokens("[a-c]").unwrap();

    assert_eq!(result.len(), 7); // Open, a, |, b, |, c, Close
    assert_eq!(result[0], RegexType::OpenParenthesis);
    assert_eq!(result[result.len()-1], RegexType::CloseParenthesis);
}

#[test]
fn test_tokens_with_unclosed_character_class() {
    let result = Regex::tokens("[abc");
    assert!(result.is_err());
	
	let err = result.unwrap_err();
    assert!(err.message().contains("Unclosed character class"));
}

#[test]
fn test_tokens_with_string() {
    let result = Regex::tokens("\"abc\"").unwrap();
    assert_eq!(result.len(), 5);
    assert_eq!(result[0], RegexType::OpenParenthesis);
    assert_eq!(result[1], RegexType::Char('a'));
    assert_eq!(result[2], RegexType::Char('b'));
    assert_eq!(result[3], RegexType::Char('c'));
    assert_eq!(result[4], RegexType::CloseParenthesis);
}

#[test]
fn test_tokens_with_unclosed_string() {
    let result = Regex::tokens("\"abc");
    assert!(result.is_err());

	let err = result.unwrap_err();
    assert!(err.message().contains("Unclosed string"));
}

#[test]
fn test_tokens_with_escaped_characters() {
    let result = Regex::tokens("\\(\\)\\[\\]\\{\\}\\^\\$").unwrap();
    assert_eq!(result.len(), 8);
    assert_eq!(result[0], RegexType::Char('('));
    assert_eq!(result[1], RegexType::Char(')'));
    assert_eq!(result[2], RegexType::Char('['));
    assert_eq!(result[3], RegexType::Char(']'));
    assert_eq!(result[4], RegexType::Char('{'));
    assert_eq!(result[5], RegexType::Char('}'));
    assert_eq!(result[6], RegexType::Char('^'));
    assert_eq!(result[7], RegexType::Char('$'));
}

#[test]
fn test_tokens_with_backslash_shorthand_classes() {
    let result = Regex::tokens("\\d\\w\\s").unwrap();
    // Each shorthand class gets expanded to a character class
    assert!(result.len() > 3);
}

#[test]
fn test_tokens_with_backslash_negated_shorthand_classes() {
    let result = Regex::tokens("\\D\\W\\S").unwrap();
    // Each negated shorthand class gets expanded to a character class
    assert!(result.len() > 3);
}

#[test]
fn test_tokens_with_wildcard() {
    let mut result = Vec::from(Regex::tokens(".").unwrap());

    // The dot character should expand to a character class that matches any character

	// parenthesis + 127(char + Or) + last char + parenthesis
	assert_eq!(result.len(), 1 + (127 * 2) + 1 + 1);
    // The first token should be an opening parenthesis for the character class
    assert_eq!(result[0], RegexType::OpenParenthesis);
    // The last token should be a closing parenthesis for the character class
    assert_eq!(result[result.len() - 1], RegexType::CloseParenthesis);

    // Check that the wildcard character class contains all 127 ASCII characters
    let mut expected_chars = HashSet::new();
    for i in 0..=127 {
        if let Some(c) = char::from_u32(i) {
            expected_chars.insert(c);
        }
    }
    
	// First token should be parenthesis
	assert!(matches!(result[0], RegexType::OpenParenthesis));
	// Last token should be parenthesis
	assert!(matches!(result[result.len() - 1], RegexType::CloseParenthesis));

    // Extract all character literals from the tokens
    let mut found_chars = HashSet::new();
    for token in &result[1..result.len() - 1] {
        if let RegexType::Char(c) = token {
            found_chars.insert(*c);
        } else {
			assert_eq!(token, &RegexType::Or);
		}
    }
    
    // Verify that all expected characters are present
    for c in expected_chars {
        assert!(found_chars.contains(&c), "Character '{}' (ASCII {}) not found in wildcard expansion", c, c as u32);
    }
}

#[test]
fn test_add_backslash() {
    let mut tokens = VecDeque::new();
    let mut chars = "d".chars();
    
    Regex::add_backslash(&mut tokens, &mut chars);
    // \d should expand to a digit character class
    assert!(tokens.len() > 2);
}

#[test]
fn test_add_string() {
    let mut tokens = VecDeque::new();
    let mut chars = "abc\"".chars();
    
    let result = Regex::add_string(&mut tokens, &mut chars);

	assert!(result.is_ok());

    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0], RegexType::OpenParenthesis);
    assert_eq!(tokens[1], RegexType::Char('a'));
    assert_eq!(tokens[2], RegexType::Char('b'));
    assert_eq!(tokens[3], RegexType::Char('c'));
    assert_eq!(tokens[4], RegexType::CloseParenthesis);
}

#[test]
fn test_add_character_class() {
    let mut tokens = VecDeque::new();
    let mut chars = "abc]".chars();
    
    let result = Regex::add_character_class(&mut tokens, &mut chars);
    assert!(result.is_ok());
    assert!(tokens.len() > 2);
}

#[test]
fn test_add_quantifier_exact() {
    let mut tokens = VecDeque::new();
    let mut chars = "5}".chars();
    
    let result = Regex::add_quantifier(&mut tokens, &mut chars);
    assert!(result.is_ok());
    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0], RegexType::Quant(Quantifier::Exact(5))));
}

#[test]
fn test_add_quantifier_at_least() {
    let mut tokens = VecDeque::new();
    let mut chars = "5,}".chars();
    
    let result = Regex::add_quantifier(&mut tokens, &mut chars);
    assert!(result.is_ok());
    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0], RegexType::Quant(Quantifier::AtLeast(5))));
}

#[test]
fn test_add_quantifier_range() {
    let mut tokens = VecDeque::new();
    let mut chars = "2,5}".chars();
    
    let result = Regex::add_quantifier(&mut tokens, &mut chars);
    assert!(result.is_ok());
    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0], RegexType::Quant(Quantifier::Range(2, 5))));
}

#[test]
fn test_into_type() {
    assert_eq!(Regex::into_type('a'), RegexType::Char('a'));
    assert_eq!(Regex::into_type('('), RegexType::OpenParenthesis);
    assert_eq!(Regex::into_type(')'), RegexType::CloseParenthesis);
    assert_eq!(Regex::into_type('|'), RegexType::Or);
    assert_eq!(Regex::into_type('^'), RegexType::LineStart);
    assert_eq!(Regex::into_type('$'), RegexType::LineEnd);
    assert!(matches!(Regex::into_type('*'), RegexType::Quant(Quantifier::AtLeast(0))));
    assert!(matches!(Regex::into_type('+'), RegexType::Quant(Quantifier::AtLeast(1))));
    assert!(matches!(Regex::into_type('?'), RegexType::Quant(Quantifier::Range(0, 1))));
}

// ==============================================
// 6. EDGE CASES AND COMPLEX PATTERNS
// ==============================================

#[test]
fn test_very_large_quantifier() {
    let result = Regex::tokens("a{999999}").unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], RegexType::Char('a'));
    assert!(matches!(result[1], RegexType::Quant(Quantifier::Exact(999999))));
}

#[test]
fn test_complex_nested_expressions() {
    let result = Regex::tokens("((a|b)*c)|d{3}").unwrap();
    assert!(result.len() > 10);
    // Validate the structure is correct
    assert_eq!(result[0], RegexType::OpenParenthesis);
    assert_eq!(result[1], RegexType::OpenParenthesis);
    // ... more detailed assertions could be added
}

#[test]
fn test_unicode_characters() {
    let result = Regex::tokens("日本語").unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], RegexType::Char('日'));
    assert_eq!(result[1], RegexType::Char('本'));
    assert_eq!(result[2], RegexType::Char('語'));
}

#[test]
fn test_empty_groups() {
    let result = Regex::tokens("()").unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], RegexType::OpenParenthesis);
    assert_eq!(result[1], RegexType::CloseParenthesis);
}

#[test]
fn test_quantifiers_applied_to_empty_groups() {
    let result = Regex::tokens("(){3}").unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], RegexType::OpenParenthesis);
    assert_eq!(result[1], RegexType::CloseParenthesis);
    assert!(matches!(result[2], RegexType::Quant(Quantifier::Exact(3))));
}

#[test]
fn test_multiple_alternations() {
    let result = Regex::tokens("a|b|c|d").unwrap();
    assert_eq!(result.len(), 7);
    assert_eq!(result[0], RegexType::Char('a'));
    assert_eq!(result[1], RegexType::Or);
    assert_eq!(result[2], RegexType::Char('b'));
    assert_eq!(result[3], RegexType::Or);
    assert_eq!(result[4], RegexType::Char('c'));
    assert_eq!(result[5], RegexType::Or);
    assert_eq!(result[6], RegexType::Char('d'));
}

#[test]
fn test_multiple_consecutive_operators() {
    // This is technically invalid in most regex engines, but we're testing the parser
    let result = Regex::tokens("a**");
    assert!(result.is_ok()); // It might parse, even if not meaningful
}

#[test]
fn test_escaped_line_boundaries() {
    let result = Regex::tokens("\\^\\$").unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], RegexType::Char('^'));
    assert_eq!(result[1], RegexType::Char('$'));
}

#[test]
fn test_quantifiers_applied_to_anchors() {
    // This is technically invalid in most regex engines, but we're testing the parser
    let result = Regex::tokens("^{3}");
    assert!(result.is_ok()); // Our parser might allow this, even if it's semantically invalid
}

#[test]
fn test_very_long_regex() {
    let long_pattern = "a".repeat(1000);
    let result = Regex::tokens(&long_pattern).unwrap();
    assert_eq!(result.len(), 1000);
    assert_eq!(result[0], RegexType::Char('a'));
    assert_eq!(result[999], RegexType::Char('a'));
}

#[test]
fn test_character_classes_with_many_ranges() {
    let result = Regex::tokens("[a-zA-Z0-9_-]").unwrap();
    assert!(result.len() > 10);
    assert_eq!(result[0], RegexType::OpenParenthesis);
    assert_eq!(result[result.len()-1], RegexType::CloseParenthesis);
}
#[test]
fn test_escape_sequences() {
    // Test common escape sequences
    let result = Regex::tokens("\\n\\r\\t").unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], RegexType::Char('\n'));
    assert_eq!(result[1], RegexType::Char('\r'));
    assert_eq!(result[2], RegexType::Char('\t'));
}

#[test]
fn test_mixed_escape_sequences() {
    // Test escape sequences mixed with regular characters
    let result = Regex::tokens("a\\nb\\tc\\r").unwrap();
    assert_eq!(result.len(), 6);
    assert_eq!(result[0], RegexType::Char('a'));
    assert_eq!(result[1], RegexType::Char('\n'));
    assert_eq!(result[2], RegexType::Char('b'));
    assert_eq!(result[3], RegexType::Char('\t'));
    assert_eq!(result[4], RegexType::Char('c'));
    assert_eq!(result[5], RegexType::Char('\r'));
}

#[test]
fn test_other_escape_sequences() {
    // Test other common escape sequences
    let result = Regex::tokens("\\f\\v\\0").unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], RegexType::Char('\u{000C}')); // Form feed
    assert_eq!(result[1], RegexType::Char('\u{000B}')); // Vertical tab
    assert_eq!(result[2], RegexType::Char('\0'));       // Null character
}

#[test]
fn test_escape_sequences_in_character_class() {
    // Test escape sequences inside character classes
    let class = CharacterClass::parse(&mut "\\n\\r\\t]".chars()).unwrap();

	assert!(class.contains(&'\n'));
	assert!(class.contains(&'\r'));
	assert!(class.contains(&'\t'));
}

// ==============================================
// 2. COMPLEX REGEX PARSING TESTS
// ==============================================

#[test]
fn test_complex_nested_groups() {
    // Test deeply nested parentheses with alternation and quantifiers
    let result = Regex::tokens("(a(b|c)*(d(e|f)+g)?)+").unwrap();
    
    // Verify structure without checking every token
    assert!(result.len() > 15);
    assert_eq!(result[0], RegexType::OpenParenthesis);
    
    // Find and verify some key elements
    let has_alternation = result.iter().any(|t| *t == RegexType::Or);
    let has_star = result.iter().any(|t| *t == RegexType::Quant(Quantifier::AtLeast(0)));
    let has_plus = result.iter().any(|t| *t == RegexType::Quant(Quantifier::AtLeast(1)));
    let has_question = result.iter().any(|t| *t == RegexType::Quant(Quantifier::Range(0, 1)));
    
    assert!(has_alternation);
    assert!(has_star);
    assert!(has_plus);
    assert!(has_question);
}

#[test]
fn test_complex_character_class_with_escapes_and_ranges() {
    // Test character class with ranges, escapes, and negation
    let result = Regex::tokens("[^a-z0-9\\n\\t\\-\\[\\]\\\\]").unwrap();
    
    // Verify the structure of the tokens
    // For character classes, tokens() should create a structure like (c1|c2|c3|...)
    // with alternation between all possible characters
    
    // First token should be open parenthesis
    assert_eq!(result[0], RegexType::OpenParenthesis);
    
    // Check for alternation operators between characters
    let or_count = result.iter().filter(|&t| *t == RegexType::Or).count();
    
    // Verify we have multiple characters with Or operators between them
    assert!(or_count > 0);
    
    // Last token should be close parenthesis
    assert_eq!(result[result.len() - 1], RegexType::CloseParenthesis);
    
    // Create and test the actual character class behavior
    let class = CharacterClass::parse(&mut "^a-z0-9\\n\\t\\-\\[\\]\\\\]".chars()).unwrap();
    
    // Test negation behavior
    assert!(!class.contains(&'a'));
    assert!(!class.contains(&'z'));
    assert!(!class.contains(&'5'));
    assert!(!class.contains(&'\n'));
    assert!(!class.contains(&'\t'));
    assert!(!class.contains(&'-'));
    assert!(!class.contains(&'['));
    assert!(!class.contains(&']'));
    assert!(!class.contains(&'\\'));
    
    // Test characters that should be included
    assert!(class.contains(&'A'));
    assert!(class.contains(&'Z'));
    assert!(class.contains(&'!'));
    assert!(class.contains(&' '));
}

#[test]
fn test_complex_quantifiers() {
    // Test various complex quantifier combinations
    let result = Regex::tokens("a{2,5}b{3}c{1,}d?e*f+").unwrap();
    
    // Verify all quantifiers are correctly parsed
    let mut found_quantifiers = 0;
    for token in &result {
        match token {
			RegexType::Quant(Quantifier::Range(0, 1)) => found_quantifiers += 1,
            RegexType::Quant(Quantifier::Range(min, max)) => {
                if *min == 2 && *max == 5 { found_quantifiers += 1; }
            },
            RegexType::Quant(Quantifier::Exact(n)) => {
                if *n == 3 { found_quantifiers += 1; }
            },
            RegexType::Quant(Quantifier::AtLeast(0)) => found_quantifiers += 1,
            RegexType::Quant(Quantifier::AtLeast(1)) => found_quantifiers += 1,
            _ => {}
        }
    }
    
    assert_eq!(found_quantifiers, 6);
}

#[test]
fn test_complex_anchors_and_escapes() {
    // Test line anchors with escaped metacharacters
    let result = Regex::tokens("^\\(\\[\\{\\*\\+\\?\\|\\\\\\$\\}\\]\\)$").unwrap();
    
    // Verify anchors
    assert_eq!(result[0], RegexType::LineStart);
    assert_eq!(result[result.len()-1], RegexType::LineEnd);
    
    // Verify escaped metacharacters
    let expected_chars = ['(', '[', '{', '*', '+', '?', '|', '\\', '$', '}', ']', ')'];
    let mut char_index = 0;
    
    for i in 1..result.len()-1 {
        if let RegexType::Char(c) = result[i] {
            assert_eq!(c, expected_chars[char_index], "Mismatch at index {}", i);
            char_index += 1;
        }
    }
    
    assert_eq!(char_index, expected_chars.len());
}

#[test]
fn test_pathological_regex() {
    // Test a pathologically complex regex that combines many features
    let pattern = "^(([a-z]+)|(\\d{1,3})|(\\w+\\-[0-9]+))+[^\\s\\d]?\\\\\\$\\d+(\\.\\d{2})?$";
    let result = Regex::tokens(pattern).unwrap();
    
    // Basic structure verification
    assert!(result.len() > 30);
    assert_eq!(result[0], RegexType::LineStart);
    assert_eq!(result[result.len()-1], RegexType::LineEnd);
    
    // Count groups, alternations, and quantifiers
    let group_count = result.iter().filter(|&t| *t == RegexType::OpenParenthesis).count();
    let alternation_count = result.iter().filter(|&t| *t == RegexType::Or).count();
    let quantifier_count = result.iter().filter(|&t| 
        matches!(t, RegexType::Quant(_))
    ).count();
    
    assert!(group_count >= 5);
    assert!(alternation_count >= 3);
    assert!(quantifier_count >= 4);
    
    // Verify we have character classes
    let has_char_class = result.iter().any(|t| 
        matches!(t, RegexType::Char('a'))
    );
    
    assert!(has_char_class);
}
