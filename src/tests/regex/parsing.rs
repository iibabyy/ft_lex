use crate::regex::parsing::*;
use crate::regex::*;
use std::collections::VecDeque;

// ==============================================
// 1. REGEXTYPE FUNCTIONALITY TESTS
// ==============================================

#[test]
fn test_regextype_char_matching() {
    let char_type = RegexType::Char('a');
    assert!(char_type.match_(&'a'));
    assert!(!char_type.match_(&'b'));
}

#[test]
fn test_regextype_any_matching() {
    let any_type = RegexType::Any;
    assert!(any_type.match_(&'a'));
    assert!(any_type.match_(&'1'));
    assert!(any_type.match_(&'\n'));
}

#[test]
fn test_regextype_precedence() {
    let char_type = RegexType::Char('a');
    let or_type = RegexType::Or;
    let concat_type = RegexType::Concatenation;
    let quant_type = RegexType::Quant(Quantifier::AtLeast(1));
    
    assert_eq!(char_type.precedence(), 0);
    assert_eq!(or_type.precedence(), 1);
    assert_eq!(concat_type.precedence(), 2);
    assert_eq!(quant_type.precedence(), 3);
}

#[test]
fn test_regextype_conversion_to_tokentype() {
    let char_type = RegexType::Char('a');
    let open_paren = RegexType::OpenParenthesis;
    let close_paren = RegexType::CloseParenthesis;
    let quant = RegexType::Quant(Quantifier::Exact(3));
    let or_type = RegexType::Or;
    let line_start = RegexType::LineStart;
    
    assert!(matches!(char_type.type_(), TokenType::Literal(_)));
    assert!(matches!(open_paren.type_(), TokenType::OpenParenthesis(_)));
    assert!(matches!(close_paren.type_(), TokenType::CloseParenthesis(_)));
    assert!(matches!(quant.type_(), TokenType::UnaryOperator(_)));
    assert!(matches!(or_type.type_(), TokenType::BinaryOperator(_)));
    assert!(matches!(line_start.type_(), TokenType::StartOrEndCondition(_)));
}

#[test]
fn test_regextype_display() {
    let char_type = RegexType::Char('a');
    let any_type = RegexType::Any;
    let or_type = RegexType::Or;
    
    assert_eq!(format!("{}", char_type), "a");
    assert_eq!(format!("{}", any_type), ".");
    assert_eq!(format!("{}", or_type), "|");
}

// ==============================================
// 2. TOKENTYPE FUNCTIONALITY TESTS
// ==============================================

#[test]
fn test_tokentype_conversion_from_regextype() {
    let char_type = RegexType::Char('a');
    let token = TokenType::from(char_type.clone());
    
    assert!(matches!(token, TokenType::Literal(_)));
    assert_eq!(token.into_inner(), &char_type);
}

#[test]
fn test_tokentype_into_inner() {
    let original = RegexType::Char('a');
    let token = TokenType::from(original.clone());
    
    let inner_ref = token.into_inner();
    assert_eq!(inner_ref, &original);
}

#[test]
fn test_tokentype_into_owned_inner() {
    let original = RegexType::Char('a');
    let token = TokenType::from(original.clone());
    
    let inner = token.into_owned_inner();
    assert_eq!(inner, original);
}

#[test]
fn test_tokentype_need_concatenation() {
    let literal = TokenType::Literal(RegexType::Char('a'));
    let open_paren = RegexType::OpenParenthesis;
    let close_paren = TokenType::CloseParenthesis(RegexType::CloseParenthesis);
    let or_op = RegexType::Or;
    
    // Literal followed by literal or opening parenthesis needs concatenation
    assert!(literal.need_concatenation_with(&RegexType::Char('b')));
    assert!(literal.need_concatenation_with(&open_paren));
    
    // Closing parenthesis followed by literal/opening parenthesis needs concatenation
    assert!(close_paren.need_concatenation_with(&RegexType::Char('b')));
    assert!(close_paren.need_concatenation_with(&open_paren));
    
    // Operator doesn't need concatenation
    assert!(!literal.need_concatenation_with(&or_op));
}

