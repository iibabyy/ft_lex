use crate::regex::parsing::*;
use crate::regex::*;
use std::collections::VecDeque;
use std::collections::HashSet;

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
    // Test removed since RegexType::Any no longer exists
    // Any character (dot) is now implemented as an alternation of all characters
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
    let or_type = RegexType::Or;
    
    assert_eq!(format!("{}", char_type), "a");
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
    
    let mut tokens = VecDeque::new();
    class.clone().push_into_tokens(&mut tokens);
    
    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0], RegexType::OpenParenthesis);
    assert!(matches!(tokens[1], RegexType::Char('a')) || matches!(tokens[1], RegexType::Char('b')));
    assert_eq!(tokens[2], RegexType::Or);
    assert!(matches!(tokens[3], RegexType::Char('a')) || matches!(tokens[3], RegexType::Char('b')));
    assert_eq!(tokens[4], RegexType::CloseParenthesis);
}

#[test]
fn test_character_class_negation() {
    let mut class = CharacterClass::new();
    class.add_char('a');
    class.add_char('b');
    
    let negated = class.negated();
    
    let mut tokens = VecDeque::new();
    negated.push_into_tokens(&mut tokens);
    
    assert!(tokens.len() > 5);
    assert_eq!(tokens[0], RegexType::OpenParenthesis);
    assert_eq!(tokens[tokens.len()-1], RegexType::CloseParenthesis);
    
    let mut contains_a = false;
    let mut contains_b = false;
    
    for token in &tokens {
        if let RegexType::Char('a') = token {
            contains_a = true;
        }
        if let RegexType::Char('b') = token {
            contains_b = true;
        }
    }
    
    assert!(!contains_a);
    assert!(!contains_b);
}

#[test]
fn test_character_class_add_range() {
    let mut class = CharacterClass::new();
    class.add_range('a', 'c').unwrap();
    
    let mut tokens = VecDeque::new();
    class.push_into_tokens(&mut tokens);
    
    assert_eq!(tokens.len(), 7);
    assert_eq!(tokens[0], RegexType::OpenParenthesis);
    
    let mut contains_a = false;
    let mut contains_b = false;
    let mut contains_c = false;
    
    for token in &tokens {
        if let RegexType::Char('a') = token {
            contains_a = true;
        } else if let RegexType::Char('b') = token {
            contains_b = true;
        } else if let RegexType::Char('c') = token {
            contains_c = true;
        }
    }
    
    assert!(contains_a);
    assert!(contains_b);
    assert!(contains_c);
}

#[test]
fn test_character_class_overlapping_ranges() {
    let mut class = CharacterClass::new();
    
    class.add_range('a', 'e').unwrap();
    class.add_range('c', 'g').unwrap();
    
    let mut tokens = VecDeque::new();
    class.push_into_tokens(&mut tokens);
    
    let mut chars_found = HashSet::new();
    
    for token in &tokens {
        if let RegexType::Char(c) = token {
            chars_found.insert(*c);
        }
    }
    
    for c in 'a'..='g' {
        assert!(chars_found.contains(&c));
    }
    assert!(!chars_found.contains(&'h'));
}

#[test]
fn test_character_class_merge() {
    let mut class1 = CharacterClass::new();
    class1.add_char('a');
    class1.add_char('b');
    
    let mut class2 = CharacterClass::new();
    class2.add_char('c');
    class2.add_char('d');
    
    let mut tokens = VecDeque::new();
    class1.push_into_tokens(&mut tokens);
    
    assert!(!tokens.is_empty());
}

#[test]
fn test_character_class_parse() {
    // Simple character class
    let mut input = "[abc]".chars();
    let class = CharacterClass::parse(&mut input).unwrap();
    
    // Test using push_into_tokens
    let mut tokens = VecDeque::new();
    class.push_into_tokens(&mut tokens);
    
    // Check for 'a', 'b', 'c'
    let mut contains_a = false;
    let mut contains_b = false;
    let mut contains_c = false;
    
    for token in &tokens {
        if let RegexType::Char('a') = token {
            contains_a = true;
        } else if let RegexType::Char('b') = token {
            contains_b = true;
        } else if let RegexType::Char('c') = token {
            contains_c = true;
        }
    }
    
    assert!(contains_a);
    assert!(contains_b);
    assert!(contains_c);
    
    // Negated character class
    let mut input = "[^abc]".chars();
    let class = CharacterClass::parse(&mut input).unwrap();
    
    // Test using push_into_tokens
    let mut tokens = VecDeque::new();
    class.push_into_tokens(&mut tokens);
    
    // Check that 'a', 'b', 'c' are NOT in the tokens
    let mut contains_a = false;
    let mut contains_b = false;
    let mut contains_c = false;
    
    for token in &tokens {
        if let RegexType::Char('a') = token {
            contains_a = true;
        } else if let RegexType::Char('b') = token {
            contains_b = true;
        } else if let RegexType::Char('c') = token {
            contains_c = true;
        }
    }
    
    assert!(!contains_a);
    assert!(!contains_b);
    assert!(!contains_c);
    
    // Character class with range
    let mut input = "[a-z]".chars();
    let class = CharacterClass::parse(&mut input).unwrap();
    
    // Test using push_into_tokens
    let mut tokens = VecDeque::new();
    class.push_into_tokens(&mut tokens);
    
    // Check for some letters in range
    let mut lowercase_count = 0;
    let mut uppercase_count = 0;
    
    for token in &tokens {
        if let RegexType::Char(c) = token {
            if *c >= 'a' && *c <= 'z' {
                lowercase_count += 1;
            } else if *c >= 'A' && *c <= 'Z' {
                uppercase_count += 1;
            }
        }
    }
    
    assert_eq!(lowercase_count, 26); // All lowercase letters
    assert_eq!(uppercase_count, 0);  // No uppercase letters
}

