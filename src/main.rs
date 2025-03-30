#![allow(dead_code, unused_imports, unreachable_code)]

mod parsing;
use std::fmt::{Debug, Formatter};

use parsing::*;

mod arg;
use arg::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::init();

    // dbg!(&config);

    let mut parser = Parsing::new()?;

    if let Err(err) = parser.parse(&config) {
        eprintln!("{}", err.to_string());
        Err(err)?;
    }

    dbg!(parser.definitions);

    Ok(())
}
