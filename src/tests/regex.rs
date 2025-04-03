use crate::parsing::error::ParsingResult;
use crate::regex::{CharacterClass, Quantifier, Regex, RegexType, TokenType};
use std::collections::VecDeque;

// 1. TOKEN CREATION TESTS
// ======================

#[test]
fn test_into_type_basic_chars() {
    let mut chars = "".chars();
    assert!(matches!(Regex::into_type('a', &mut chars), RegexType::Char('a')));
    assert!(matches!(Regex::into_type('1', &mut chars), RegexType::Char('1')));
    assert!(matches!(Regex::into_type(' ', &mut chars), RegexType::Char(' ')));
}

#[test]
fn test_into_type_special_chars() {
    let mut chars = "".chars();
    assert!(matches!(Regex::into_type('(', &mut chars), RegexType::OpenParenthesis));
    assert!(matches!(Regex::into_type(')', &mut chars), RegexType::CloseParenthesis));
    assert!(matches!(Regex::into_type('?', &mut chars), RegexType::QuestionMark));
    assert!(matches!(Regex::into_type('|', &mut chars), RegexType::Or));
    assert!(matches!(Regex::into_type('.', &mut chars), RegexType::Dot));
}

#[test]
fn test_into_type_escape_sequences() {
    let mut chars = "d".chars();
    if let RegexType::Class(class) = Regex::into_type('\\', &mut chars) {
        assert!(class.matches(&'0'));
        assert!(!class.matches(&'a'));
    } else {
        panic!("Expected a CharacterClass for digit");
    }
    
    let mut chars = "w".chars();
    if let RegexType::Class(class) = Regex::into_type('\\', &mut chars) {
        assert!(class.matches(&'a'));
        assert!(class.matches(&'_'));
        assert!(!class.matches(&' '));
    } else {
        panic!("Expected a CharacterClass for word char");
    }
    
    let mut chars = "n".chars();
    if let RegexType::Char(c) = Regex::into_type('\\', &mut chars) {
        assert_eq!(c, '\n');
    } else {
        panic!("Expected a newline character");
    }
}

// 2. TOKENIZATION TESTS
// ====================

#[test]
fn test_tokens_simple_regex() -> ParsingResult<()> {
    let input = "a+b*".to_string();
    let tokens = Regex::tokens(&input)?;
    
    assert_eq!(tokens.len(), 4);
    assert!(matches!(tokens[0], RegexType::Char('a')));
    assert!(matches!(tokens[2], RegexType::Char('b')));
    
    Ok(())
}

#[test]
fn test_tokens_with_string_literals() -> ParsingResult<()> {
    let input = "\"abc\"".to_string();
    let tokens = Regex::tokens(&input)?;
    
    assert_eq!(tokens.len(), 5);
    assert!(matches!(tokens[0], RegexType::OpenParenthesis));
    assert!(matches!(tokens[1], RegexType::Char('a')));
    assert!(matches!(tokens[2], RegexType::Char('b')));
    assert!(matches!(tokens[3], RegexType::Char('c')));
    assert!(matches!(tokens[4], RegexType::CloseParenthesis));
    
    Ok(())
}

#[test]
fn test_tokens_unclosed_string() {
    let input = "\"abc".to_string();
    let result = Regex::tokens(&input);
    assert!(result.is_err());
}

#[test]
fn test_tokens_with_character_classes() -> ParsingResult<()> {
    let input = "a[bc]d".to_string();
    let tokens = Regex::tokens(&input)?;
    
    assert_eq!(tokens.len(), 3);
    assert!(matches!(tokens[0], RegexType::Char('a')));
    
    if let RegexType::Class(class) = &tokens[1] {
        assert!(class.contains_char(&'b'));
        assert!(class.contains_char(&'c'));
        assert!(!class.contains_char(&'a'));
    } else {
        panic!("Expected a character class");
    }
    
    assert!(matches!(tokens[2], RegexType::Char('d')));
    
    Ok(())
}

