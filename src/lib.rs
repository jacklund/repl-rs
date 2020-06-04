extern crate clap;
extern crate rustyline;

mod command_def;
mod error;
pub mod help;
pub mod repl;

pub use command_def::{Command, Parameter, Type};
pub use error::{Error, Result};
pub use repl::Repl;

use std::collections::HashMap;
use std::fmt;

type Callback<Context> = fn(HashMap<String, Value>, &mut Context) -> Result<String>;

#[derive(Clone, Debug)]
pub struct Value {
    value: String,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

pub trait Convert<T> {
    fn convert(&self) -> Result<T>;
}

impl Value {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

impl Convert<String> for Value {
    fn convert(&self) -> Result<String> {
        Ok(self.value.to_string())
    }
}

macro_rules! add_num_converter {
    ($type: ident) => {
        impl Convert<$type> for Value {
            fn convert(&self) -> Result<$type> {
                Ok(self.value.parse::<$type>()?)
            }
        }
    };
}

add_num_converter!(i8);
add_num_converter!(i16);
add_num_converter!(i32);
add_num_converter!(i64);
add_num_converter!(i128);
add_num_converter!(isize);
add_num_converter!(u8);
add_num_converter!(u16);
add_num_converter!(u32);
add_num_converter!(u64);
add_num_converter!(u128);
add_num_converter!(usize);
add_num_converter!(f32);
add_num_converter!(f64);
