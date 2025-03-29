use super::*;

use std::{env, error::Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetLanguage {
	C,
}

impl Default for TargetLanguage {
	fn default() -> Self {
		TargetLanguage::C
	}
}

#[derive(Debug, Default)]
pub(super) struct Config {
	/// input files
	/// None if stdin
	args: Vec<Option<String>>,

	/* OPTIONS */

	/// -o FILE / --output-file=FILE
	output_file: Option<String>,

	/// -t / --stdout
	stdout: bool,

	/// -e LANG / --emit=LANG
	target_language: TargetLanguage,

	/// replace yy (e.g yylex -> {STRING}lex)
	/// -P STRING / --prefix=STRING
	prefix: String,

	/// do not include <unistd.h>
	/// --nounistd
	no_unistd: bool,

	// do not generate those functions
	// --noFUNCTION
	no_functions: Vec<String>,

	// -i --case-insensitive
	case_insensitive: bool,

	// track line count in yylineno
	// --yylineno
	yylineno: bool
}

impl Config {

	pub(super) fn init() -> Result<Self, ()> {
		let mut args = env::args();

		let _executable = args.next();

		let mut config = Self::default();

		for arg in args {

			if arg == "-" {
				// stdin input
				config.args.push(None);
			}

			if arg.starts_with("--") {
				// TODO: add option
				eprintln!("Long argument detected ({}) -> skip", arg);
				continue;
			}

			if arg.starts_with("-") {
				// TODO: add option
				eprintln!("Short argument detected ({}) -> skip", arg);
				continue;
			}

			config.args.push(Some(arg));
			
		}
		
		if config.args.is_empty() {
			// stdin input if no file
			config.args.push(None);
		}

		Ok(config)
	}

}