#[test]
fn test_tokens_with_negated_character_class() -> ParsingResult<()> {
    let input = "[^abc]".to_string();
    let tokens = Regex::tokens(&input)?;
    
    assert_eq!(tokens.len(), 1);
    
    if let RegexType::Class(class) = &tokens[0] {
        // Check if it's negated by verifying that it doesn't match characters in the class
        // but matches characters outside the class
        assert!(!class.matches(&'a'));
        assert!(!class.matches(&'b'));
        assert!(!class.matches(&'c'));
        assert!(class.matches(&'x'));
    } else {
        panic!("Expected a character class");
    }
    
    Ok(())
}

#[test]
fn test_tokens_with_shorthand_classes() -> ParsingResult<()> {
    let input = "\\d\\w\\s".to_string();
    let tokens = Regex::tokens(&input)?;
    
    assert_eq!(tokens.len(), 3);
    
    // Check digit class
    if let RegexType::Class(class) = &tokens[0] {
        assert!(class.matches(&'0'));
        assert!(!class.matches(&'a'));
    } else {
        panic!("Expected a digit class");
    }
    
    // Check word class
    if let RegexType::Class(class) = &tokens[1] {
        assert!(class.matches(&'a'));
        assert!(class.matches(&'_'));
    } else {
        panic!("Expected a word class");
    }
    
    // Check whitespace class
    if let RegexType::Class(class) = &tokens[2] {
        assert!(class.matches(&' '));
        assert!(class.matches(&'\t'));
    } else {
        panic!("Expected a whitespace class");
    }
    
    Ok(())
}

#[test]
fn test_complex_regex_pattern() -> ParsingResult<()> {
    let input = "(a|b)+[0-9]?\\w{2,5}".to_string();
    let tokens = Regex::tokens(&input)?;
    
    // Check for expected token types
    assert!(matches!(tokens[0], RegexType::OpenParenthesis));
    assert!(matches!(tokens[1], RegexType::Char('a')));
    assert!(matches!(tokens[2], RegexType::Or));
    assert!(matches!(tokens[3], RegexType::Char('b')));
    assert!(matches!(tokens[4], RegexType::CloseParenthesis));
    
    // Check character class
    if let RegexType::Class(class) = &tokens[6] {
        assert!(class.matches(&'0'));
        assert!(class.matches(&'9'));
        assert!(!class.matches(&'a'));
    } else {
        panic!("Expected a digit character class");
    }
    
    assert!(matches!(tokens[7], RegexType::QuestionMark));
    
    // Check word char class
    if let RegexType::Class(class) = &tokens[8] {
        assert!(class.matches(&'a'));
        assert!(class.matches(&'_'));
    } else {
        panic!("Expected a word character class");
    }
    
    // Check quantifier
    if let RegexType::Quant(quant) = &tokens[9] {
        if let Quantifier::Range(min, max) = quant {
            assert_eq!(*min, 2);
            assert_eq!(*max, 5);
        } else {
            panic!("Expected a range quantifier");
        }
    } else {
        panic!("Expected a quantifier");
    }
    
    Ok(())
}

// 3. CONCATENATION TESTS
// =====================

#[test]
fn test_add_concatenation_simple() {
    // a|b should not have concatenation
    let mut tokens = VecDeque::new();
    tokens.push_back(RegexType::Char('a'));
    tokens.push_back(RegexType::Or);
    tokens.push_back(RegexType::Char('b'));
    
    let result = Regex::add_concatenation(tokens);
    
    assert_eq!(result.len(), 3);
    assert!(matches!(result[0], TokenType::Literal(RegexType::Char('a'))));
    assert!(matches!(result[1], TokenType::BinaryOperator(RegexType::Or)));
    assert!(matches!(result[2], TokenType::Literal(RegexType::Char('b'))));
}

#[test]
fn test_add_concatenation_needed() {
    // ab should become a·b
    let mut tokens = VecDeque::new();
    tokens.push_back(RegexType::Char('a'));
    tokens.push_back(RegexType::Char('b'));
    
    let result = Regex::add_concatenation(tokens);
    
    assert_eq!(result.len(), 3);
    assert!(matches!(result[0], TokenType::Literal(RegexType::Char('a'))));
    assert!(matches!(result[1], TokenType::BinaryOperator(RegexType::Concatenation)));
    assert!(matches!(result[2], TokenType::Literal(RegexType::Char('b'))));
}

