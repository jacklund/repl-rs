extern crate repl_rs;

use repl_rs::{Command, Parameter, Result, Value};
use repl_rs::{Convert, Repl};
use std::collections::HashMap;

#[derive(Default)]
struct Context {
    _foobar: usize,
}

fn add(args: HashMap<String, Value>, _context: &mut Context) -> Result<String> {
    let first: i32 = args["first"].convert()?;
    let second: i32 = args["second"].convert()?;

    Ok((first + second).to_string())
}

fn hello(args: HashMap<String, Value>, _context: &mut Context) -> Result<String> {
    Ok(format!("Hello, {}", args["who"]))
}

fn main() -> Result<()> {
    let mut repl = Repl::new(
        "MyApp",
        "v0.1.0",
        "My very cool app",
        Context::default(),
        None,
    );
    repl.add_command(
        Command::new("add", add)
            .with_parameter(Parameter::new("first").set_required(true)?)?
            .with_parameter(Parameter::new("second").set_required(true)?)?,
    );
    repl.add_command(
        Command::new("hello", hello)
            .with_parameter(Parameter::new("who").set_required(true)?)?
            .with_help("Greetings!"),
    );
    repl.run()
}
