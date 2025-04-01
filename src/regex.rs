mod re2post;
mod nfa;

use std::{collections::VecDeque, fmt, ops, str::Chars};
use crate::parsing::{error::{ParsingError, ParsingResult}, utils::Utils};

use super::*;

// 1. BASIC TYPE DEFINITIONS
// =========================

pub struct Regex {
    operator_stack: Vec<RegexType>,
    output_stack: Vec<RegexType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegexType {
    Char(char),
    QuestionMark,
    Hat,
    Dollar,
    Star,
    Comma,
    OpenParenthesis,
    CloseParenthesis,
    Dot,
    Minus,
    Plus,
    Or,
    Concatenation,
    Class(CharacterClass),
    Quant(Quantifier),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenType {
    Literal(RegexType),
    Opening(RegexType),
    Closing(RegexType),
    UnaryOperator(RegexType),
    BinaryOperator(RegexType),
    StartOrEndCondition(RegexType),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterClass {
    // Whether this is a negated class [^...]
    negated: bool,
    // Individual characters in the class
    singles: Vec<char>,
    // Character ranges in the class
    ranges: Vec<(char, char)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Quantifier {
    // {n}
    Exact(usize),
    // {n,}
    AtLeast(usize),
    // {n,m}
    Range(usize, usize),
}

// 2. CONVERSION IMPLEMENTATIONS
// ===========================

impl ops::Deref for TokenType {
    type Target = RegexType;

    fn deref(&self) -> &Self::Target {
        self.into_inner()
    }
}

impl TokenType {
    /// Converts a TokenType back to its inner RegexType
    pub fn into_inner(&self) -> &RegexType {
        match self {
            TokenType::Literal(rt) => rt,
            TokenType::Opening(rt) => rt,
            TokenType::Closing(rt) => rt,
            TokenType::UnaryOperator(rt) => rt,
            TokenType::BinaryOperator(rt) => rt,
            TokenType::StartOrEndCondition(rt) => rt,
        }
    }
}



// 3. DISPLAY IMPLEMENTATIONS
// =========================

impl fmt::Display for RegexType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegexType::Char(c) => write!(f, "{}", c),
            RegexType::QuestionMark => write!(f, "?"),
            RegexType::Hat => write!(f, "^"),
            RegexType::Dollar => write!(f, "$"),
            RegexType::Star => write!(f, "*"),
            RegexType::Comma => write!(f, ","),
            RegexType::OpenParenthesis => write!(f, "("),
            RegexType::CloseParenthesis => write!(f, ")"),
            RegexType::Dot => write!(f, "."),
            RegexType::Minus => write!(f, "-"),
            RegexType::Plus => write!(f, "+"),
            RegexType::Or => write!(f, "|"),
            RegexType::Concatenation => write!(f, "&"),
            RegexType::Class(c) => write!(f, "{}", c),
            RegexType::Quant(q) => write!(f, "{}", q),
        }
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(rt) => write!(f, "{}", rt),
            Self::Opening(rt) => write!(f, "{}", rt),
            Self::Closing(rt) => write!(f, "{}", rt),
            Self::UnaryOperator(rt) => write!(f, "{}", rt),
            Self::BinaryOperator(rt) => write!(f, "{}", rt),
            Self::StartOrEndCondition(rt) => write!(f, "{}", rt),
        }
    }
}

impl fmt::Display for CharacterClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        
        if self.negated {
            write!(f, "^")?;
        }
        
        // Print individual characters
        for &c in &self.singles {
            write!(f, "{}", c)?;
        }
        
        // Print ranges
        for &(start, end) in &self.ranges {
            write!(f, "{}-{}", start, end)?;
        }
        
        write!(f, "]")
    }
}

impl fmt::Display for Quantifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Quantifier::Exact(n) => write!(f, "{{{}}}", n),
            Quantifier::AtLeast(n) => write!(f, "{{{},}}", n),
            Quantifier::Range(min, max) => write!(f, "{{{},{}}}", min, max),
        }
    }
}