#[test]
fn test_add_concatenation_complex() {
    // (a)b should become (a)·b
    let mut tokens = VecDeque::new();
    tokens.push_back(RegexType::OpenParenthesis);
    tokens.push_back(RegexType::Char('a'));
    tokens.push_back(RegexType::CloseParenthesis);
    tokens.push_back(RegexType::Char('b'));
    
    let result = Regex::add_concatenation(tokens);

    assert_eq!(result.len(), 5);
    assert!(matches!(result[0], TokenType::OpenParenthesis(RegexType::OpenParenthesis)));
    assert!(matches!(result[1], TokenType::Literal(RegexType::Char('a'))));
    assert!(matches!(result[2], TokenType::CloseParenthesis(RegexType::CloseParenthesis)));
    assert!(matches!(result[3], TokenType::BinaryOperator(RegexType::Concatenation)));
    assert!(matches!(result[4], TokenType::Literal(RegexType::Char('b'))));
}

// 4. END-TO-END TESTS
// ==================

#[test]
fn test_regex_new_simple() -> ParsingResult<()> {
    let expr = "ab+c*".to_string();
    let result = Regex::new(expr)?;
    
    // Should produce postfix: a b+ · c* ·
    assert_eq!(result.len(), 7);
    assert!(matches!(result[0], TokenType::Literal(RegexType::Char('a'))));
    assert!(matches!(result[1], TokenType::Literal(RegexType::Char('b'))));
    assert!(matches!(result[2], TokenType::UnaryOperator(RegexType::Quant(Quantifier::AtLeast(1)))));
    assert!(matches!(result[3], TokenType::BinaryOperator(RegexType::Concatenation)));
    assert!(matches!(result[4], TokenType::Literal(RegexType::Char('c'))));
    assert!(matches!(result[5], TokenType::UnaryOperator(RegexType::Quant(Quantifier::AtLeast(0)))));
    assert!(matches!(result[6], TokenType::BinaryOperator(RegexType::Concatenation)));
    
    Ok(())
}

#[test]
fn test_regex_new_with_alternation() -> ParsingResult<()> {
    let expr = "a|b|c".to_string();
    let result = Regex::new(expr)?;
    
    // Should produce postfix: a b | c |
    assert_eq!(result.len(), 5);
    assert!(matches!(result[0], TokenType::Literal(RegexType::Char('a'))));
    assert!(matches!(result[1], TokenType::Literal(RegexType::Char('b'))));
    assert!(matches!(result[2], TokenType::BinaryOperator(RegexType::Or)));
    assert!(matches!(result[3], TokenType::Literal(RegexType::Char('c'))));
    assert!(matches!(result[4], TokenType::BinaryOperator(RegexType::Or)));
    
    Ok(())
}

#[test]
fn test_regex_new_with_groups() -> ParsingResult<()> {
    let expr = "(a)(b)".to_string();
    let result = Regex::new(expr)?;
    
    // Should produce postfix: a b ·
    assert_eq!(result.len(), 3);
    assert!(matches!(result[0], TokenType::Literal(RegexType::Char('a'))));
    assert!(matches!(result[1], TokenType::Literal(RegexType::Char('b'))));
    assert!(matches!(result[2], TokenType::BinaryOperator(RegexType::Concatenation)));
    
    Ok(())
}

#[test]
fn test_regex_new_with_complex_pattern() -> ParsingResult<()> {
    let expr = "(a|b)+c?".to_string();
    let result = Regex::new(expr)?;
    
    dbg!(&result);

    // Should produce postfix: a b | + c ? ·
    assert_eq!(result.len(), 7);
    assert!(matches!(result[0], TokenType::Literal(RegexType::Char('a'))));
    assert!(matches!(result[1], TokenType::Literal(RegexType::Char('b'))));
    assert!(matches!(result[2], TokenType::BinaryOperator(RegexType::Or)));
    assert!(matches!(result[3], TokenType::UnaryOperator(RegexType::Quant(Quantifier::AtLeast(1)))));
    assert!(matches!(result[4], TokenType::Literal(RegexType::Char('c'))));
    assert!(matches!(result[5], TokenType::UnaryOperator(RegexType::QuestionMark)));
    assert!(matches!(result[6], TokenType::BinaryOperator(RegexType::Concatenation)));
    
    Ok(())
}

