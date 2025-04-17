use crate::parsing::definitions::{
    DefinitionType, Definitions, StateType, TableSizeDeclaration, TypeDeclaration,
};
use crate::parsing::error::ParsingResult;
use crate::parsing::reader::Reader;
use crate::parsing::Rules;
use std::io::Cursor;

// Helper function to create a Reader from a string
fn reader_from_str(s: &str) -> Reader<Cursor<Vec<u8>>> {
	Reader::new(Cursor::new(s.as_bytes().to_vec()), "<test>").expect("Failed to create reader")
}

#[test]
fn test_start_condition_extraction() {
	let mut reader = reader_from_str("STATE>");

	let result = Rules::extract_start_conditions(&mut reader).unwrap();

	assert_eq!(result.len(), 1);

	assert_eq!(&result[0], "STATE");
}

#[test]
fn test_multiple_start_conditions() {
	let mut reader = reader_from_str("STATE1,STATE2>");

	let result = Rules::extract_start_conditions(&mut reader).unwrap();

	assert_eq!(result.len(), 2);

	assert_eq!(&result[0], "STATE1");
	assert_eq!(&result[1], "STATE2");
}

#[test]
fn test_empty_start_condition() {
	let mut reader = reader_from_str(">");

	let result = Rules::extract_start_conditions(&mut reader);

	assert!(result.is_err());

	let err = result.unwrap_err();

	assert!(err.message().contains("empty condition"));
}

#[test]
fn test_invalid_multiple_start_conditions() {
	let mut reader = reader_from_str("STATE,>");

	let result = Rules::extract_start_conditions(&mut reader);

	assert!(result.is_err());

	let err = result.unwrap_err();

	assert!(err.message().contains("empty condition"));
}

#[test]
fn test_unclosed_start_conditions() {

	let check_if_unclosed = |str: &str| {
		let mut reader = reader_from_str(str);

		let result = Rules::extract_start_conditions(&mut reader);
		assert!(result.is_err());

		let err = result.unwrap_err();
		assert!(err.message().contains("unclosed start condition list"));
	};

	check_if_unclosed("STATE");
	check_if_unclosed("STATE1,STATE2");
}

#[test]
fn test_empty_str_start_conditions() {
	let mut reader = reader_from_str("");

	let result = Rules::extract_start_conditions(&mut reader);

	assert!(result.is_err());

	let err = result.unwrap_err();

	assert!(err.message().contains("unclosed start condition list"));
}

#[test]
#[allow(non_snake_case)]
fn test_non_iso_C_start_conditions() {
	let mut reader = reader_from_str("123>");

	let result = Rules::extract_start_conditions(&mut reader);

	assert!(result.is_err());

	let err = result.unwrap_err();

	assert!(err.message().contains("'1': invalid char in start condition"));
	assert!(err.message().contains("start conditions have to be iso-C normed"));
}

#[test]
fn test_whitespace_in_start_conditions() {
    let mut reader = reader_from_str("STATE1, STATE2 >");

    let result = Rules::extract_start_conditions(&mut reader);

	assert!(result.is_err());

	let err = result.unwrap_err();

	assert!(err.message().contains("' ': invalid char in start condition"));
	assert!(err.message().contains("start conditions have to be iso-C normed"));
}

#[test]
fn test_duplicate_start_conditions() {
    let mut reader = reader_from_str("STATE,STATE>");

    let result = Rules::extract_start_conditions(&mut reader).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(&result[0], "STATE");
}

#[test]
fn test_special_characters_in_start_conditions() {
    let mut reader = reader_from_str("STATE_1>");

    let result = Rules::extract_start_conditions(&mut reader).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(&result[0], "STATE_1");

	let mut reader = reader_from_str("STATE-2>");

	let result = Rules::extract_start_conditions(&mut reader).unwrap_err();

	assert!(result.message().contains("'-': invalid char in start condition"));
	assert!(result.message().contains("start conditions have to be iso-C normed"));
}

