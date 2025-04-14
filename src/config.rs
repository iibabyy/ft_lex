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
pub struct Config {
    /// input files
    /// None if stdin
    pub args: Vec<Option<String>>,

    /// -t / --stdout
    pub stdout: bool,

    /// -v
    /// Write a summary of lex statistics to the standard output. If the -t option is specified and -n is not specified, this report shall be written to standard error. If table sizes are specified in the lex source code, and if the -n option is not specified, the -v option may be enabled.
    pub summary: bool,

    /// -n
    /// Suppress the summary of statistics usually written with the -v option. If no table sizes are specified in the lex source code and the -v option is not specified, then -n is implied
    pub no_stats_summary: bool,
}

impl Config {
    pub(super) fn init() -> Result<Self, String> {
        let mut args = env::args();

        let _executable = args.next();

        let mut config = Self::default();

        for arg in args {
            match arg.as_str() {
                "-t" | "--stdout" => config.stdout = true,

                "-v" | "--verbose" => config.summary = true,

                "-n" => config.no_stats_summary = true,

                arg if arg.starts_with("-") => return Err(format!("Invalid option: {arg}")),

                _ => config.args.push(Some(arg))
            }
        }

        if config.args.is_empty() {
            // stdin input if no file
            config.args.push(None);
        }

        Ok(config)
    }
}
