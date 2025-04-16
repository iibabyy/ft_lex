use crate::parsing::definitions::{
    DefinitionType, Definitions, StateType, TableSizeDeclaration, TypeDeclaration,
};
use crate::parsing::error::ParsingResult;
use crate::parsing::reader::Reader;
use std::io::Cursor;

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a Reader from a string
    fn reader_from_str(s: &str) -> Reader<Cursor<Vec<u8>>> {
        Reader::new(Cursor::new(s.as_bytes().to_vec()), "<test>").expect("Failed to create reader")
    }

    #[test]
    fn test_state_type_to_string() {
        assert_eq!(StateType::Exclusive.to_string(), "exclusive");
        assert_eq!(StateType::Inclusive.to_string(), "inclusive");
    }

    #[test]
    fn test_state_type_try_from() {
        assert_eq!(StateType::try_from("s"), Ok(StateType::Inclusive));
        assert_eq!(StateType::try_from("S"), Ok(StateType::Inclusive));
        assert_eq!(StateType::try_from("x"), Ok(StateType::Exclusive));
        assert_eq!(StateType::try_from("X"), Ok(StateType::Exclusive));
        assert!(StateType::try_from("a").is_err());
    }

    #[test]
    fn test_empty_definitions() {
        let defs = Definitions::new();
        assert!(defs.substitutes.is_empty());
        assert!(defs.table_sizes.is_empty());
        assert!(defs.type_declaration.is_none());
        assert!(defs.fragments.is_empty());

        assert_eq!(defs.states.len(), 1);
		assert!(defs.states.get("INITIAL").is_some());
    }

    #[test]
    fn test_parse_empty_definitions_section() -> ParsingResult<()> {
        let mut defs = Definitions::new();
        let input = "%%\n";
        let mut reader = reader_from_str(input);

        defs.parse(&mut reader)?;
        // Successfully parsed the empty section
        assert!(defs.substitutes.is_empty());
        assert!(defs.fragments.is_empty());
        assert_eq!(defs.states.len(), 1);

		assert!(defs.states.get("INITIAL").is_some());

        Ok(())
    }

    #[test]
    fn test_parse_name_substitution() -> ParsingResult<()> {
        let mut defs = Definitions::new();
        let input = "DIGIT [0-9]\n%%\n";
        let mut reader = reader_from_str(input);

        defs.parse(&mut reader)?;

        // Check the substitution was parsed correctly
        assert_eq!(defs.substitutes.get("DIGIT"), Some(&"[0-9]".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_invalid_name_substitution() {
        let mut defs = Definitions::new();
        let input = "123DIGIT [0-9]\n%%\n";
        let mut reader = reader_from_str(input);

        let result = defs.parse(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_program_fragment_inline() -> ParsingResult<()> {
        let mut defs = Definitions::new();
        let input = " int counter = 0;\n%%\n";
        let mut reader = reader_from_str(input);

        defs.parse(&mut reader)?;

        assert_eq!(defs.fragments.len(), 1);
        assert_eq!(defs.fragments[0], "int counter = 0;");
        Ok(())
    }

    #[test]
    fn test_parse_program_fragment_block() -> ParsingResult<()> {
        let mut defs = Definitions::new();
        let input = "%{\n  #include <stdio.h>\n  int counter = 0;\n%}\n%%\n";
        let mut reader = reader_from_str(input);

        defs.parse(&mut reader)?;

        assert_eq!(defs.fragments.len(), 1);
        assert!(defs.fragments[0].contains("#include <stdio.h>"));
        assert!(defs.fragments[0].contains("int counter = 0;"));
        Ok(())
    }

    #[test]
    fn test_parse_unclosed_program_fragment_block() {
        let mut defs = Definitions::new();
        let input = "%{\n  #include <stdio.h>\n  int counter = 0;\n";
        let mut reader = reader_from_str(input);

        let result = defs.parse(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_state_declaration() -> ParsingResult<()> {
        let mut defs = Definitions::new();
        let input = "%s STATE1 STATE2\n%%\n";
        let mut reader = reader_from_str(input);

        defs.parse(&mut reader)?;

        assert_eq!(defs.states.get("STATE1"), Some(&StateType::Inclusive));
        assert_eq!(defs.states.get("STATE2"), Some(&StateType::Inclusive));
        Ok(())
    }

    #[test]
    fn test_parse_exclusive_state_declaration() -> ParsingResult<()> {
        let mut defs = Definitions::new();
        let input = "%x STATE1 STATE2\n%%\n";
        let mut reader = reader_from_str(input);

        defs.parse(&mut reader)?;

        assert_eq!(defs.states.get("STATE1"), Some(&StateType::Exclusive));
        assert_eq!(defs.states.get("STATE2"), Some(&StateType::Exclusive));
        Ok(())
    }

    #[test]
    fn test_parse_duplicate_state_declaration() {
        let mut defs = Definitions::new();
        let input = "%s STATE1\n%s STATE1\n%%\n";
        let mut reader = reader_from_str(input);

        let result = defs.parse(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_table_size_declaration() -> ParsingResult<()> {
        let mut defs = Definitions::new();
        let input = "%p 5000\n%%\n";
        let mut reader = reader_from_str(input);

        defs.parse(&mut reader)?;

        assert_eq!(
            defs.table_sizes.get(&TableSizeDeclaration::Positions),
            Some(&5000)
        );
        Ok(())
    }

    #[test]
    fn test_parse_invalid_table_size() {
        let mut defs = Definitions::new();
        let input = "%p -100\n%%\n"; // Negative number should be invalid
        let mut reader = reader_from_str(input);

        let result = defs.parse(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_type_declaration() -> ParsingResult<()> {
        let mut defs = Definitions::new();
        let input = "%array\n%%\n";
        let mut reader = reader_from_str(input);

        defs.parse(&mut reader)?;

        assert_eq!(defs.type_declaration, Some(TypeDeclaration::Array));
        Ok(())
    }

    #[test]
    fn test_parse_conflicting_type_declarations() -> ParsingResult<()> {
        let mut defs = Definitions::new();
        let input = "%array\n%pointer\n%%\n";
        let mut reader = reader_from_str(input);

        // This should succeed but with a warning (which we can't easily verify)
        defs.parse(&mut reader)?;

        // The second declaration should override the first
        assert_eq!(defs.type_declaration, Some(TypeDeclaration::Pointer));
        Ok(())
    }

    #[test]
    fn test_parse_combined_definitions() -> ParsingResult<()> {
        let mut defs = Definitions::new();
        let input = r#"
DIGIT       [0-9]
ALPHA       [a-zA-Z]
ALPHANUM    [a-zA-Z0-9]

%{
    #include <stdio.h>
    int line_num = 1;
%}

%s STRING COMMENT
%x EXCLUSIVE

%pointer

%p 5000
%%

        "#; // Note the extra newline at the end to ensure proper parsing
        let mut reader = reader_from_str(input);

        defs.parse(&mut reader)?;

        // Verify each component
        assert_eq!(defs.substitutes.get("DIGIT"), Some(&"[0-9]".to_string()));
        assert_eq!(defs.substitutes.get("ALPHA"), Some(&"[a-zA-Z]".to_string()));
        assert_eq!(
            defs.substitutes.get("ALPHANUM"),
            Some(&"[a-zA-Z0-9]".to_string())
        );

        assert!(!defs.fragments.is_empty());
        assert!(defs.fragments[0].contains("#include <stdio.h>"));

        assert_eq!(defs.states.get("STRING"), Some(&StateType::Inclusive));
        assert_eq!(defs.states.get("COMMENT"), Some(&StateType::Inclusive));
        assert_eq!(defs.states.get("EXCLUSIVE"), Some(&StateType::Exclusive));

        assert_eq!(defs.type_declaration, Some(TypeDeclaration::Pointer));
        assert_eq!(
            defs.table_sizes.get(&TableSizeDeclaration::Positions),
            Some(&5000)
        );

        Ok(())
    }

    #[test]
    fn test_invalid_flag() {
        let mut defs = Definitions::new();
        let input = "%invalid 123\n%%\n";
        let mut reader = reader_from_str(input);

        let result = defs.parse(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_split_size() {
        // Testing the check_split_size utility function
        let split = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        // Correct size
        assert!(Definitions::check_split_size(&split, 3, "test").is_ok());

        // Too few elements
        assert!(Definitions::check_split_size(&split, 4, "test").is_err());

        // Too many elements
        assert!(Definitions::check_split_size(&split, 2, "test").is_err());
    }

    #[test]
    fn test_is_valid_description_flag() {
        // Now we can test the public method
        assert!(Definitions::is_valid_description_flag('{'));
        assert!(Definitions::is_valid_description_flag('p'));
        assert!(Definitions::is_valid_description_flag('s'));
        assert!(Definitions::is_valid_description_flag('x'));
        assert!(!Definitions::is_valid_description_flag('z'));
    }
}