#[test]
fn test_regex_new_with_nested_groups() -> ParsingResult<()> {
    let expr = "a(b(c|d)e)f".to_string();
    let result = Regex::new(expr)?;
    
    dbg!(&result);  

    // Should produce postfix: a b c d | · e · · f ·
    assert_eq!(result.len(), 11);
    assert!(matches!(result[0], TokenType::Literal(RegexType::Char('a'))));
    assert!(matches!(result[1], TokenType::Literal(RegexType::Char('b'))));
    assert!(matches!(result[2], TokenType::Literal(RegexType::Char('c'))));
    assert!(matches!(result[3], TokenType::Literal(RegexType::Char('d'))));
    assert!(matches!(result[4], TokenType::BinaryOperator(RegexType::Or)));
    assert!(matches!(result[5], TokenType::BinaryOperator(RegexType::Concatenation)));
    assert!(matches!(result[6], TokenType::Literal(RegexType::Char('e'))));
    assert!(matches!(result[7], TokenType::BinaryOperator(RegexType::Concatenation)));
    assert!(matches!(result[8], TokenType::BinaryOperator(RegexType::Concatenation)));
    assert!(matches!(result[9], TokenType::Literal(RegexType::Char('f'))));
    assert!(matches!(result[10], TokenType::BinaryOperator(RegexType::Concatenation)));
    
    Ok(())
}

#[test]
fn test_regex_new_with_character_classes() -> ParsingResult<()> {
    let expr = "[a-z]+\\d*".to_string();
    let result = Regex::new(expr)?;
    
    // Should produce postfix where first token is a character class and third is a digit class
    assert!(result.len() >= 5);
    
    if let TokenType::Literal(RegexType::Class(class1)) = &result[0] {
        assert!(class1.matches(&'a'));
        assert!(class1.matches(&'z'));
        assert!(!class1.matches(&'0'));
    } else {
        panic!("Expected a character class at position 0");
    }
    
    assert!(matches!(result[1], TokenType::UnaryOperator(RegexType::Quant(Quantifier::AtLeast(1)))));
    
    if let TokenType::Literal(RegexType::Class(class2)) = &result[2] {
        assert!(class2.matches(&'0'));
        assert!(class2.matches(&'9'));
        assert!(!class2.matches(&'a'));
    } else {
        panic!("Expected a digit class at position 2");
    }
    
    assert!(matches!(result[3], TokenType::UnaryOperator(RegexType::Quant(Quantifier::AtLeast(0)))));
    
    Ok(())
}

// 5. SHORTHAND CLASS TESTS
// =======================

#[test]
fn test_character_class_from_shorthand() -> ParsingResult<()> {
    assert!(CharacterClass::from_shorthand('d').is_ok());
    assert!(CharacterClass::from_shorthand('D').is_ok());
    assert!(CharacterClass::from_shorthand('w').is_ok());
    assert!(CharacterClass::from_shorthand('W').is_ok());
    assert!(CharacterClass::from_shorthand('s').is_ok());
    assert!(CharacterClass::from_shorthand('S').is_ok());
    assert!(CharacterClass::from_shorthand('x').is_err());
    
    let digit_class = CharacterClass::from_shorthand('d')?;
    assert!(digit_class.matches(&'0'));
    assert!(!digit_class.matches(&'a'));
    
    let word_class = CharacterClass::from_shorthand('w')?;
    assert!(word_class.matches(&'a'));
    assert!(word_class.matches(&'_'));
    assert!(!word_class.matches(&' '));
    
    let space_class = CharacterClass::from_shorthand('s')?;
    assert!(space_class.matches(&' '));
    assert!(space_class.matches(&'\t'));
    assert!(!space_class.matches(&'a'));
    
    Ok(())
}

// 6. TOKEN TYPE TESTS
// =================

