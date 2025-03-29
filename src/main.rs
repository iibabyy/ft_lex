#![allow(dead_code, unused_imports, unreachable_code)]

mod parsing;
use parsing::*;

mod arg;
use arg::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let config = Config::init();

    dbg!(&config);

    let mut parser = Parsing::new()?;

    parser.parse(&config)?;

    Ok(())
}
