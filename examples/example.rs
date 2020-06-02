extern crate repl_rs;

use repl_rs::{Convert, Repl};
use repl_rs::{ParameterDefinition, Result, Type, Value};
use std::collections::HashMap;

#[derive(Default)]
struct Context {
    foobar: usize,
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
        "add",
        vec![
            ParameterDefinition::new("first", Type::Int, true, None).unwrap(),
            ParameterDefinition::new("second", Type::Int, true, None).unwrap(),
        ],
        add,
        Some("Add two numbers".to_string()),
    )?;
    repl.add_command(
        "hello",
        vec![ParameterDefinition::new("who", Type::String, true, None).unwrap()],
        hello,
        Some("Greetings!".to_string()),
    )?;
    repl.run()
}