#[test]
fn test_tokentype_precedence() {
    let literal = TokenType::Literal(RegexType::Char('a'));
    let or_op = TokenType::BinaryOperator(RegexType::Or);
    let concat_op = TokenType::BinaryOperator(RegexType::Concatenation);
    let quant_op = TokenType::UnaryOperator(RegexType::Quant(Quantifier::AtLeast(1)));
    
    assert_eq!(literal.precedence(), 0);
    assert_eq!(or_op.precedence(), 1);
    assert_eq!(concat_op.precedence(), 2);
    assert_eq!(quant_op.precedence(), 3);
}

#[test]
fn test_tokentype_display() {
    let literal = TokenType::Literal(RegexType::Char('a'));
    let or_op = TokenType::BinaryOperator(RegexType::Or);
    
    assert_eq!(format!("{}", literal), "a");
    assert_eq!(format!("{}", or_op), "|");
}

// ==============================================
// 3. CHARACTER CLASS FUNCTIONALITY TESTS
// ==============================================

#[test]
fn test_character_class_construction() {
    let mut class = CharacterClass::new();
    class.add_char('a');
    class.add_char('b');
    
    assert!(class.contains_char(&'a'));
    assert!(class.contains_char(&'b'));
    assert!(!class.contains_char(&'c'));
}

#[test]
fn test_character_class_negation() {
    let mut class = CharacterClass::new();
    class.add_char('a');
    class.add_char('b');
    
    let negated = class.negated();
    
    assert!(!negated.matches(&'a'));
    assert!(!negated.matches(&'b'));
    assert!(negated.matches(&'c'));
}

#[test]
fn test_character_class_add_range() {
    let mut class = CharacterClass::new();
    class.add_range('a', 'c');
    
    assert!(class.contains_char(&'a'));
    assert!(class.contains_char(&'b'));
    assert!(class.contains_char(&'c'));
    assert!(!class.contains_char(&'d'));
}

#[test]
fn test_character_class_overlapping_ranges() {
    let mut class = CharacterClass::new();
    
    // Add a range
    class.add_range('a', 'e');
    
    // Add an overlapping range - should be added since it's not fully contained
    class.add_range('c', 'g');
    
    // This range is fully contained in an existing one - shouldn't be added
    class.add_range('b', 'd');
    
    // Check that all characters in the expected ranges are matched
    for c in 'a'..='g' {
        assert!(class.contains_char(&c));
    }
    assert!(!class.contains_char(&'h'));
}

#[test]
fn test_character_class_merge() {
    let mut class1 = CharacterClass::new();
    class1.add_range('a', 'c');
    
    let mut class2 = CharacterClass::new();
    class2.add_range('x', 'z');
    class2.add_char('d');
    
    class1.merge(&class2);
    
    assert!(class1.contains_char(&'a'));
    assert!(class1.contains_char(&'b'));
    assert!(class1.contains_char(&'c'));
    assert!(class1.contains_char(&'d'));
    assert!(class1.contains_char(&'x'));
    assert!(class1.contains_char(&'y'));
    assert!(class1.contains_char(&'z'));
}

#[test]
fn test_character_class_parse() {
    // Normal character class
    let mut input = "[abc]".chars();
    input.next(); // Skip the opening bracket
    let class = CharacterClass::parse(&mut input).unwrap();
    
    assert!(class.contains_char(&'a'));
    assert!(class.contains_char(&'b'));
    assert!(class.contains_char(&'c'));
    assert!(!class.contains_char(&'d'));
    
    // Negated character class
    let mut input = "[^abc]".chars();
    input.next(); // Skip the opening bracket
    let class = CharacterClass::parse(&mut input).unwrap();
    
    assert!(!class.matches(&'a'));
    assert!(!class.matches(&'b'));
    assert!(!class.matches(&'c'));
    assert!(class.matches(&'d'));
    
    // Character class with range
    let mut input = "[a-z]".chars();
    input.next(); // Skip the opening bracket
    let class = CharacterClass::parse(&mut input).unwrap();
    
    assert!(class.contains_char(&'a'));
    assert!(class.contains_char(&'m'));
    assert!(class.contains_char(&'z'));
    assert!(!class.contains_char(&'A'));
}

