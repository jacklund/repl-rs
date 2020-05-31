extern crate rustyline;

mod command_def;
mod error;
pub mod help;
pub mod repl;

use error::*;
use std::collections::HashMap;

type Callback<Context> = fn(HashMap<String, Value>, &mut Context) -> Result<String>;

#[derive(Clone, Debug)]
pub enum Value {
    String(String),
    Int(i32),
    Float(f32),
}