#[test]
fn test_token_type_from_regex_type() {
    assert!(matches!(
        TokenType::from(RegexType::Char('a')),
        TokenType::Literal(RegexType::Char('a'))
    ));
    
    assert!(matches!(
        TokenType::from(RegexType::Or),
        TokenType::BinaryOperator(RegexType::Or)
    ));
    
    assert!(matches!(
        TokenType::from(RegexType::OpenParenthesis),
        TokenType::OpenParenthesis(RegexType::OpenParenthesis)
    ));
    
    assert!(matches!(
        TokenType::from(RegexType::CloseParenthesis),
        TokenType::CloseParenthesis(RegexType::CloseParenthesis)
    ));
}

// 7. CHARACTER CLASS TESTS
// ======================

#[test]
fn test_character_class_basic() {
    let mut class = CharacterClass::new();
    class.add_char('a');
    class.add_char('b');
    class.add_char('c');
    
    assert!(class.contains_char(&'a'));
    assert!(class.contains_char(&'b'));
    assert!(class.contains_char(&'c'));
    assert!(!class.contains_char(&'d'));
    
    assert!(class.matches(&'a'));
    assert!(!class.matches(&'d'));
}

#[test]
fn test_character_class_range() {
    let mut class = CharacterClass::new();
    class.add_range('a', 'z');
    
    assert!(class.contains_char(&'a'));
    assert!(class.contains_char(&'m'));
    assert!(class.contains_char(&'z'));
    assert!(!class.contains_char(&'A'));
}

#[test]
fn test_character_class_negated() {
    let class = CharacterClass::new().negated();
    
    assert!(!class.contains_char(&'a')); // Empty class contains nothing
    assert!(class.matches(&'a')); // But negated matches everything
}

#[test]
fn test_character_class_complex() {
    let mut class = CharacterClass::new();
    class.add_char('a');
    class.add_char('b');
    class.add_range('x', 'z');
    
    assert!(class.contains_char(&'a'));
    assert!(class.contains_char(&'b'));
    assert!(!class.contains_char(&'c'));
    assert!(class.contains_char(&'x'));
    assert!(class.contains_char(&'y'));
    assert!(class.contains_char(&'z'));
    assert!(!class.contains_char(&'w'));
}

#[test]
fn test_character_class_parse() -> ParsingResult<()> {
    let input = "abc]";
    let mut chars = input.chars();
    let class = CharacterClass::parse(&mut chars)?;
    
    assert!(class.contains_char(&'a'));
    assert!(class.contains_char(&'b'));
    assert!(class.contains_char(&'c'));
    assert!(!class.contains_char(&'d'));
    
    Ok(())
}

#[test]
fn test_character_class_parse_range() -> ParsingResult<()> {
    let input = "a-z]";
    let mut chars = input.chars();
    let class = CharacterClass::parse(&mut chars)?;
    
    assert!(class.contains_char(&'a'));
    assert!(class.contains_char(&'m'));
    assert!(class.contains_char(&'z'));
    assert!(!class.contains_char(&'A'));
    
    Ok(())
}

#[test]
fn test_character_class_parse_complex() -> ParsingResult<()> {
    let input = "a-cx-z]";
    let mut chars = input.chars();
    let class = CharacterClass::parse(&mut chars)?;
    
    assert!(class.contains_char(&'a'));
    assert!(class.contains_char(&'b'));
    assert!(class.contains_char(&'c'));
    assert!(!class.contains_char(&'d'));
    assert!(class.contains_char(&'x'));
    assert!(class.contains_char(&'y'));
    assert!(class.contains_char(&'z'));
    
    Ok(())
}

#[test]
fn test_character_class_parse_negated() -> ParsingResult<()> {
    let input = "^a-c]";
    let mut chars = input.chars();
    let class = CharacterClass::parse(&mut chars)?;
    
    assert!(!class.matches(&'a'));
    assert!(!class.matches(&'b'));
    assert!(!class.matches(&'c'));
    assert!(class.matches(&'d'));
    
    Ok(())
}

#[test]
fn test_character_class_merge() {
    let mut class1 = CharacterClass::new();
    class1.add_char('a');
    class1.add_range('0', '9');
    
    let mut class2 = CharacterClass::new();
    class2.add_char('b');
    class2.add_range('x', 'z');
    
    class1.merge(&class2);
    
    // Check that class1 now contains all characters from both classes
    assert!(class1.contains_char(&'a'));
    assert!(class1.contains_char(&'b'));
    assert!(class1.contains_char(&'0'));
    assert!(class1.contains_char(&'5'));
    assert!(class1.contains_char(&'9'));
    assert!(class1.contains_char(&'x'));
    assert!(class1.contains_char(&'y'));
    assert!(class1.contains_char(&'z'));
}