#[test]
fn test_max_length_start_condition() {
    let long_name = "A".repeat(100);
    let mut reader = reader_from_str(&format!("{}>", long_name));

    let result = Rules::extract_start_conditions(&mut reader).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0], long_name);
}

#[test]
fn test_mixed_case_start_conditions() {
    let mut reader = reader_from_str("State1,STATE2,state3>");

    let result = Rules::extract_start_conditions(&mut reader).unwrap();

    assert_eq!(result.len(), 3);
    assert_eq!(&result[0], "State1");
    assert_eq!(&result[1], "STATE2");
    assert_eq!(&result[2], "state3");
}

#[test]
fn test_trailing_comma() {
    let mut reader = reader_from_str("STATE1,STATE2,>");

    let result = Rules::extract_start_conditions(&mut reader);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message().contains("empty condition"));
}

#[test]
fn test_leading_comma() {
    let mut reader = reader_from_str(",STATE1,STATE2>");

    let result = Rules::extract_start_conditions(&mut reader);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message().contains("empty condition"));
}

#[test]
fn test_multiple_commas() {
    let mut reader = reader_from_str("STATE1,,STATE2>");

    let result = Rules::extract_start_conditions(&mut reader);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message().contains("empty condition"));
}

#[test]
fn test_read_one_regular_expression_with_slash_delimiter() {
    let mut reader = reader_from_str("[a-z]+/");
    
    let result = Rules::read_one_regular_expression(&mut reader).unwrap();
    
    assert_eq!(result, "[a-z]+");
}

#[test]
fn test_read_one_regular_expression_with_whitespace_delimiter() {
    let mut reader = reader_from_str("[0-9]+ ");
    
    let result = Rules::read_one_regular_expression(&mut reader).unwrap();
    
    assert_eq!(result, "[0-9]+");
}

#[test]
fn test_read_one_regular_expression_with_tab_delimiter() {
    let mut reader = reader_from_str("abc\t");
    
    let result = Rules::read_one_regular_expression(&mut reader).unwrap();
    
    assert_eq!(result, "abc");
}

#[test]
fn test_read_one_regular_expression_with_newline_delimiter() {
    let mut reader = reader_from_str("xyz\n");
    
    let result = Rules::read_one_regular_expression(&mut reader).unwrap();
    
    assert_eq!(result, "xyz");
}

#[test]
fn test_read_one_regular_expression_with_escaped_slash() {
    let mut reader = reader_from_str("a\\/b/");
    
    let result = Rules::read_one_regular_expression(&mut reader).unwrap();
    
    assert_eq!(result, "a\\/b");
}

#[test]
fn test_read_one_regular_expression_with_escaped_whitespace() {
    let mut reader = reader_from_str("a\\ b/");
    
    let result = Rules::read_one_regular_expression(&mut reader).unwrap();
    
    assert_eq!(result, "a\\ b");
}

#[test]
fn test_read_one_regular_expression_complex() {
    let mut reader = reader_from_str("[a-zA-Z_][a-zA-Z0-9_]*\\(.*\\) ");
    
    let result = Rules::read_one_regular_expression(&mut reader).unwrap();
    
    assert_eq!(result, "[a-zA-Z_][a-zA-Z0-9_]*\\(.*\\)");
}

#[test]
fn test_read_one_regular_expression_empty() {
    let mut reader = reader_from_str("/");
    
    let result = Rules::read_one_regular_expression(&mut reader).unwrap();
    
    assert_eq!(result, "");
}

#[test]
fn test_read_one_regular_expression_unclosed() {
    let mut reader = reader_from_str("[a-z]+");
    
    let result = Rules::read_one_regular_expression(&mut reader);
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message().contains("unexpected end of file"));
}

#[test]
fn test_read_one_regular_expression_with_multiple_escapes() {
    let mut reader = reader_from_str("\\[\\]\\(\\)\\*\\+\\?\\/\\\\/ ");
    
    let result = Rules::read_one_regular_expression(&mut reader).unwrap();
    
    assert_eq!(result, "\\[\\]\\(\\)\\*\\+\\?\\/\\\\");
}

