extern crate clap;
extern crate rustyline;

mod command;
mod error;
mod help;
mod parameter;
mod repl;
mod value;

pub use command::Command;
pub use error::{Error, Result};
pub use repl::Repl;
#[doc(inline)]
pub use value::{Convert, Value};

use std::collections::HashMap;

/// This is the signature for your command callback functions
pub type Callback<Context> = fn(HashMap<String, Value>, &mut Context) -> Result<Option<String>>;