#[test]
fn test_character_class_merge_with_negated() {
    let mut class1 = CharacterClass::new();
    class1.add_char('a');
    
    let class2 = CharacterClass::new().negated();
    
    // Merging with a negated class should not change class1
    class1.merge(&class2);
    
    assert!(class1.contains_char(&'a'));
    assert!(!class1.contains_char(&'b'));
}

#[test]
fn test_character_class_parse_edge_cases() -> ParsingResult<()> {
    // Test dash at beginning
    let input = "-abc]";
    let mut chars = input.chars();
    let class = CharacterClass::parse(&mut chars)?;
    
    assert!(class.contains_char(&'-'));
    assert!(class.contains_char(&'a'));
    
    // Test dash at end
    let input = "abc-]";
    let mut chars = input.chars();
    let class = CharacterClass::parse(&mut chars)?;
    
    assert!(class.contains_char(&'-'));
    assert!(class.contains_char(&'a'));
    
    Ok(())
}

#[test]
fn test_character_class_parse_with_escapes() -> ParsingResult<()> {
    let input = "a\\n\\t]";
    let mut chars = input.chars();
    let class = CharacterClass::parse(&mut chars)?;
    
    assert!(class.contains_char(&'a'));
    assert!(class.contains_char(&'\n'));
    assert!(class.contains_char(&'\t'));
    
    Ok(())
}

#[test]
fn test_character_class_predefined_methods() {
    // Test digit class
    let digit = CharacterClass::digit();
    assert!(digit.matches(&'0'));
    assert!(digit.matches(&'9'));
    assert!(!digit.matches(&'a'));
    
    // Test non-digit class
    let non_digit = CharacterClass::non_digit();
    assert!(!non_digit.matches(&'0'));
    assert!(non_digit.matches(&'a'));
    
    // Test word char class
    let word = CharacterClass::word_char();
    assert!(word.matches(&'a'));
    assert!(word.matches(&'Z'));
    assert!(word.matches(&'0'));
    assert!(word.matches(&'_'));
    assert!(!word.matches(&' '));
    assert!(!word.matches(&'-'));
    
    // Test non-word char class
    let non_word = CharacterClass::non_word_char();
    assert!(!non_word.matches(&'a'));
    assert!(non_word.matches(&' '));
    assert!(non_word.matches(&'-'));
    
    // Test whitespace class
    let space = CharacterClass::whitespace();
    assert!(space.matches(&' '));
    assert!(space.matches(&'\t'));
    assert!(space.matches(&'\n'));
    assert!(space.matches(&'\r'));
    assert!(!space.matches(&'a'));
    
    // Test non-whitespace class
    let non_space = CharacterClass::non_whitespace();
    assert!(!non_space.matches(&' '));
    assert!(non_space.matches(&'a'));
}

#[test]
fn test_character_class_convenience_constructors() {
    // Test single character constructor
    let single = CharacterClass::single('x');
    assert!(single.contains_char(&'x'));
    assert!(!single.contains_char(&'y'));
    
    // Test range constructor
    let range = CharacterClass::range('a', 'c');
    assert!(range.contains_char(&'a'));
    assert!(range.contains_char(&'b'));
    assert!(range.contains_char(&'c'));
    assert!(!range.contains_char(&'d'));
    
    // Test negated helper
    let negated = CharacterClass::from_negated(CharacterClass::single('x'));
    assert!(!negated.matches(&'x'));
    assert!(negated.matches(&'y'));
}

#[test]
fn test_character_class_unclosed() {
    let input = "[abc".to_string();
    let mut chars = input.chars();
    let result = CharacterClass::parse(&mut chars);
    assert!(result.is_err());
}

#[test]
fn test_character_class_empty() -> ParsingResult<()> {
    let input = "]".to_string();
    let mut chars = input.chars();
    let class = CharacterClass::parse(&mut chars)?;
    
    // Empty class should match nothing
    assert!(!class.contains_char(&'a'));
    assert!(!class.contains_char(&'b'));
    
    Ok(())
}