// 4. CONVERSION IMPLEMENTATIONS
// ============================

impl From<RegexType> for TokenType {
    fn from(value: RegexType) -> Self {
        match value {
            // Opening group
            RegexType::OpenParenthesis => TokenType::Opening(value),

            // Closing group
            RegexType::CloseParenthesis => TokenType::Closing(value),

            // One element operator
            RegexType::Star
                | RegexType::Plus
                | RegexType::QuestionMark => TokenType::UnaryOperator(value),
            
            // Two element operator
            RegexType::Or
                | RegexType::Concatenation => TokenType::BinaryOperator(value),
            
            // start or end of line conditions
            RegexType::Hat
                | RegexType::Dollar => TokenType::StartOrEndCondition(value),


            _ => TokenType::Literal(value)
        }
    }
}

// 5. TYPE-SPECIFIC METHODS
// =======================

impl RegexType {
    pub fn precedence(&self) -> usize {
        match self {
            RegexType::Star | RegexType::Plus | RegexType::QuestionMark => 3,

            RegexType::Concatenation => 2,

            RegexType::Or => 1,

            _ => 0,
        }
    }

    pub fn type_(&self) -> TokenType {
        match self {
            // Opening group
            RegexType::OpenParenthesis => TokenType::Opening(self.clone()),

            // Closing group
            RegexType::CloseParenthesis => TokenType::Closing(self.clone()),

            // One element operator
            RegexType::Star
                | RegexType::Plus
                | RegexType::QuestionMark => TokenType::UnaryOperator(self.clone()),
            
            // Two element operator
            RegexType::Or
                | RegexType::Concatenation => TokenType::BinaryOperator(self.clone()),
            
            // start or end of line conditions
            RegexType::Hat
                | RegexType::Dollar => TokenType::StartOrEndCondition(self.clone()),

            _ => TokenType::Literal(self.clone())
        }
    }

    pub fn is_special_character(&self) -> bool {
        todo!()
    }
}

impl TokenType {
    pub fn need_concatenation_with(&self, other: &RegexType) -> bool {
        match (self, other.type_()) {
            // Literal followed by literal or opening parenthesis
            (TokenType::Literal(_),
                TokenType::Literal(_) | TokenType::Opening(_) | TokenType::StartOrEndCondition(_)) => true,

            // Closing parenthesis followed by literal/opening parenthesis
            (TokenType::Closing(_),
                TokenType::Literal(_) | TokenType::Opening(_) | TokenType::StartOrEndCondition(_)) => true,

            // Unary operator followed by literal/opening parenthesis
            (TokenType::UnaryOperator(_),
                TokenType::Literal(_) | TokenType::Opening(_) | TokenType::StartOrEndCondition(_)) => true,

            // Start/end condition followed by literal/opening parenthesis
            (TokenType::StartOrEndCondition(_),
                TokenType::Literal(_) | TokenType::Opening(_)) => true,

            _ => false
        }
    }

    pub fn precedence(&self) -> usize {
        match self {
            Self::Literal(rt) => rt.precedence(),
            Self::Opening(rt) => rt.precedence(),
            Self::Closing(rt) => rt.precedence(),
            Self::UnaryOperator(rt) => rt.precedence(),
            Self::BinaryOperator(rt) => rt.precedence(),
            Self::StartOrEndCondition(rt) => rt.precedence()
        }
    }
}

// 6. REGEX PARSING IMPLEMENTATION
// ==============================

impl Regex {
    pub fn new(expr: String) -> ParsingResult<VecDeque<TokenType>> {
        let tokens = Self::tokens(&expr)?;

        let tokens_with_concatenation = Self::add_concatenation(tokens);

        for token in &tokens_with_concatenation {
            eprint!("{} ", token.to_string())
        }
        eprintln!();

        Ok(tokens_with_concatenation)
    }

