use super::*;
use std::{
    collections::{HashMap, HashSet},
    mem::{self, take},
};

/// Represents the type of a lexer state.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StateType {
    /// Exclusive state: only one rule can match
    Exclusive,
    /// Inclusive state: multiple rules can match
    Inclusive,
}

impl ToString for StateType {
    fn to_string(&self) -> String {
        match self {
            StateType::Exclusive => "exclusive",
            StateType::Inclusive => "inclusive",
        }
        .to_string()
    }
}

impl TryFrom<&str> for StateType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "s" | "S" => Ok(StateType::Inclusive),
            "x" | "X" => Ok(StateType::Exclusive),
            _ => Err(()),
        }
    }
}

/// Collection of all lexer definitions including substitutions, fragments, and declarations.
#[derive(Debug)]
pub struct Definitions {
    /// Map of name to substitution text
    pub substitutes: HashMap<String, String>,

    /// List of program fragments
    pub fragments: Vec<String>,

    /// Declaration of yytext type (array or pointer)
    pub type_declaration: Option<TypeDeclaration>,

    /// Map of table size declarations to their values
    pub table_sizes: HashMap<TableSizeDeclaration, usize>,

    /// Map of state names to their types
    pub states: HashMap<String, StateType>,
}

/// Represents different types of definitions that can appear in the definitions section.
pub enum DefinitionType {
    /// Table size declaration (e.g., "%p 100")
    TableSize(TableSizeDeclaration, usize),
    /// Name substitution (e.g., "name text")
    Substitute(String, String),
    /// Program fragment (either inline or block)
    Fragment(String),
    /// yytext type declaration (array or pointer)
    TypeDeclaration(TypeDeclaration),
    /// State declaration with type and names
    StateDeclaration(StateType, Vec<String>),
    /// Empty line
    Empty,
    /// End of definitions section marker ("%%")
    EndOfSection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TableSizeDeclaration {
    Positions,
    Transitions,
    Statesets,
    Equivalence,
    Characters,
    Outputfiles,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeDeclaration {
    Array,
    Pointer,
}

impl TryFrom<String> for TableSizeDeclaration {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for TableSizeDeclaration {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "p" => Ok(Self::Positions),
            "n" => Ok(Self::Transitions),
            "e" => Ok(Self::Equivalence),
            "a" => Ok(Self::Statesets),
            "k" => Ok(Self::Characters),
            "o" => Ok(Self::Outputfiles),
            _ => Err(()),
        }
    }
}

impl ToString for TableSizeDeclaration {
    fn to_string(&self) -> String {
        match self {
            TableSizeDeclaration::Positions => "%p",
            TableSizeDeclaration::Transitions => "%n",
            TableSizeDeclaration::Equivalence => "%e",
            TableSizeDeclaration::Statesets => "%a",
            TableSizeDeclaration::Characters => "%k",
            TableSizeDeclaration::Outputfiles => "%o",
        }
        .to_string()
    }
}

impl TryFrom<String> for TypeDeclaration {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for TypeDeclaration {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "array" => Ok(Self::Array),
            "pointer" => Ok(Self::Pointer),
            _ => Err(()),
        }
    }
}

impl ToString for TypeDeclaration {
    fn to_string(&self) -> String {
        match self {
            TypeDeclaration::Array => "array",
            TypeDeclaration::Pointer => "pointer",
        }
        .to_string()
    }
}

impl Definitions {
    /// Creates a new empty definitions collection.
    pub fn new() -> Self {
        Self {
            substitutes: HashMap::new(),
            fragments: Vec::new(),
            states: HashMap::new(),
            table_sizes: HashMap::new(),
            type_declaration: None,
        }
    }