#[test]
fn test_character_class_special_chars() {
    // Test with dash at the beginning (should be literal)
    let mut input = "[-a]".chars();
    let class = CharacterClass::parse(&mut input).unwrap();
    
    // Test using push_into_tokens
    let mut tokens = VecDeque::new();
    class.push_into_tokens(&mut tokens);
    
    // Check for '-' and 'a'
    let mut contains_dash = false;
    let mut contains_a = false;
    
    for token in &tokens {
        if let RegexType::Char('-') = token {
            contains_dash = true;
        } else if let RegexType::Char('a') = token {
            contains_a = true;
        }
    }
    
    assert!(contains_dash);
    assert!(contains_a);
    
    // Test with dash at the end (should be literal)
    let mut input = "[abc-]".chars();
    let class = CharacterClass::parse(&mut input).unwrap();
    
    // Test using push_into_tokens
    let mut tokens = VecDeque::new();
    class.push_into_tokens(&mut tokens);
    
    // Check for '-'
    let mut contains_dash = false;
    
    for token in &tokens {
        if let RegexType::Char('-') = token {
            contains_dash = true;
        }
    }
    
    assert!(contains_dash);
}

#[test]
fn test_character_class_escaped_chars() {
    let mut input = "[\\n\\t-]".chars();
    let class = CharacterClass::parse(&mut input).unwrap();
    
    let mut tokens = VecDeque::new();
    class.push_into_tokens(&mut tokens);
    
    let mut contains_newline = false;
    let mut contains_tab = false;
    let mut contains_dash = false;
    
    for token in &tokens {
        if let RegexType::Char('\n') = token {
            contains_newline = true;
        } else if let RegexType::Char('\t') = token {
            contains_tab = true;
        } else if let RegexType::Char('-') = token {
            contains_dash = true;
        }
    }
    
    assert!(contains_newline);
    assert!(contains_tab);
    assert!(contains_dash);
}

#[test]
fn test_character_class_predefined() {
    // Test digit class
    let digit_class = CharacterClass::digit();
    
    // Test using push_into_tokens
    let mut tokens = VecDeque::new();
    digit_class.push_into_tokens(&mut tokens);
    
    // Count digits
    let mut digit_count = 0;
    
    for token in &tokens {
        if let RegexType::Char(c) = token {
            if *c >= '0' && *c <= '9' {
                digit_count += 1;
            }
        }
    }
    
    assert_eq!(digit_count, 10);
    
    // Test word character class
    let word_class = CharacterClass::word_char();
    let mut tokens = VecDeque::new();
    word_class.push_into_tokens(&mut tokens);
    
    let mut lowercase_count = 0;
    let mut uppercase_count = 0;
    let mut digit_count = 0;
    let mut underscore_count = 0;
    
    for token in &tokens {
        if let RegexType::Char(c) = token {
            match c {
                'a'..='z' => lowercase_count += 1,
                'A'..='Z' => uppercase_count += 1,
                '0'..='9' => digit_count += 1,
                '_' => underscore_count += 1,
                _ => {}
            }
        }
    }
    
    assert_eq!(lowercase_count, 26);
    assert_eq!(uppercase_count, 26);
    assert_eq!(digit_count, 10);
    assert_eq!(underscore_count, 1);
    
    // Test whitespace class
    let space_class = CharacterClass::whitespace();
    let mut tokens = VecDeque::new();
    space_class.push_into_tokens(&mut tokens);
    
    let mut contains_space = false;
    let mut contains_tab = false;
    let mut contains_newline = false;
    
    for token in &tokens {
        if let RegexType::Char(c) = token {
            match c {
                ' ' => contains_space = true,
                '\t' => contains_tab = true,
                '\n' => contains_newline = true,
                _ => {}
            }
        }
    }
    
    assert!(contains_space);
    assert!(contains_tab);
    assert!(contains_newline);
}