#[test]
fn test_character_class_add_invalid_range() {
    let mut class = CharacterClass::new();
    
    // Adding a range where start > end should be ignored
    class.add_range('z', 'a');
    
    assert!(!class.contains_char(&'a'));
    assert!(!class.contains_char(&'z'));
}

#[test]
fn test_character_class_from_shorthand_invalid() {
    let result = CharacterClass::from_shorthand('x');
    assert!(result.is_err());
    
    if let Err(err) = result {
        assert!(err.to_string().contains("Unknown shorthand class"));
    }
}

// 8. QUANTIFIER TESTS
// =================

#[test]
fn test_quantifier_exact() -> ParsingResult<()> {
    let input = "5}";
    let mut chars = input.chars();
    let tokens = &mut VecDeque::new();
    
    Regex::add_quantifier(tokens, &mut chars)?;
    
    assert_eq!(tokens.len(), 1);
    if let RegexType::Quant(Quantifier::Exact(n)) = &tokens[0] {
        assert_eq!(*n, 5);
    } else {
        panic!("Expected Quantifier::Exact");
    }
    
    Ok(())
}

#[test]
fn test_quantifier_at_least() -> ParsingResult<()> {
    let input = "3,}";
    let mut chars = input.chars();
    let tokens = &mut VecDeque::new();
    
    Regex::add_quantifier(tokens, &mut chars)?;
    
    assert_eq!(tokens.len(), 1);
    if let RegexType::Quant(Quantifier::AtLeast(n)) = &tokens[0] {
        assert_eq!(*n, 3);
    } else {
        panic!("Expected Quantifier::AtLeast");
    }
    
    Ok(())
}

#[test]
fn test_quantifier_range() -> ParsingResult<()> {
    let input = "2,5}";
    let mut chars = input.chars();
    let tokens = &mut VecDeque::new();
    
    Regex::add_quantifier(tokens, &mut chars)?;
    
    assert_eq!(tokens.len(), 1);
    if let RegexType::Quant(Quantifier::Range(min, max)) = &tokens[0] {
        assert_eq!(*min, 2);
        assert_eq!(*max, 5);
    } else {
        panic!("Expected Quantifier::Range");
    }
    
    Ok(())
}

#[test]
fn test_quantifier_zero() -> ParsingResult<()> {
    let input = "0}";
    let mut chars = input.chars();
    let tokens = &mut VecDeque::new();
    
    Regex::add_quantifier(tokens, &mut chars)?;
    
    if let RegexType::Quant(Quantifier::Exact(n)) = &tokens[0] {
        assert_eq!(*n, 0);
    } else {
        panic!("Expected Quantifier::Exact(0)");
    }
    
    Ok(())
}

#[test]
fn test_quantifier_empty_range() -> ParsingResult<()> {
    let input = "0,0}";
    let mut chars = input.chars();
    let tokens = &mut VecDeque::new();
    
    Regex::add_quantifier(tokens, &mut chars)?;
    
    if let RegexType::Quant(Quantifier::Range(min, max)) = &tokens[0] {
        assert_eq!(*min, 0);
        assert_eq!(*max, 0);
    } else {
        panic!("Expected Quantifier::Range(0, 0)");
    }
    
    Ok(())
}

#[test]
fn test_quantifier_display() {
    let exact = Quantifier::Exact(5);
    assert_eq!(exact.to_string(), "{5}");
    
    let at_least = Quantifier::AtLeast(2);
    assert_eq!(at_least.to_string(), "{2,}");
    
    let range = Quantifier::Range(1, 3);
    assert_eq!(range.to_string(), "{1,3}");
}

#[test]
fn test_quantifier_invalid_range() {
    let input = "5,2}";
    let mut chars = input.chars();
    let tokens = &mut VecDeque::new();
    
    let result = Regex::add_quantifier(tokens, &mut chars);
    assert!(result.is_err());
}

#[test]
fn test_quantifier_unclosed() {
    let input = "5";
    let mut chars = input.chars();
    let tokens = &mut VecDeque::new();
    
    let result = Regex::add_quantifier(tokens, &mut chars);
    assert!(result.is_err());
}
