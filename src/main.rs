#![allow(dead_code, unused_imports, unreachable_code)]

use std::fmt::{Debug, Formatter};

mod tests;

mod parsing;
use parsing::*;

mod config;
use config::*;

mod regex;
use regex::*;

// TODO: error if '\' or '/' in Description section

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let config = Config::init()?;

    // dbg!(&config);

    let mut parser = Parsing::new()?;

    if let Err(errors) = parser.parse(&config) {
		// print errors
        for err in errors {
			match config.stdout {
				// stderr if -t/--stdout is set
				true => eprintln!("{}", err),

				// stdout if -t/--stdout is not set
				false => println!("{}", err),
			}
        }
    }

    dbg!(parser.definitions);

    Ok(())
}
