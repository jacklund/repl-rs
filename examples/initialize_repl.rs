#[macro_use]
extern crate clap;

use repl_rs::{initialize_repl, Convert, Repl};
use repl_rs::{Command, Parameter, Result, Value};
use std::collections::{HashMap, VecDeque};

/// Example using initialize_repl

#[derive(Default)]
struct Context {
    list: VecDeque<String>,
}

// Append name to list
fn append(args: HashMap<String, Value>, context: &mut Context) -> Result<Option<String>> {
    let name: String = args["name"].convert()?;
    context.list.push_back(name);
    let list: Vec<String> = context.list.clone().into();

    Ok(Some(list.join(", ")))
}

// Prepend name to list
fn prepend(args: HashMap<String, Value>, context: &mut Context) -> Result<Option<String>> {
    let name: String = args["name"].convert()?;
    context.list.push_front(name);
    let list: Vec<String> = context.list.clone().into();

    Ok(Some(list.join(", ")))
}

fn main() -> Result<()> {
    let mut repl = initialize_repl!(Context::default())
        .use_completion(true)
        .add_command(
            Command::new("append", append)
                .with_parameter(Parameter::new("name").set_required(true)?)?
                .with_help("Append name to end of list"),
        )
        .add_command(
            Command::new("prepend", prepend)
                .with_parameter(Parameter::new("name").set_required(true)?)?
                .with_help("Prepend name to front of list"),
        );
    repl.run()
}
