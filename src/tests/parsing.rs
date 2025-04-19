use std::io::{Cursor, Read};
use crate::parsing::{Parsing, Reader, Section, ParsingError};

/// Create a `Reader` from a string for testing purposes
fn create_reader(content: &str) -> Reader<Cursor<Vec<u8>>> {
    Reader::new(Cursor::new(content.as_bytes().to_vec()), "<test>").unwrap()
}

#[test]
fn test_parse_sections_empty() {
    let mut parsing = Parsing::new().unwrap();
    let mut reader = create_reader("");
    
    let result = parsing.parse_sections(&mut reader);
    assert!(result.is_ok());
    assert_eq!(parsing.section, Section::Subroutines);
    assert!(parsing.rules.is_empty());
    assert!(parsing.user_subroutines.is_none());
}

#[test]
fn test_parse_sections_definitions_only() {
    let mut parsing = Parsing::new().unwrap();
    let mut reader = create_reader("DIGIT [0-9]\nLETTER [a-zA-Z]");
    
    let result = parsing.parse_sections(&mut reader);
    assert!(result.is_ok());
    assert_eq!(parsing.section, Section::Subroutines);
    
    // Verify definitions were parsed
    assert!(parsing.definitions.substitutes.contains_key("DIGIT"));
    assert!(parsing.definitions.substitutes.contains_key("LETTER"));
}

#[test]
fn test_parse_sections_with_rules() {
    let mut parsing = Parsing::new().unwrap();
    let content = "DIGIT [0-9]\n%%\n{DIGIT}+ { return NUMBER; }";
    let mut reader = create_reader(content);
    
    let result = parsing.parse_sections(&mut reader);
    assert!(result.is_ok());
    assert_eq!(parsing.section, Section::Subroutines);
    
    // Verify rules were parsed
    assert_eq!(parsing.rules.len(), 1);
    assert_eq!(parsing.rules[0].regex_nfa.borrow().to_string(), "[0123456789]+");
}

#[test]
fn test_parse_sections_complete() {
    let mut parsing = Parsing::new().unwrap();
    let content = "DIGIT [0-9]\n%%\n{DIGIT}+ { return NUMBER; }\n%%\nvoid user_code() {\n    // test code\n}";
    let mut reader = create_reader(content);
    
    let result = parsing.parse_sections(&mut reader);
    assert!(result.is_ok());
    assert_eq!(parsing.section, Section::Subroutines);
    
    // Verify all sections were parsed
    assert!(parsing.definitions.substitutes.contains_key("DIGIT"));
    assert_eq!(parsing.rules.len(), 1);
    assert!(parsing.user_subroutines.is_some());
    assert!(parsing.user_subroutines.as_ref().unwrap().contains("void user_code()"));
}

#[test]
fn test_parse_sections_invalid_definition() {
    let mut parsing = Parsing::new().unwrap();
    let content = "DIGIT [0-9\n%%"; // Missing closing bracket
    let mut reader = create_reader(content);
    
    let result = parsing.parse_sections(&mut reader);
    assert!(result.is_err());
    assert!(!parsing.errors.is_empty());
}

#[test]
fn test_parse_sections_invalid_rule() {
    let mut parsing = Parsing::new().unwrap();
    let content = "DIGIT [0-9]\n%%\n{DIGIT+ { return NUMBER; }"; // Missing closing brace in pattern
    let mut reader = create_reader(content);
    
    let result = parsing.parse_sections(&mut reader);
    assert!(result.is_err());
    assert!(!parsing.errors.is_empty());
}

#[test]
fn test_parse_sections_skip_errors() {
    let mut parsing = Parsing::new().unwrap();
    let content = "INVALID [0-9\nVALID [a-z]\n%%\n{VALID} { return LETTER; }";
    let mut reader = create_reader(content);

    // Should still parse the valid parts
    let result = parsing.parse_sections(&mut reader);
    assert!(result.is_err()); // Because it encountered errors
	dbg!(&parsing.errors);
    assert!(parsing.definitions.substitutes.contains_key("VALID"));
    assert_eq!(parsing.rules.len(), 1);
}