#[test]
fn test_read_one_regular_expression_with_character_classes() {
    let mut reader = reader_from_str("[\\d\\a]+/");
    
    let result = Rules::read_one_regular_expression(&mut reader).unwrap();
    
    assert_eq!(result, "[\\d\\a]+");
}

#[test]
fn test_read_one_regular_expression_no_input() {
    let mut reader = reader_from_str("");
    
    let result = Rules::read_one_regular_expression(&mut reader);
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message().contains("unexpected end of file"));
}

#[test]
fn test_get_regular_expression_simple() {
    let mut reader = reader_from_str("[a-z]+ ");
    
    let result = Rules::get_regular_expression(&mut reader).unwrap();
    
    assert_eq!(result.0, "[a-z]+");
    assert_eq!(result.1, None);
}

#[test]
fn test_get_regular_expression_with_tab() {
    let mut reader = reader_from_str("abc\t");
    
    let result = Rules::get_regular_expression(&mut reader).unwrap();
    
    assert_eq!(result.0, "abc");
    assert_eq!(result.1, None);
}

#[test]
fn test_get_regular_expression_with_newline() {
    let mut reader = reader_from_str("xyz\n");
    
    let result = Rules::get_regular_expression(&mut reader).unwrap();
    
    assert_eq!(result.0, "xyz");
    assert_eq!(result.1, None);
}

#[test]
fn test_get_regular_expression_with_escaped_whitespace() {
    let mut reader = reader_from_str("a\\ b\\ c ");
    
    let result = Rules::get_regular_expression(&mut reader).unwrap();
    
    assert_eq!(result.0, "a\\ b\\ c");
    assert_eq!(result.1, None);
}

#[test]
fn test_get_regular_expression_with_escaped_slash() {
    let mut reader = reader_from_str("a\\/b ");
    
    let result = Rules::get_regular_expression(&mut reader).unwrap();
    
    assert_eq!(result.0, "a\\/b");
    assert_eq!(result.1, None);
}

#[test]
fn test_get_regular_expression_complex() {
    let mut reader = reader_from_str("[a-zA-Z_][a-zA-Z0-9_]*\\(.*\\) ");
    
    let result = Rules::get_regular_expression(&mut reader).unwrap();
    
    assert_eq!(result.0, "[a-zA-Z_][a-zA-Z0-9_]*\\(.*\\)");
    assert_eq!(result.1, None);
}

#[test]
fn test_get_regular_expression_with_quantifiers() {
    let mut reader = reader_from_str("a+b*c? ");
    
    let result = Rules::get_regular_expression(&mut reader).unwrap();
    
    assert_eq!(result.0, "a+b*c?");
    assert_eq!(result.1, None);
}

#[test]
fn test_get_regular_expression_with_groups() {
    let mut reader = reader_from_str("(ab|cd)+ ");
    
    let result = Rules::get_regular_expression(&mut reader).unwrap();
    
    assert_eq!(result.0, "(ab|cd)+");
    assert_eq!(result.1, None);
}

#[test]
fn test_get_regular_expression_with_character_sets() {
    let mut reader = reader_from_str("[^abc][0-9] ");
    
    let result = Rules::get_regular_expression(&mut reader).unwrap();
    
    assert_eq!(result.0, "[^abc][0-9]");
    assert_eq!(result.1, None);
}

#[test]
fn test_get_regular_expression_with_special_chars() {
    let mut reader = reader_from_str("\\w+\\d+\\s+ ");
    
    let result = Rules::get_regular_expression(&mut reader).unwrap();
    
    assert_eq!(result.0, "\\w+\\d+\\s+");
    assert_eq!(result.1, None);
}

