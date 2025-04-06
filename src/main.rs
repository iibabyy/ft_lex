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

    // let config = Config::init();

    // // dbg!(&config);

    // let mut parser = Parsing::new()?;

    // parser.parse(&config)?;

    // dbg!(parser.definitions);

    Ok(())
}
