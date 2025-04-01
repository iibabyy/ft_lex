#![allow(dead_code, unused_imports, unreachable_code)]

use std::fmt::{Debug, Formatter};

mod tests;

mod parsing;
pub use parsing::*;

mod arg;
pub use arg::*;

mod regex;
pub use regex::*;

// TODO: error if '\' or '/' in Description section

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // Simple patterns
    let _regex1 = Regex::new("a*".to_string())?;
    let _regex2 = Regex::new("b+".to_string())?;
    let _regex3 = Regex::new("[0-9]+".to_string())?;
    
    // More complex patterns
    let _regex4 = Regex::new("(abc|def)".to_string())?;
    let _regex5 = Regex::new("a{2,5}".to_string())?;
    let _regex6 = Regex::new("\\w+@\\w+\\.\\w+".to_string())?;
    
    // Special character classes
    let _regex7 = Regex::new("[^a-z]".to_string())?;
    let _regex8 = Regex::new("\\d{3}-\\d{3}-\\d{4}".to_string())?;

    // let config = Config::init();

    // // dbg!(&config);

    // let mut parser = Parsing::new()?;

    // parser.parse(&config)?;

    // dbg!(parser.definitions);

    Ok(())
}
