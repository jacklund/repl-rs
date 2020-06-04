extern crate repl_rs;

use repl_rs::{Command, Parameter, Result, Value};
use repl_rs::{Convert, Repl};
use std::collections::{HashMap, VecDeque};

/// Example using Repl with Context

#[derive(Default)]
struct Context {
    list: VecDeque<String>,
}

// Append name to list
fn append(args: HashMap<String, Value>, context: &mut Context) -> Result<String> {
    let name: String = args["name"].convert()?;
    context.list.push_back(name);
    let list: Vec<String> = context.list.clone().into();

    Ok(list.join(", "))
}

// Prepend name to list
fn prepend(args: HashMap<String, Value>, context: &mut Context) -> Result<String> {
    let name: String = args["name"].convert()?;
    context.list.push_front(name);
    let list: Vec<String> = context.list.clone().into();

    Ok(list.join(", "))
}

fn main() -> Result<()> {
    let mut repl = Repl::new(Context::default())
        .with_name("MyApp")
        .with_version("v0.1.0")
        .with_description("My very cool app")
        .add_command(
            Command::new("append", append)
                .with_parameter(Parameter::new("name").set_required(true)?)?,
        )
        .add_command(
            Command::new("prepend", prepend)
                .with_parameter(Parameter::new("name").set_required(true)?)?,
        );
    repl.run()
}
