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
	check_if_unclosed("STATE ");
	check_if_unclosed("STATE\n");
	check_if_unclosed("STATE\t");
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

	assert!(err.message().contains("'1': invalid first char"));
	assert!(err.message().contains("start conditions have to be iso-C normed"));
}