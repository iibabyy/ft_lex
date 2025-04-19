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
    assert!(result.is_err());
}

#[test]
fn test_parse_definitions_empty() {
    let mut parsing = Parsing::new().unwrap();
    let mut reader = create_reader("%%");
    
	assert!(parsing.parse_sections(&mut reader).is_ok());

	assert_eq!(parsing.errors.len(), 0);
}

#[test]
fn test_parse_sections_definitions_only() {
    let mut parsing = Parsing::new().unwrap();
    let mut reader = create_reader("DIGIT [0-9]\nLETTER [a-zA-Z]\n%%");
    
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
    assert!(parsing.rules[0].regex_nfa.borrow().to_string().contains("[0123456789]"));
	
	let split_out_1 = parsing.rules[0].regex_nfa.borrow().basic_out().unwrap().borrow().borrow().split_out().unwrap().0.borrow().borrow().to_string();
    assert!(split_out_1.contains("[0123456789]"));

	let split_out_2 = parsing.rules[0].regex_nfa.borrow().basic_out().unwrap().borrow().borrow().split_out().unwrap().1.borrow().borrow().to_string();
	assert!(split_out_2.contains("Match"));
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
fn test_parse_sections_invalid_rule() {
    let mut parsing = Parsing::new().unwrap();
    let content = "DIGIT [0-9]\n%%\n{DIGIT+ { return NUMBER; }"; // Missing closing brace in pattern
    let mut reader = create_reader(content);
    
    let result = parsing.parse_sections(&mut reader);
    assert!(result.is_err());
    assert_eq!(parsing.errors.len(), 1);
}