#[test]
fn test_character_class_special_chars() {
    // Test with dash at the beginning (should be literal)
    let mut input = "[-abc]".chars();
    input.next(); // Skip the opening bracket
    let class = CharacterClass::parse(&mut input).unwrap();
    
    assert!(class.contains_char(&'-'));
    assert!(class.contains_char(&'a'));
    
    // Test with dash at the end (should be literal)
    let mut input = "[abc-]".chars();
    input.next(); // Skip the opening bracket
    let class = CharacterClass::parse(&mut input).unwrap();
    
    assert!(class.contains_char(&'-'));
    assert!(class.contains_char(&'a'));
}

#[test]
fn test_character_class_escaped_chars() {
    // Test with escaped characters
    let mut input = "[\\n\\t\\-]".chars();
    input.next(); // Skip the opening bracket
    let class = CharacterClass::parse(&mut input).unwrap();
    
    assert!(class.contains_char(&'\n'));
    assert!(class.contains_char(&'\t'));
    assert!(class.contains_char(&'-'));
}

#[test]
fn test_character_class_predefined() {
    // Test digit class
    let digit_class = CharacterClass::digit();
    for c in '0'..='9' {
        assert!(digit_class.contains_char(&c));
    }
    assert!(!digit_class.contains_char(&'a'));
    
    // Test word character class
    let word_class = CharacterClass::word_char();
    for c in 'a'..='z' {
        assert!(word_class.contains_char(&c));
    }
    for c in 'A'..='Z' {
        assert!(word_class.contains_char(&c));
    }
    for c in '0'..='9' {
        assert!(word_class.contains_char(&c));
    }
    assert!(word_class.contains_char(&'_'));
    assert!(!word_class.contains_char(&'!'));
    
    // Test whitespace class
    let space_class = CharacterClass::whitespace();
    assert!(space_class.contains_char(&' '));
    assert!(space_class.contains_char(&'\t'));
    assert!(space_class.contains_char(&'\n'));
    assert!(!space_class.contains_char(&'a'));
}

#[test]
fn test_character_class_display() {
    let mut class = CharacterClass::new();
    class.add_char('a');
    class.add_range('0', '9');
    
    assert_eq!(format!("{}", class), "[a0-9]");
    
    let negated = class.negated();
    assert_eq!(format!("{}", negated), "[^a0-9]");
}

// ==============================================
// 4. QUANTIFIER FUNCTIONALITY TESTS
// ==============================================

#[test]
fn test_quantifier_exact() {
    let quant = Quantifier::Exact(3);
    assert_eq!(format!("{}", quant), "{3}");
}

#[test]
fn test_quantifier_at_least() {
    let quant = Quantifier::AtLeast(2);
    assert_eq!(format!("{}", quant), "{2,}");
}

#[test]
fn test_quantifier_range() {
    let quant = Quantifier::Range(2, 5);
    assert_eq!(format!("{}", quant), "{2,5}");
}

#[test]
fn test_quantifier_invalid_range() {
    // Test parsing of invalid range in add_quantifier method
    let mut input = "{5,3}".chars();
    input.next(); // Skip the opening brace
    
    let mut tokens = VecDeque::new();
    let result = Regex::add_quantifier(&mut tokens, &mut input);
    
    assert!(result.is_err());
}

// ==============================================
// 5. REGEX PARSING METHODS TESTS
// ==============================================

#[test]
fn test_add_concatenation() {
    let mut tokens = VecDeque::new();
    tokens.push_back(RegexType::Char('a'));
    tokens.push_back(RegexType::Char('b'));
    tokens.push_back(RegexType::Or);
    tokens.push_back(RegexType::Char('c'));
    
    let result = Regex::add_concatenation(tokens);
    
    // Expected: a & b | c
    assert_eq!(result.len(), 5);
    assert!(matches!(result[0].into_inner(), RegexType::Char('a')));
    assert!(matches!(result[1].into_inner(), RegexType::Concatenation));
    assert!(matches!(result[2].into_inner(), RegexType::Char('b')));
    assert!(matches!(result[3].into_inner(), RegexType::Or));
    assert!(matches!(result[4].into_inner(), RegexType::Char('c')));
}