    pub fn tokens_to_posix(_tokens: VecDeque<RegexType>) {
        todo!()
    }

    pub fn add_concatenation(tokens: VecDeque<RegexType>) -> VecDeque<TokenType> {
        if tokens.len() < 2 {
            return tokens.into_iter()
                .map(TokenType::from)
                .collect();
        }

        let mut result: VecDeque<TokenType> = VecDeque::with_capacity(tokens.len() * 2);
        let mut tokens_iter = tokens.into_iter().peekable();
        
        // Process first token
        if let Some(token) = tokens_iter.next() {
            result.push_back(TokenType::from(token));
            
            // Process remaining tokens
            while let Some(next_token) = tokens_iter.next() {
                let last = result.back().unwrap();
                let current = &next_token;
                
                // Check if concatenation is needed
                if last.need_concatenation_with(current) {
                    result.push_back(TokenType::from(RegexType::Concatenation));
                }
                
                result.push_back(TokenType::from(next_token));
            }
        }
        
        result
    }

    pub fn tokens(input: &str) -> ParsingResult<VecDeque<RegexType>> {
        let mut tokens = VecDeque::with_capacity(input.len());
        let mut chars = input.chars();

        while let Some(c) = chars.next() {
            match c {
                '"' => Self::add_string(&mut tokens, &mut chars)?,
                '[' => Self::add_character_class(&mut tokens, &mut chars)?,
                '{' => Self::add_quantifier(&mut tokens, &mut chars)?,
                c => tokens.push_back(Self::into_type(c, &mut chars)),
            }
        }

        Ok(tokens)
    }

    /// Handling litterals (trick: transform litterals into parenthesis of chars)
    pub fn add_string(tokens: &mut VecDeque<RegexType>, chars: &mut Chars<'_>) -> ParsingResult<()> {
        // Open parenthesis replace open '"'
        tokens.push_back(RegexType::OpenParenthesis);

        while let Some(c) = chars.next() {
            match c {                            
                '\\' => {
                    let c = chars.next()
                        .unwrap_or('\\');

                    tokens.push_back(RegexType::Char(Utils::backslashed(c)));
                },

                '\"' => {
                    // Close parenthesis replace close '"'
                    tokens.push_back(RegexType::CloseParenthesis);
                    return Ok(());
                },

                c => tokens.push_back(RegexType::Char(c)),
            }
        }

        return Err(ParsingError::unrecognized_rule().because("Unclosed string"));
    }

    /// Handle character classes ([...])
    pub fn add_character_class(tokens: &mut VecDeque<RegexType>, chars: &mut Chars<'_>) -> ParsingResult<()> {
        let class = CharacterClass::parse(chars)?;
        tokens.push_back(RegexType::Class(class));
        Ok(())
    }

    /// Handle quantifiers ({n}, {n,}, {n,m})
    pub fn add_quantifier(tokens: &mut VecDeque<RegexType>, chars: &mut Chars<'_>) -> ParsingResult<()> {
        let mut digits1 = String::new();
        let mut digits2 = String::new();
        let mut saw_comma = false;
        
        while let Some(c) = chars.next() {
            match c {
                '0'..='9' => {
                    if saw_comma {
                        digits2.push(c);
                    } else {
                        digits1.push(c);
                    }
                },
                ',' => {
                    saw_comma = true;
                },
                '}' => {
                    let quant = if !saw_comma {
                        // {n}
                        let n = digits1.parse::<usize>()
                            .map_err(|_| ParsingError::unrecognized_rule().because("Invalid quantifier number"))?;
                        Quantifier::Exact(n)
                    } else if digits2.is_empty() {
                        // {n,}
                        let n = digits1.parse::<usize>()
                            .map_err(|_| ParsingError::unrecognized_rule().because("Invalid quantifier number"))?;
                        Quantifier::AtLeast(n)
                    } else {
                        // {n,m}
                        let n = digits1.parse::<usize>()
                            .map_err(|_| ParsingError::unrecognized_rule().because("Invalid quantifier number"))?;
                        let m = digits2.parse::<usize>()
                            .map_err(|_| ParsingError::unrecognized_rule().because("Invalid quantifier number"))?;
                        
                        if n > m {
                            return Err(ParsingError::unrecognized_rule().because("Invalid quantifier range: min > max"));
                        }
                        
                        Quantifier::Range(n, m)
                    };
                    
                    tokens.push_back(RegexType::Quant(quant));
                    return Ok(());
                },
                _ => {
                    return Err(ParsingError::unrecognized_rule().because("Invalid character in quantifier"));
                }
            }
        }
        
        Err(ParsingError::unrecognized_rule().because("Unclosed quantifier"))
    }

