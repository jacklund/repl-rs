extern crate repl_rs;

use repl_rs::Repl;
use repl_rs::{Command, Parameter, Result, Value};
use std::collections::HashMap;
use std::fmt::Display;

/// Example using Repl with a custom prompt
struct CustomPrompt;

impl Display for CustomPrompt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "$ ")
    }
}

fn hello<T>(args: HashMap<String, Value>, _context: &mut T) -> Result<Option<String>> {
    Ok(Some(format!("Hello, {}", args["who"])))
}

fn main() -> Result<()> {
    let mut repl = Repl::new(())
        .with_name("MyApp")
        .with_prompt(&CustomPrompt)
        .with_version("v0.1.0")
        .with_description("My very cool app")
        .add_command(
            Command::new("hello", hello)
                .with_parameter(Parameter::new("who").set_required(true)?)?
                .with_help("Greetings!"),
        );
    repl.run()
}