#[test]
fn test_tokens_basic() {
    let input = "ab|c";
    let tokens = Regex::tokens(input).unwrap();
    
    assert_eq!(tokens.len(), 4);
    assert!(matches!(tokens[0], RegexType::Char('a')));
    assert!(matches!(tokens[1], RegexType::Char('b')));
    assert!(matches!(tokens[2], RegexType::Or));
    assert!(matches!(tokens[3], RegexType::Char('c')));
}

#[test]
fn test_add_string() {
    let mut tokens = VecDeque::new();
    let mut input = "\"hello\"".chars();
    input.next(); // Skip the opening quote
    
    Regex::add_string(&mut tokens, &mut input).unwrap();
    
    assert_eq!(tokens.len(), 7); // Open parenthesis + 5 chars + close parenthesis
    assert!(matches!(tokens[0], RegexType::OpenParenthesis));
    assert!(matches!(tokens[1], RegexType::Char('h')));
    assert!(matches!(tokens[5], RegexType::Char('o')));
    assert!(matches!(tokens[6], RegexType::CloseParenthesis));
}

#[test]
fn test_add_character_class() {
    let mut tokens = VecDeque::new();
    let mut input = "a-z]".chars();
    
    Regex::add_character_class(&mut tokens, &mut input).unwrap();
    
    assert_eq!(tokens.len(), 1);
    if let RegexType::Class(class) = &tokens[0] {
        assert!(class.contains_char(&'a'));
        assert!(class.contains_char(&'m'));
        assert!(class.contains_char(&'z'));
    } else {
        panic!("Expected a character class");
    }
}

#[test]
fn test_add_quantifier() {
    let mut tokens = VecDeque::new();
    let mut input = "2,5}".chars();
    
    Regex::add_quantifier(&mut tokens, &mut input).unwrap();
    
    assert_eq!(tokens.len(), 1);
    if let RegexType::Quant(Quantifier::Range(min, max)) = &tokens[0] {
        assert_eq!(*min, 2);
        assert_eq!(*max, 5);
    } else {
        panic!("Expected a range quantifier");
    }
}

#[test]
fn test_shorthand_notations() {
    let star = Regex::into_type('*', &mut "".chars());
    let plus = Regex::into_type('+', &mut "".chars());
    let question = Regex::into_type('?', &mut "".chars());
    
    if let RegexType::Quant(Quantifier::AtLeast(n)) = star {
        assert_eq!(n, 0);
    } else {
        panic!("Expected AtLeast(0) quantifier");
    }
    
    if let RegexType::Quant(Quantifier::AtLeast(n)) = plus {
        assert_eq!(n, 1);
    } else {
        panic!("Expected AtLeast(1) quantifier");
    }
    
    if let RegexType::Quant(Quantifier::Range(min, max)) = question {
        assert_eq!(min, 0);
        assert_eq!(max, 1);
    } else {
        panic!("Expected Range(0,1) quantifier");
    }
}

// ==============================================
// 6. ESCAPE SEQUENCE HANDLING TESTS
// ==============================================

#[test]
fn test_basic_escape_sequences() {
    let mut chars = "n".chars();
    let escaped_n = Regex::into_type('\\', &mut chars);
    
    if let RegexType::Char(c) = escaped_n {
        assert_eq!(c, '\n');
    } else {
        panic!("Expected a character");
    }
    
    let mut chars = "t".chars();
    let escaped_t = Regex::into_type('\\', &mut chars);
    
    if let RegexType::Char(c) = escaped_t {
        assert_eq!(c, '\t');
    } else {
        panic!("Expected a character");
    }
}

#[test]
fn test_shorthand_classes() {
    let mut chars = "d".chars();
    let digit_class = Regex::into_type('\\', &mut chars);
    
    if let RegexType::Class(class) = digit_class {
        assert!(class.contains_char(&'0'));
        assert!(class.contains_char(&'9'));
        assert!(!class.contains_char(&'a'));
    } else {
        panic!("Expected a character class");
    }
    
    let mut chars = "w".chars();
    let word_class = Regex::into_type('\\', &mut chars);
    
    if let RegexType::Class(class) = word_class {
        assert!(class.contains_char(&'a'));
        assert!(class.contains_char(&'Z'));
        assert!(class.contains_char(&'0'));
        assert!(class.contains_char(&'_'));
        assert!(!class.contains_char(&'!'));
    } else {
        panic!("Expected a character class");
    }
}