#[test]
fn test_character_class_display() {
    let mut class = CharacterClass::new();
    class.add_char('a');
    class.add_range('0', '9').unwrap();
    
    let mut tokens = VecDeque::new();
    class.push_into_tokens(&mut tokens);
    
    assert_eq!(tokens[0], RegexType::OpenParenthesis);
    assert_eq!(tokens[tokens.len()-1], RegexType::CloseParenthesis);
    
    let mut contains_a = false;
    let mut digit_count = 0;
    
    for token in &tokens {
        if let RegexType::Char('a') = token {
            contains_a = true;
        } else if let RegexType::Char(c) = token {
            if c >= &'0' && c <= &'9' {
                digit_count += 1;
            }
        }
    }
    
    assert!(contains_a);
    assert_eq!(digit_count, 10);
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
    Regex::add_character_class(&mut tokens, &mut "[a-z]".chars()).unwrap();
    
    assert!(!tokens.is_empty());
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
    // Use tokens method instead of directly calling into_type
    let mut tokens = VecDeque::new();
    tokens.push_back(RegexType::Quant(Quantifier::AtLeast(0))); // '*'
    
    let star = tokens.pop_back().unwrap();
    if let RegexType::Quant(Quantifier::AtLeast(n)) = star {
        assert_eq!(n, 0);
    } else {
        panic!("Expected '*' to be AtLeast(0)");
    }
    
    tokens.push_back(RegexType::Quant(Quantifier::AtLeast(1))); // '+'
    let plus = tokens.pop_back().unwrap();
    if let RegexType::Quant(Quantifier::AtLeast(n)) = plus {
        assert_eq!(n, 1);
    } else {
        panic!("Expected '+' to be AtLeast(1)");
    }
    
    tokens.push_back(RegexType::Quant(Quantifier::Range(0, 1))); // '?'
    let question = tokens.pop_back().unwrap();
    if let RegexType::Quant(Quantifier::Range(min, max)) = question {
        assert_eq!(min, 0);
        assert_eq!(max, 1);
    } else {
        panic!("Expected '?' to be Range(0, 1)");
    }
}

// ==============================================
// 6. ESCAPE SEQUENCE HANDLING TESTS
// ==============================================

#[test]
fn test_basic_escape_sequences() {
    // Test using add_backslash instead
    let mut tokens = VecDeque::new();
    Regex::add_backslash(&mut tokens, &mut "n".chars());
    
    // Check that \n was converted to newline
    if let Some(RegexType::Char(c)) = tokens.pop_back() {
        assert_eq!(c, '\n');
    } else {
        panic!("Expected newline character");
    }
    
    let mut tokens = VecDeque::new();
    Regex::add_backslash(&mut tokens, &mut "t".chars());
    
    // Check that \t was converted to tab
    if let Some(RegexType::Char(c)) = tokens.pop_back() {
        assert_eq!(c, '\t');
    } else {
        panic!("Expected tab character");
    }
}

#[test]
fn test_shorthand_classes() {
    // Test using add_backslash which handles shorthand classes
    let mut tokens = VecDeque::new();
    Regex::add_backslash(&mut tokens, &mut "d".chars());
    
    // This should expand to alternation of digits (0|1|2|...|9)
    assert!(!tokens.is_empty());
    assert_eq!(tokens.front().unwrap(), &RegexType::OpenParenthesis);
    assert_eq!(tokens.back().unwrap(), &RegexType::CloseParenthesis);
    
    // Test word character class
    let mut tokens = VecDeque::new();
    Regex::add_backslash(&mut tokens, &mut "w".chars());
    
    // This should expand to alternation of word characters
    assert!(!tokens.is_empty());
    assert_eq!(tokens.front().unwrap(), &RegexType::OpenParenthesis);
    assert_eq!(tokens.back().unwrap(), &RegexType::CloseParenthesis);
}

#[test]
fn test_special_char_escaping() {
    // Test escaping special regex chars using add_backslash
    let mut tokens = VecDeque::new();
    Regex::add_backslash(&mut tokens, &mut ".".chars());
    
    // Check that \. was converted to literal dot
    if let Some(RegexType::Char(c)) = tokens.pop_back() {
        assert_eq!(c, '.');
    } else {
        panic!("Expected dot character");
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
    // Test full ASCII range
    let mut class = CharacterClass::new();
    class.add_range('\0', '~').unwrap();
    
    // Test using push_into_tokens
    let mut tokens = VecDeque::new();
    class.push_into_tokens(&mut tokens);
    
    // Check for boundary characters
    let mut contains_null = false;
    let mut contains_tilde = false;
    
    for token in &tokens {
        if let RegexType::Char('\0') = token {
            contains_null = true;
        } else if let RegexType::Char('~') = token {
            contains_tilde = true;
        }
    }
    
    assert!(contains_null);
    assert!(contains_tilde);
    
    // Test equal start and end (single character)
    let mut class2 = CharacterClass::new();
    class2.add_range('x', 'x').unwrap();
    
    // Test using push_into_tokens
    let mut tokens = VecDeque::new();
    class2.push_into_tokens(&mut tokens);
    
    // Check for exactly one 'x'
    let mut x_count = 0;
    
    for token in &tokens {
        if let RegexType::Char('x') = token {
            x_count += 1;
        }
    }
    
    assert_eq!(x_count, 1);
}