    pub fn into_type(c: char, chars: &mut Chars<'_>) -> RegexType {
        match c {
            '*' => RegexType::Star,

            '.' => RegexType::Dot,

            '+' => RegexType::Plus,

            '-' => RegexType::Minus,

            ',' => RegexType::Comma,

            '(' => RegexType::OpenParenthesis,

            ')' => RegexType::CloseParenthesis,

            '^' => RegexType::Hat,

            '$' => RegexType::Dollar,

            '?' => RegexType::QuestionMark,

            '|' => RegexType::Or,

            '\\' => {
                let next_c = chars.next().unwrap_or('\\');
                
                // Check if it's a shorthand character class
                match next_c {
                    'd' | 'D' | 'w' | 'W' | 's' | 'S' => {
                        if let Ok(class) = CharacterClass::from_shorthand(next_c) {
                            RegexType::Class(class)
                        } else {
                            RegexType::Char(Utils::backslashed(next_c))
                        }
                    },
                    // Handle other escape sequences
                    _ => RegexType::Char(Utils::backslashed(next_c))
                }
            },

            c => RegexType::Char(c),
        }
    }
}

impl CharacterClass {
    pub fn new() -> Self {
        Self {
            negated: false,
            singles: Vec::new(),
            ranges: Vec::new(),
        }
    }

    pub fn negated(mut self) -> Self {
        self.negated = true;
        self
    }

    pub fn add_char(&mut self, c: char) {
        // Only add if not already in a range or singles
        if !self.contains_char(c) {
            self.singles.push(c);
        }
    }

    // Private method to remove a character from singles
    fn remove_char(&mut self, c: char) {
        if let Some(index) = self.singles.iter().position(|&x| x == c) {
            self.singles.swap_remove(index);
        }
    }

    pub fn add_range(&mut self, start: char, end: char) {
        // Validate the range
        if start <= end {
            // Check for overlaps with existing ranges
            if !self.ranges.iter().any(|(s, e)| *s <= start && end <= *e) {
                self.ranges.push((start, end));
            }
        }
    }

    // Check if a character is contained in this class
    pub fn contains_char(&self, c: char) -> bool {
        self.singles.contains(&c) || 
        self.ranges.iter().any(|(start, end)| *start <= c && c <= *end)
    }

    // Check if a character matches this class (considering negation)
    pub fn matches(&self, c: char) -> bool {
        let contains = self.contains_char(c);
        if self.negated {
            !contains
        } else {
            contains
        }
    }

    // Constructor for a single character class
    pub fn single(c: char) -> Self {
        let mut class = Self::new();
        class.add_char(c);
        class
    }

    // Constructor for a range class
    pub fn range(start: char, end: char) -> Self {
        let mut class = Self::new();
        class.add_range(start, end);
        class
    }

    // Merge two character classes
    pub fn merge(&mut self, other: &CharacterClass) {
        // Only merge non-negated classes
        if !other.negated {
            // Add all singles from other
            for &c in &other.singles {
                self.add_char(c);
            }
            
            // Add all ranges from other
            for &(start, end) in &other.ranges {
                self.add_range(start, end);
            }
        }
    }