    /// Parses the definitions section of a lexer file.
    ///
    /// This function handles all types of definitions:
    /// - Table size declarations (%p, %n, etc.)
    /// - Name substitutions
    /// - Program fragments (inline and block)
    /// - Type declarations (%array, %pointer)
    /// - State declarations (%s, %x)
    ///
    /// Returns an error if any definition is invalid or if the section delimiter is missing.
    pub fn parse<'de, R: Read>(
        &'de mut self,
        reader: &mut Reader<R>,
    ) -> ParsingResult<&'de mut Self> {
        loop {
            match Self::line_type(reader)? {
                DefinitionType::TableSize(table, size) => {
                    if let Some(previous_size) = self.table_sizes.insert(table, size) {
                        eprintln!("Warning: Duplicate table size declaration for {} : previous value ({}) replaced by {}",
                            table.to_string(), previous_size, size
                        )
                    }
                }
                DefinitionType::Substitute(name, substitute) => {
                    if let Some(previous_substitute) =
                        self.substitutes.insert(name.clone(), substitute.clone())
                    {
                        eprintln!("Warning: Duplicate Substitution declaration for {} : previous value ({}) replaced by {}",
                            name, previous_substitute, substitute
                        )
                    }
                }
                DefinitionType::StateDeclaration(state_type, states_names) => {
                    for name in states_names {
                        if let Some(_) = self.states.insert(name.clone(), state_type) {
                            // Duplicate Value
                            return Err(ParsingError::syntax(format!(
                                "start condition {} declared twice",
                                name
                            )));
                        }
                    }
                }
                DefinitionType::Fragment(fragment) => {
                    self.fragments.push(fragment);
                }
                DefinitionType::TypeDeclaration(type_decla) => {
                    if self.type_declaration.is_some() && self.type_declaration != Some(type_decla)
                    {
                        eprintln!("Warning: Duplicate type declaration declaration : previous value (%{}) replaced by %{}",
                            self.type_declaration.unwrap().to_string(), type_decla.to_string()
                        )
                    }
                    self.type_declaration = Some(type_decla)
                }
                DefinitionType::Empty => {}
                DefinitionType::EndOfSection => return Ok(self),
            }
        }
    }

    /// Determines the type of definition from a line of input.
    ///
    /// This function handles all possible definition formats:
    /// - Lines starting with space: inline program fragments
    /// - Lines starting with name: substitutions
    /// - Lines starting with %: declarations and block fragments
    /// - Empty lines
    /// - Section delimiter
    fn line_type<R: Read>(reader: &mut Reader<R>) -> ParsingResult<DefinitionType> {
        let line = reader.line()?.ok_or(ParsingError::end_of_file())?;

        if line.is_empty() {
            return Ok(DefinitionType::Empty);
        }

        if line == "%%" {
            // Section delimiter found - end of definition section
            return Ok(DefinitionType::EndOfSection);
        }

        let mut chars = line.chars();
        let first_char = chars.next().unwrap();

        // Line Program Fragment: lines that start with a space
        // This is C code that will be included directly in the output
        if first_char == ' ' || first_char == '\t' {
            return Ok(DefinitionType::Fragment(line[1..].to_string()));
        }

        // Substitution Chains: lines that start with an identifier
        // Format: name substitute_text
        if first_char.is_alphabetic() || first_char == '_' {
            // Split into name and substitute text (first whitespace only)
            let split = Utils::split_whitespace_once(&line)
                .ok_or(ParsingError::syntax("incomplete name definition"))?;

            // Validate that the name follows C naming conventions
            if !Utils::is_iso_C_normed(split.0) {
                return Err(ParsingError::syntax(format!("`{}`", split.0))
                    .because("name must be iso-C normed"));
            }

            return Ok(DefinitionType::Substitute(
                split.0.to_string(),
                split.1.to_string(),
            ));
        }

        // Block Program Fragments: multi-line C code blocks
        // Format: %{ ... %}
        if line.starts_with("%{") {
            let open_dilimiter_index = reader.index();

            // Read all lines until closing delimiter %}
            let (content, found) = Utils::read_until_line("%}", reader)?;

            // Check if the closing delimiter was found or if we reached EOF
            if !found {
                return Err(ParsingError::end_of_file()
                    .because(format!("expected close matching delimiter for open delimiter at line {open_dilimiter_index}"))
                );
            }

            // Join the lines with newlines and add extra newlines at start and end
            // This ensures proper separation in the generated code
            let mut content = content.join("\n");
            content.insert(0, '\n');
            content.push('\n');

            // If something after %{, insert it at the beginning
            if line.len() > 2 {
                content.insert_str(0, &line[2..]);
            }

            return Ok(DefinitionType::Fragment(content));
        }

        // The remaining cases all start with '%' - flag-based declarations
        if first_char != '%' {
            return Err(ParsingError::unexpected_token(first_char));
        }

        // Split the line into words after the % character
        let mut split: Vec<String> = line[1..]
            .split_ascii_whitespace()
            .map(|str| str.to_string())
            .collect();

        if split.is_empty() {
            return Ok(DefinitionType::Empty);
        }

        // Take the first word as the flag (removing it from split)
        let flag = take(&mut split[0]);

        match flag.as_str() {
            // State declarations (%s for inclusive, %x for exclusive)
            "s" | "S" | "x" | "X" => {
                if split.len() < 2 {
                    return Err(
                        ParsingError::end_of_line().because(format!("`%{flag} {{STATE_NAME}}`"))
                    );
                }

                let states_type = StateType::try_from(flag.as_str()).unwrap();

                // First element is now empty after take(), remove it
                split.remove(0);

                // Validate that all state names follow C naming conventions
                for name in &split {
                    if !Utils::is_iso_C_normed(name) {
                        return Err(ParsingError::syntax(format!("`{name}`"))
                            .because("states must be iso-C normed"));
                    }
                }

                return Ok(DefinitionType::StateDeclaration(states_type, split));
            }
            // Table size declarations (%p, %n, %a, %e, %k, %o followed by a number)
            "p" | "n" | "a" | "e" | "k" | "o" => {
                // Ensure the format is correct: %flag number
                Self::check_split_size(&split, 2, format!("`%{flag} {{POSITIVE_NUMBER}}`"))?;

                // Parse the size as a positive number
                let size = split[1].as_str().parse::<usize>().map_err(|err| {
                    ParsingError::invalid_number(&split[1]).because(err.to_string())
                })?;

                return Ok(DefinitionType::TableSize(
                    TableSizeDeclaration::try_from(flag).unwrap(),
                    size,
                ));
            }
            // Type declarations (%array or %pointer)
            "array" | "pointer" => {
                // Ensure there are no unexpected tokens after the type
                Self::check_split_size(&split, 1, format!("`%{flag}`"))?;
                return Ok(DefinitionType::TypeDeclaration(
                    TypeDeclaration::try_from(flag).unwrap(),
                ));
            }
            // Any other flag is an error
            _ => return Err(ParsingError::invalid_flag(format!("%{flag}"))),
        }
    }

    /// Splits a line into parts and verifies it has the expected number of parts.
    fn split_definition(
        line: &String,
        expected: usize,
        expected_err_msg: impl ToString,
    ) -> ParsingResult<Vec<String>> {
        let split: Vec<String> = line
            .split_ascii_whitespace()
            .map(|str| str.to_string())
            .collect();
        let expected_err_msg = expected_err_msg.to_string();

        Self::check_split_size(&split, expected, expected_err_msg)?;

        Ok(split)
    }

    /// Verifies that a split line has the expected number of parts.
    ///
    /// Returns an error if:
    /// - The line has fewer parts than expected
    /// - The line has more parts than expected
    pub fn check_split_size(
        split: &Vec<String>,
        expected: usize,
        expected_err_msg: impl ToString,
    ) -> ParsingResult<()> {
        let expected_err_msg = expected_err_msg.to_string();

        if split.len() < expected {
            return Err(
                ParsingError::end_of_line().because(format!("expected: {expected_err_msg}"))
            );
        }

        if split.len() > expected {
            return Err(ParsingError::unexpected_token(&split[expected])
                .because("expected")
                .because(expected_err_msg));
        }

        Ok(())
    }

    /// Checks if a character is a valid description section flag.
    pub fn is_valid_description_flag(c: char) -> bool {
        // Program Fragment
        if c == '{' {
            return true;
        }

        // Table Size Declaration
        if c == 'p' || c == 'n' || c == 'a' || c == 'e' || c == 'k' || c == 'o' {
            return true;
        }

        // State
        if c == 's' || c == 'S' || c == 'x' || c == 'X' {
            return true;
        }

        false
    }
}
