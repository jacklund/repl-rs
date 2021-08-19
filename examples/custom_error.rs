extern crate repl_rs;

use std::fmt;

use repl_rs::{Command, Repl, Value};
use std::collections::HashMap;

/// Example using Repl with a custom error type.
#[derive(Debug)]
enum CustomError {
    ReplError(repl_rs::Error),
    StringError(String),
}

impl From<repl_rs::Error> for CustomError {
    fn from(e: repl_rs::Error) -> Self {
        CustomError::ReplError(e)
    }
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CustomError::ReplError(e) => write!(f, "REPL Error: {}", e),
            CustomError::StringError(s) => write!(f, "String Error: {}", s),
        }
    }
}

impl std::error::Error for CustomError {}

// Do nothing, unsuccesfully
fn hello<T>(
    _args: HashMap<String, Value>,
    _context: &mut T,
) -> Result<Option<String>, CustomError> {
    Err(CustomError::StringError("Returning an error".to_string()))
}

fn main() -> Result<(), repl_rs::Error> {
    let mut repl = Repl::new(())
        .with_name("MyApp")
        .with_version("v0.1.0")
        .with_description("My very cool app")
        .add_command(Command::new("hello", hello).with_help("Do nothing, unsuccessfully"));
    repl.run()
}