    // Parse a character class from a string
    pub fn parse(chars: &mut std::str::Chars) -> ParsingResult<Self> {
        let mut class = Self::new();
        let mut negated = false;
        let mut prev_char: Option<char> = None;
        
        // Check for negation
        if let Some('^') = chars.clone().next() {
            negated = true;
            chars.next(); // Consume the '^'
        }
        
        while let Some(c) = chars.next() {
            match c {
                ']' => {
                    if negated {
                        class = class.negated();
                    }
                    return Ok(class);
                }
                '-' => {
                    // Range definition
                    if let Some(start) = prev_char {
                        if let Some(end) = chars.next() {
                            if end == ']' {
                                // '-' at the end is a literal character
                                class.add_char('-');
                                return Ok(class);
                            } else {
                                class.add_range(start, end);
                                class.remove_char(start); // Remove the start char as it's now part of a range
                                prev_char = None;
                            }
                        } else {
                            return Err(ParsingError::unrecognized_rule().because("Unclosed character class"));
                        }
                    } else {
                        // '-' at the beginning is a literal character
                        class.add_char('-');
                    }
                }
                '\\' => {
                    // Handle escape sequences
                    if let Some(next_c) = chars.next() {
                        let interpreted = Utils::backslashed(next_c);
                        class.add_char(interpreted);
                        prev_char = Some(interpreted);
                    } else {
                        return Err(ParsingError::unrecognized_rule().because("Escape sequence at end of character class"));
                    }
                }
                c => {
                    class.add_char(c);
                    prev_char = Some(c);
                }
            }
        }
        
        Err(ParsingError::unrecognized_rule().because("Unclosed character class"))
    }

    // Compatibility methods to create instances that match the old enum API
    pub fn from_single(c: char) -> Self {
        let mut class = Self::new();
        class.add_char(c);
        class
    }
    
    pub fn from_range(start: char, end: char) -> Self {
        let mut class = Self::new();
        class.add_range(start, end);
        class
    }
    
    pub fn from_negated(other: CharacterClass) -> Self {
        let mut class = other;
        class.negated = true;
        class
    }

    // Create a character class from a shorthand character class like \d, \w, etc.
    pub fn from_shorthand(c: char) -> ParsingResult<Self> {
        match c {
            'd' => Ok(Self::digit()),
            'D' => Ok(Self::non_digit()),
            'w' => Ok(Self::word_char()),
            'W' => Ok(Self::non_word_char()),
            's' => Ok(Self::whitespace()),
            'S' => Ok(Self::non_whitespace()),
            _ => Err(ParsingError::unrecognized_rule().because(&format!("Unknown shorthand class \\{}", c))),
        }
    }

    // Predefined character classes
    pub fn digit() -> Self {
        let mut class = Self::new();
        class.add_range('0', '9');
        class
    }

    pub fn non_digit() -> Self {
        Self::digit().negated()
    }

    pub fn word_char() -> Self {
        let mut class = Self::new();
        class.add_range('a', 'z');
        class.add_range('A', 'Z');
        class.add_range('0', '9');
        class.add_char('_');
        class
    }

    pub fn non_word_char() -> Self {
        Self::word_char().negated()
    }

    pub fn whitespace() -> Self {
        let mut class = Self::new();
        // Add all whitespace characters
        for c in [' ', '\t', '\r', '\n', '\u{000C}', '\u{000B}'] {
            class.add_char(c);
        }
        class
    }

    pub fn non_whitespace() -> Self {
        Self::whitespace().negated()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
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
        assert!(matches!(Regex::into_type('*', &mut chars), RegexType::Star));
        assert!(matches!(Regex::into_type('+', &mut chars), RegexType::Plus));
        assert!(matches!(Regex::into_type('?', &mut chars), RegexType::QuestionMark));
        assert!(matches!(Regex::into_type('|', &mut chars), RegexType::Or));
        assert!(matches!(Regex::into_type('.', &mut chars), RegexType::Dot));
    }
    