#[test]
fn test_get_regular_expression_empty() {
    let mut reader = reader_from_str(" ");
    
    let result = Rules::get_regular_expression(&mut reader).unwrap();
    
    assert_eq!(result.0, "");
    assert_eq!(result.1, None);
}

#[test]
fn test_get_regular_expression_no_whitespace() {
    let mut reader = reader_from_str("abc");
    
    let result = Rules::get_regular_expression(&mut reader);
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message().contains("unexpected end of file"));
}

#[test]
fn test_get_regular_expression_with_slash_delimiter() {
    let mut reader = reader_from_str("abc/");
    
    let result = Rules::get_regular_expression(&mut reader);
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message().contains("unexpected end of file"));
    assert!(err.message().contains("unclosed regular expression"));
}

#[test]
fn test_get_regular_expression_no_input() {
    let mut reader = reader_from_str("");
    
    let result = Rules::get_regular_expression(&mut reader);
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message().contains("unexpected end of file"));
}

#[test]
fn test_get_regular_expression_with_section() {
    let mut reader = reader_from_str("first/second ");
    
    let result = Rules::get_regular_expression(&mut reader).unwrap();
    
    assert_eq!(result.0, "first");
    assert_eq!(result.1, Some("second".to_string()));
}

#[test]
fn test_get_regular_expression_with_duplicate_slash() {
    let mut reader = reader_from_str("first/second/");
    
    let result = Rules::get_regular_expression(&mut reader);
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message().contains("duplicate '/'"));
}

#[test]
fn test_read_entire_block_simple() {
    let mut reader = reader_from_str("{ simple block }");
    
    let result = Rules::read_entire_block(&mut reader).unwrap();
    
    assert_eq!(result, "{ simple block }");
}

#[test]
fn test_read_entire_block_empty() {
    let mut reader = reader_from_str("{}");
    
    let result = Rules::read_entire_block(&mut reader).unwrap();
    
    assert_eq!(result, "{}");
}

#[test]
fn test_read_entire_block_with_nested_braces() {
    let mut reader = reader_from_str("{ outer { inner } block }");
    
    let result = Rules::read_entire_block(&mut reader).unwrap();
    
    assert_eq!(result, "{ outer { inner } block }");
}

#[test]
fn test_read_entire_block_with_escaped_braces() {
    let mut reader = reader_from_str("{ block with \\{ escaped \\} braces }");

    let result = Rules::read_entire_block(&mut reader).unwrap();
    
    assert_eq!(result, "{ block with \\{ escaped \\} braces }");
}

#[test]
fn test_read_entire_block_multiline() {
    let mut reader = reader_from_str("{\n    multiline\n    block\n}");
    
    let result = Rules::read_entire_block(&mut reader).unwrap();
    
    assert_eq!(result, "{\n    multiline\n    block\n}");
}

#[test]
fn test_read_entire_block_with_special_chars() {
    let mut reader = reader_from_str("{ block with special chars: !@#$%^&*()_+-=[] }");
    
    let result = Rules::read_entire_block(&mut reader).unwrap();
    
    assert_eq!(result, "{ block with special chars: !@#$%^&*()_+-=[] }");
}

#[test]
fn test_read_entire_block_unclosed() {
    let mut reader = reader_from_str("{ unclosed block");
    
    let result = Rules::read_entire_block(&mut reader);
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message().contains("unclosed block"));
}

#[test]
fn test_read_entire_block_no_opening_brace() {
    let mut reader = reader_from_str("no opening brace }");
    
    let result = Rules::read_entire_block(&mut reader);
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message().contains("expected '{'"));
}

#[test]
fn test_read_entire_block_nested_unclosed() {
    let mut reader = reader_from_str("{ outer { inner block }");
    
    let result = Rules::read_entire_block(&mut reader);
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message().contains("unclosed block"));
}

#[test]
fn test_read_entire_block_empty_input() {
    let mut reader = reader_from_str("");
    
    let result = Rules::read_entire_block(&mut reader);
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message().contains("unexpected end of file"));
}