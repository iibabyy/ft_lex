#![allow(dead_code, unused_imports, unreachable_code)]

use std::fmt::{Debug, Formatter};

mod parsing;
use parsing::*;

mod arg;
use arg::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::init();

    // dbg!(&config);

    let mut parser = Parsing::new()?;

    parser.parse(&config)?;

    dbg!(parser.definitions);

    Ok(())
}