    #[test]
    fn test_into_type_escape_sequences() {
        let mut chars = "d".chars();
        if let RegexType::Class(class) = Regex::into_type('\\', &mut chars) {
            assert!(class.matches('0'));
            assert!(!class.matches('a'));
        } else {
            panic!("Expected a CharacterClass for digit");
        }
        
        let mut chars = "w".chars();
        if let RegexType::Class(class) = Regex::into_type('\\', &mut chars) {
            assert!(class.matches('a'));
            assert!(class.matches('_'));
            assert!(!class.matches(' '));
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
        assert!(matches!(tokens[1], RegexType::Plus));
        assert!(matches!(tokens[2], RegexType::Char('b')));
        assert!(matches!(tokens[3], RegexType::Star));
        
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
            assert!(class.contains_char('b'));
            assert!(class.contains_char('c'));
            assert!(!class.contains_char('a'));
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
            assert!(class.negated);
            assert!(!class.matches('a'));
            assert!(class.matches('x'));
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
            assert!(class.matches('0'));
            assert!(!class.matches('a'));
        } else {
            panic!("Expected a digit class");
        }
        
        // Check word class
        if let RegexType::Class(class) = &tokens[1] {
            assert!(class.matches('a'));
            assert!(class.matches('_'));
        } else {
            panic!("Expected a word class");
        }
        
        // Check whitespace class
        if let RegexType::Class(class) = &tokens[2] {
            assert!(class.matches(' '));
            assert!(class.matches('\t'));
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
        assert!(matches!(tokens[5], RegexType::Plus));
        
        // Check character class
        if let RegexType::Class(class) = &tokens[6] {
            assert!(class.matches('0'));
            assert!(class.matches('9'));
            assert!(!class.matches('a'));
        } else {
            panic!("Expected a digit character class");
        }
        
        assert!(matches!(tokens[7], RegexType::QuestionMark));
        
        // Check word char class
        if let RegexType::Class(class) = &tokens[8] {
            assert!(class.matches('a'));
            assert!(class.matches('_'));
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
        assert!(matches!(result[0], TokenType::Opening(RegexType::OpenParenthesis)));
        assert!(matches!(result[1], TokenType::Literal(RegexType::Char('a'))));
        assert!(matches!(result[2], TokenType::Closing(RegexType::CloseParenthesis)));
        assert!(matches!(result[3], TokenType::BinaryOperator(RegexType::Concatenation)));
        assert!(matches!(result[4], TokenType::Literal(RegexType::Char('b'))));
    }
    
    // 4. END-TO-END TESTS
    // ==================
    
    #[test]
    fn test_regex_new_simple() -> ParsingResult<()> {
        let expr = "ab+c*".to_string();
        let result = Regex::new(expr)?;
        
        // Should be: a·b+·c*
        assert_eq!(result.len(), 7);
        Ok(())
    }
    
    #[test]
    fn test_regex_new_with_alternation() -> ParsingResult<()> {
        let expr = "a|b|c".to_string();
        let result = Regex::new(expr)?;
        
        // Should be: a|b|c
        assert_eq!(result.len(), 5);
        Ok(())
    }
    
    #[test]
    fn test_regex_new_with_groups() -> ParsingResult<()> {
        let expr = "(a)(b)".to_string();
        let result = Regex::new(expr)?;
        
        // Should be: (a)·(b)
        assert_eq!(result.len(), 7);
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
        assert!(digit_class.matches('0'));
        assert!(!digit_class.matches('a'));
        
        let word_class = CharacterClass::from_shorthand('w')?;
        assert!(word_class.matches('a'));
        assert!(word_class.matches('_'));
        assert!(!word_class.matches(' '));
        
        let space_class = CharacterClass::from_shorthand('s')?;
        assert!(space_class.matches(' '));
        assert!(space_class.matches('\t'));
        assert!(!space_class.matches('a'));
        
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
            TokenType::from(RegexType::Star),
            TokenType::UnaryOperator(RegexType::Star)
        ));
        
        assert!(matches!(
            TokenType::from(RegexType::Or),
            TokenType::BinaryOperator(RegexType::Or)
        ));
        
        assert!(matches!(
            TokenType::from(RegexType::OpenParenthesis),
            TokenType::Opening(RegexType::OpenParenthesis)
        ));
        
        assert!(matches!(
            TokenType::from(RegexType::CloseParenthesis),
            TokenType::Closing(RegexType::CloseParenthesis)
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
        
        assert!(class.contains_char('a'));
        assert!(class.contains_char('b'));
        assert!(class.contains_char('c'));
        assert!(!class.contains_char('d'));
        
        assert!(class.matches('a'));
        assert!(!class.matches('d'));
    }
    
    #[test]
    fn test_character_class_range() {
        let mut class = CharacterClass::new();
        class.add_range('a', 'z');
        
        assert!(class.contains_char('a'));
        assert!(class.contains_char('m'));
        assert!(class.contains_char('z'));
        assert!(!class.contains_char('A'));
    }
    
    #[test]
    fn test_character_class_negated() {
        let class = CharacterClass::new().negated();
        
        assert!(!class.contains_char('a')); // Empty class contains nothing
        assert!(class.matches('a')); // But negated matches everything
    }
    
    #[test]
    fn test_character_class_complex() {
        let mut class = CharacterClass::new();
        class.add_char('a');
        class.add_char('b');
        class.add_range('x', 'z');
        
        assert!(class.contains_char('a'));
        assert!(class.contains_char('b'));
        assert!(!class.contains_char('c'));
        assert!(class.contains_char('x'));
        assert!(class.contains_char('y'));
        assert!(class.contains_char('z'));
        assert!(!class.contains_char('w'));
    }
    
    #[test]
    fn test_character_class_parse() -> ParsingResult<()> {
        let input = "abc]";
        let mut chars = input.chars();
        let class = CharacterClass::parse(&mut chars)?;
        
        assert!(class.contains_char('a'));
        assert!(class.contains_char('b'));
        assert!(class.contains_char('c'));
        assert!(!class.contains_char('d'));
        
        Ok(())
    }
    
    #[test]
    fn test_character_class_parse_range() -> ParsingResult<()> {
        let input = "a-z]";
        let mut chars = input.chars();
        let class = CharacterClass::parse(&mut chars)?;
        
        assert!(class.contains_char('a'));
        assert!(class.contains_char('m'));
        assert!(class.contains_char('z'));
        assert!(!class.contains_char('A'));
        
        Ok(())
    }
    
    #[test]
    fn test_character_class_parse_complex() -> ParsingResult<()> {
        let input = "a-cx-z]";
        let mut chars = input.chars();
        let class = CharacterClass::parse(&mut chars)?;
        
        assert!(class.contains_char('a'));
        assert!(class.contains_char('b'));
        assert!(class.contains_char('c'));
        assert!(!class.contains_char('d'));
        assert!(class.contains_char('x'));
        assert!(class.contains_char('y'));
        assert!(class.contains_char('z'));
        
        Ok(())
    }
    
    #[test]
    fn test_character_class_parse_negated() -> ParsingResult<()> {
        let input = "^a-c]";
        let mut chars = input.chars();
        let class = CharacterClass::parse(&mut chars)?;
        
        assert!(!class.matches('a'));
        assert!(!class.matches('b'));
        assert!(!class.matches('c'));
        assert!(class.matches('d'));
        
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
        assert!(class1.contains_char('a'));
        assert!(class1.contains_char('b'));
        assert!(class1.contains_char('0'));
        assert!(class1.contains_char('5'));
        assert!(class1.contains_char('9'));
        assert!(class1.contains_char('x'));
        assert!(class1.contains_char('y'));
        assert!(class1.contains_char('z'));
    }
    
    #[test]
    fn test_character_class_merge_with_negated() {
        let mut class1 = CharacterClass::new();
        class1.add_char('a');
        
        let class2 = CharacterClass::new().negated();
        
        // Merging with a negated class should not change class1
        class1.merge(&class2);
        
        assert!(class1.contains_char('a'));
        assert!(!class1.contains_char('b'));
    }
    
    #[test]
    fn test_character_class_parse_edge_cases() -> ParsingResult<()> {
        // Test dash at beginning
        let input = "-abc]";
        let mut chars = input.chars();
        let class = CharacterClass::parse(&mut chars)?;
        
        assert!(class.contains_char('-'));
        assert!(class.contains_char('a'));
        
        // Test dash at end
        let input = "abc-]";
        let mut chars = input.chars();
        let class = CharacterClass::parse(&mut chars)?;
        
        assert!(class.contains_char('-'));
        assert!(class.contains_char('a'));
        
        Ok(())
    }
    
    #[test]
    fn test_character_class_parse_with_escapes() -> ParsingResult<()> {
        let input = "a\\n\\t]";
        let mut chars = input.chars();
        let class = CharacterClass::parse(&mut chars)?;
        
        assert!(class.contains_char('a'));
        assert!(class.contains_char('\n'));
        assert!(class.contains_char('\t'));
        
        Ok(())
    }
    
    #[test]
    fn test_character_class_predefined_methods() {
        // Test digit class
        let digit = CharacterClass::digit();
        assert!(digit.matches('0'));
        assert!(digit.matches('9'));
        assert!(!digit.matches('a'));
        
        // Test non-digit class
        let non_digit = CharacterClass::non_digit();
        assert!(!non_digit.matches('0'));
        assert!(non_digit.matches('a'));
        
        // Test word char class
        let word = CharacterClass::word_char();
        assert!(word.matches('a'));
        assert!(word.matches('Z'));
        assert!(word.matches('0'));
        assert!(word.matches('_'));
        assert!(!word.matches(' '));
        assert!(!word.matches('-'));
        
        // Test non-word char class
        let non_word = CharacterClass::non_word_char();
        assert!(!non_word.matches('a'));
        assert!(non_word.matches(' '));
        assert!(non_word.matches('-'));
        
        // Test whitespace class
        let space = CharacterClass::whitespace();
        assert!(space.matches(' '));
        assert!(space.matches('\t'));
        assert!(space.matches('\n'));
        assert!(space.matches('\r'));
        assert!(!space.matches('a'));
        
        // Test non-whitespace class
        let non_space = CharacterClass::non_whitespace();
        assert!(!non_space.matches(' '));
        assert!(non_space.matches('a'));
    }
    
    #[test]
    fn test_character_class_convenience_constructors() {
        // Test single character constructor
        let single = CharacterClass::single('x');
        assert!(single.contains_char('x'));
        assert!(!single.contains_char('y'));
        
        // Test range constructor
        let range = CharacterClass::range('a', 'c');
        assert!(range.contains_char('a'));
        assert!(range.contains_char('b'));
        assert!(range.contains_char('c'));
        assert!(!range.contains_char('d'));
        
        // Test negated helper
        let negated = CharacterClass::from_negated(CharacterClass::single('x'));
        assert!(!negated.matches('x'));
        assert!(negated.matches('y'));
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
        assert!(!class.contains_char('a'));
        assert!(!class.contains_char('b'));
        
        Ok(())
    }
    
    #[test]
    fn test_character_class_add_invalid_range() {
        let mut class = CharacterClass::new();
        
        // Adding a range where start > end should be ignored
        class.add_range('z', 'a');
        
        assert!(!class.contains_char('a'));
        assert!(!class.contains_char('z'));
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
}