use std::{collections::VecDeque, fmt, str::Chars};
use crate::parsing::{error::{ParsingError, ParsingResult}, utils::Utils};

use super::*;

// 1. BASIC TYPE DEFINITIONS
// =========================

pub struct Regex {
    operator_stack: Vec<RegexType>,
    output_stack: Vec<RegexType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegexType {
    Char(char),
    OpenBracket,
    CloseBracket,
    QuestionMark,
    Hat,
    Dollar,
    Star,
    OpenBrace,
    CloseBrace,
    Comma,
    OpenParenthesis,
    CloseParenthesis,
    Dot,
    Minus,
    Plus,
    Or,
    Concatenation,
    ShorthandClass(ShorthandClass),
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShorthandClass {
    /// [0-9]
    Digit,
    /// [^0-9]
    NonDigit,

    /// [a-zA-Z0-9_]
    WordChar,
    /// [^a-zA-Z0-9_]
    NonWordChar,

    /// [ \t\r\n\f\v]
    WhiteSpace,
    /// [^ \t\r\n\f\v]
    NonWhiteSpace,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
    Literal(RegexType),
    Opening(RegexType),
    Closing(RegexType),
    UnaryOperator(RegexType),
    BinaryOperator(RegexType),
    StartOrEndCondition(RegexType),
}

// 2. DISPLAY IMPLEMENTATIONS
// =========================

impl fmt::Display for RegexType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegexType::Char(c) => write!(f, "{}", c),
            RegexType::OpenBracket => write!(f, "["),
            RegexType::CloseBracket => write!(f, "]"),
            RegexType::QuestionMark => write!(f, "?"),
            RegexType::Hat => write!(f, "^"),
            RegexType::Dollar => write!(f, "$"),
            RegexType::Star => write!(f, "*"),
            RegexType::OpenBrace => write!(f, "{{"),
            RegexType::CloseBrace => write!(f, "}}"),
            RegexType::Comma => write!(f, ","),
            RegexType::OpenParenthesis => write!(f, "("),
            RegexType::CloseParenthesis => write!(f, ")"),
            RegexType::Dot => write!(f, "."),
            RegexType::Minus => write!(f, "-"),
            RegexType::Plus => write!(f, "+"),
            RegexType::Or => write!(f, "|"),
            RegexType::Concatenation => write!(f, "·"),
            RegexType::ShorthandClass(sc) => write!(f, "{}", sc),
            RegexType::None => write!(f, "`None`"),
        }
    }
}

impl fmt::Display for ShorthandClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShorthandClass::Digit => write!(f, r"\d"),
            ShorthandClass::NonDigit => write!(f, r"\D"),
            ShorthandClass::WordChar => write!(f, r"\w"),
            ShorthandClass::NonWordChar => write!(f, r"\W"),
            ShorthandClass::WhiteSpace => write!(f, r"\s"),
            ShorthandClass::NonWhiteSpace => write!(f, r"\S"),
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

// 3. CONVERSION IMPLEMENTATIONS
// ============================

impl TryFrom<char> for ShorthandClass {
    type Error = ();

    /// valid values:
    /// d/D w/W s/S
    fn try_from(value: char) -> Result<Self, Self::Error> {
        let res = match value {
            'd' => ShorthandClass::Digit,
            'D' => ShorthandClass::NonDigit,
            
            'w' => ShorthandClass::WordChar,
            'W' => ShorthandClass::NonWordChar,
            
            's' => ShorthandClass::WhiteSpace,
            'S' => ShorthandClass::NonWhiteSpace,

            _ => return Err(())
        };

        Ok(res)
    }
}

impl From<RegexType> for TokenType {
    fn from(value: RegexType) -> Self {
        match value {
            // Opening group
            RegexType::OpenParenthesis
                | RegexType::OpenBrace
                | RegexType::OpenBracket => TokenType::Opening(value),

            // Closing group
            RegexType::CloseParenthesis
                | RegexType::CloseBrace
                | RegexType::CloseBracket => TokenType::Closing(value),

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

// 4. TYPE-SPECIFIC METHODS
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
            RegexType::OpenParenthesis
                | RegexType::OpenBrace
                | RegexType::OpenBracket => TokenType::Opening(self.clone()),

            // Closing group
            RegexType::CloseParenthesis
                | RegexType::CloseBrace
                | RegexType::CloseBracket => TokenType::Closing(self.clone()),

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

// 5. REGEX PARSING IMPLEMENTATION
// ==============================

impl Regex {
    pub fn new(expr: String) -> ParsingResult<VecDeque<TokenType>> {
        let tokens = Self::tokens(&expr)?;

        let tokens_with_concatenation = Self::add_concatenation(tokens);

        for token in &tokens_with_concatenation {
            eprint!("{}", token.to_string())
        }

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

    pub fn into_type(c: char, chars: &mut Chars<'_>) -> RegexType {
        match c {
            '[' => RegexType::OpenBracket,
            ']' => RegexType::CloseBracket,

            '{' => RegexType::OpenBrace,
            '}' => RegexType::CloseBrace,

            '(' => RegexType::OpenParenthesis,
            ')' => RegexType::CloseParenthesis,

            '.' => RegexType::Dot,

            '-' => RegexType::Minus,
            '+' => RegexType::Plus,

            ',' => RegexType::Comma,

            '*' => RegexType::Star,

            '^' => RegexType::Hat,

            '$' => RegexType::Dollar,

            '?' => RegexType::QuestionMark,

            '|' => RegexType::Or,

            '\\' => {
                let c = chars.next()
                    .unwrap_or('\\');

                let interpreted = Utils::backslashed(c);

                let is_class = ShorthandClass::try_from(c);

                if let Ok(class) = is_class {
                    RegexType::ShorthandClass(class)
                } else {
                    RegexType::Char(interpreted)
                }
            },

            c => RegexType::Char(c),
        }
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
        assert!(matches!(Regex::into_type('[', &mut chars), RegexType::OpenBracket));
        assert!(matches!(Regex::into_type(']', &mut chars), RegexType::CloseBracket));
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
        assert!(matches!(
            Regex::into_type('\\', &mut chars),
            RegexType::ShorthandClass(ShorthandClass::Digit)
        ));
        
        let mut chars = "w".chars();
        assert!(matches!(
            Regex::into_type('\\', &mut chars),
            RegexType::ShorthandClass(ShorthandClass::WordChar)
        ));
        
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
    fn test_shorthand_class_try_from() {
        assert_eq!(ShorthandClass::try_from('d'), Ok(ShorthandClass::Digit));
        assert_eq!(ShorthandClass::try_from('D'), Ok(ShorthandClass::NonDigit));
        assert_eq!(ShorthandClass::try_from('w'), Ok(ShorthandClass::WordChar));
        assert_eq!(ShorthandClass::try_from('W'), Ok(ShorthandClass::NonWordChar));
        assert_eq!(ShorthandClass::try_from('s'), Ok(ShorthandClass::WhiteSpace));
        assert_eq!(ShorthandClass::try_from('S'), Ok(ShorthandClass::NonWhiteSpace));
        assert!(ShorthandClass::try_from('x').is_err());
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
}