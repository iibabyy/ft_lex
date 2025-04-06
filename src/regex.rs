pub mod re2post;
pub use re2post::*;

pub mod post2nfa;
pub use post2nfa::*;

pub mod nfa_simulation;
pub use nfa_simulation::*;


use std::{collections::VecDeque, fmt, ops, str::Chars};

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
    LineStart,
    LineEnd,
    OpenParenthesis,
    CloseParenthesis,
    Any,
    Or,
    Concatenation,
    Class(CharacterClass),
    Quant(Quantifier),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenType {
    Literal(RegexType),
    OpenParenthesis(RegexType),
    CloseParenthesis(RegexType),
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
            TokenType::OpenParenthesis(rt) => rt,
            TokenType::CloseParenthesis(rt) => rt,
            TokenType::UnaryOperator(rt) => rt,
            TokenType::BinaryOperator(rt) => rt,
            TokenType::StartOrEndCondition(rt) => rt,
        }
    }

    /// Converts a TokenType back to its inner RegexType
    pub fn into_owned_inner(self) -> RegexType {
        match self {
            TokenType::Literal(rt) => rt,
            TokenType::OpenParenthesis(rt) => rt,
            TokenType::CloseParenthesis(rt) => rt,
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
            RegexType::LineStart => write!(f, "^"),
            RegexType::LineEnd => write!(f, "$"),
            RegexType::OpenParenthesis => write!(f, "("),
            RegexType::CloseParenthesis => write!(f, ")"),
            RegexType::Any => write!(f, "."),
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
            Self::OpenParenthesis(rt) => write!(f, "{}", rt),
            Self::CloseParenthesis(rt) => write!(f, "{}", rt),
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
            RegexType::OpenParenthesis => TokenType::OpenParenthesis(value),

            // Closing group
            RegexType::CloseParenthesis => TokenType::CloseParenthesis(value),

            // One element operator
            RegexType::Quant(_) => TokenType::UnaryOperator(value),

            // Two element operator
            RegexType::Or
                | RegexType::Concatenation => TokenType::BinaryOperator(value),
            
            // start or end of line conditions
            RegexType::LineStart
                | RegexType::LineEnd => TokenType::StartOrEndCondition(value),


            _ => TokenType::Literal(value)
        }
    }
}

// 5. TYPE-SPECIFIC METHODS
// =======================

impl RegexType {
    pub fn precedence(&self) -> usize {
        match self {

            RegexType::Quant(_) => 3,

            RegexType::Concatenation => 2,

            RegexType::Or => 1,

            _ => 0,
        }
    }

    pub fn type_(&self) -> TokenType {
        match self {
            // Opening group
            RegexType::OpenParenthesis => TokenType::OpenParenthesis(self.clone()),

            // Closing group
            RegexType::CloseParenthesis => TokenType::CloseParenthesis(self.clone()),

            // One element operator
            RegexType::Quant(_) => TokenType::UnaryOperator(self.clone()),
            
            // Two element operator
            RegexType::Or
                | RegexType::Concatenation => TokenType::BinaryOperator(self.clone()),
            
            // start or end of line conditions
            RegexType::LineStart
                | RegexType::LineEnd => TokenType::StartOrEndCondition(self.clone()),

            _ => TokenType::Literal(self.clone())
        }
    }

    pub fn match_(&self, c: &char) -> bool {

		match self {
			RegexType::Char(char) => char == c,

			RegexType::Class(class) => class.matches(c),

			_ => todo!()
		}
	}
}

impl TokenType {
    pub fn need_concatenation_with(&self, other: &RegexType) -> bool {
        match (self, other.type_()) {
            // Literal followed by literal or opening parenthesis
            (TokenType::Literal(_),
                TokenType::Literal(_) | TokenType::OpenParenthesis(_)) => true,

            // Closing parenthesis followed by literal/opening parenthesis
            (TokenType::CloseParenthesis(_),
                TokenType::Literal(_) | TokenType::OpenParenthesis(_)) => true,

            // Unary operator followed by literal/opening parenthesis
            (TokenType::UnaryOperator(_),
                TokenType::Literal(_) | TokenType::OpenParenthesis(_)) => true,

            _ => false
        }
    }

    pub fn precedence(&self) -> usize {
        match self {
            Self::Literal(rt) => rt.precedence(),
            Self::OpenParenthesis(rt) => rt.precedence(),
            Self::CloseParenthesis(rt) => rt.precedence(),
            Self::UnaryOperator(rt) => rt.precedence(),
            Self::BinaryOperator(rt) => rt.precedence(),
            Self::StartOrEndCondition(rt) => rt.precedence()
        }
    }
}

// 6. REGEX PARSING IMPLEMENTATION
// ==============================

impl Regex {
    pub fn new(expr: String) -> ParsingResult<Vec<TokenType>> {
        let tokens = Self::tokens(&expr)?;

        let tokens_with_concatenation = Self::add_concatenation(tokens);

        let postfix = re2post(tokens_with_concatenation)?;

        postfix.iter().for_each(|token| eprint!("{} ", token.to_string()));

        eprintln!();

        Ok(postfix)
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
            '*' => RegexType::Quant(Quantifier::AtLeast(0)),

            '.' => RegexType::Any,

            '+' => RegexType::Quant(Quantifier::AtLeast(1)),

            '(' => RegexType::OpenParenthesis,

            ')' => RegexType::CloseParenthesis,

            '^' => RegexType::LineStart,

            '$' => RegexType::LineEnd,

            '?' => RegexType::Quant(Quantifier::Range(0, 1)),

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
        if !self.contains_char(&c) {
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
    pub fn contains_char(&self, c: &char) -> bool {
        self.singles.contains(c) || 
        self.ranges.iter().any(|(start, end)| start <= c && c <= end)
    }

    // Check if a character matches this class (considering negation)
    pub fn matches(&self, c: &char) -> bool {
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

    // Compatibility methods to create instances tLineStart match the old enum API
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