#[test]
fn test_special_char_escaping() {
    let mut chars = ".".chars();
    let escaped_dot = Regex::into_type('\\', &mut chars);
    
    if let RegexType::Char(c) = escaped_dot {
        assert_eq!(c, '.');
    } else {
        panic!("Expected a character");
    }
}

// ==============================================
// 7. ERROR HANDLING TESTS
// ==============================================

#[test]
fn test_unclosed_string() {
    let mut tokens = VecDeque::new();
    let mut input = "hello".chars();
    
    let result = Regex::add_string(&mut tokens, &mut input);
    assert!(result.is_err());
}

#[test]
fn test_unclosed_character_class() {
    let mut input = "[abc".chars();
    input.next(); // Skip the opening bracket
    
    let result = CharacterClass::parse(&mut input);
    assert!(result.is_err());
}

#[test]
fn test_invalid_quantifier_syntax() {
    let mut tokens = VecDeque::new();
    let mut input = "2,x}".chars();
    
    let result = Regex::add_quantifier(&mut tokens, &mut input);
    assert!(result.is_err());
}

#[test]
fn test_unclosed_quantifier() {
    let mut tokens = VecDeque::new();
    let mut input = "2,5".chars();
    
    let result = Regex::add_quantifier(&mut tokens, &mut input);
    assert!(result.is_err());
}

// ==============================================
// 8. EDGE CASES TESTS
// ==============================================

#[test]
fn test_empty_regex() {
    let input = "";
    let tokens = Regex::tokens(input).unwrap();
    
    assert_eq!(tokens.len(), 0);
}

#[test]
fn test_complex_nested_expressions() {
    let input = "a(b|c)+d";
    let tokens = Regex::tokens(input).unwrap();

    assert_eq!(tokens.len(), 8);
    assert!(matches!(tokens[0], RegexType::Char('a')));
    assert!(matches!(tokens[1], RegexType::OpenParenthesis));
    assert!(matches!(tokens[2], RegexType::Char('b')));
    assert!(matches!(tokens[3], RegexType::Or));
    assert!(matches!(tokens[4], RegexType::Char('c')));
    assert!(matches!(tokens[5], RegexType::CloseParenthesis));
    assert!(matches!(tokens[6], RegexType::Quant(Quantifier::AtLeast(1))));
    assert!(matches!(tokens[7], RegexType::Char('d')));
}

#[test]
fn test_line_anchors() {
    let input = "^abc$";
    let tokens = Regex::tokens(input).unwrap();
    
    assert_eq!(tokens.len(), 5);
    assert!(matches!(tokens[0], RegexType::LineStart));
    assert!(matches!(tokens[1], RegexType::Char('a')));
    assert!(matches!(tokens[2], RegexType::Char('b')));
    assert!(matches!(tokens[3], RegexType::Char('c')));
    assert!(matches!(tokens[4], RegexType::LineEnd));
}

#[test]
fn test_alternation() {
    let input = "a|b|c";
    let tokens = Regex::tokens(input).unwrap();
    
    assert_eq!(tokens.len(), 5);
    assert!(matches!(tokens[0], RegexType::Char('a')));
    assert!(matches!(tokens[1], RegexType::Or));
    assert!(matches!(tokens[2], RegexType::Char('b')));
    assert!(matches!(tokens[3], RegexType::Or));
    assert!(matches!(tokens[4], RegexType::Char('c')));
}

#[test]
fn test_boundary_character_ranges() {
    // Test with boundary values for character ranges
    let mut class = CharacterClass::new();
    
    // Test the full ASCII range
    class.add_range('\0', '~');
    
    assert!(class.contains_char(&'\0'));
    assert!(class.contains_char(&'A'));
    assert!(class.contains_char(&'~'));
    
    // Test equal start and end (single character)
    let mut class2 = CharacterClass::new();
    class2.add_range('x', 'x');
    
    assert!(class2.contains_char(&'x'));
    assert!(!class2.contains_char(&'w'));
    assert!(!class2.contains_char(&'y'));
}
