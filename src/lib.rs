extern crate rustyline;

mod command_def;
mod error;
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

#[cfg(test)]
mod tests {
    use crate::command_def::{ParameterDefinition, Type};
    use crate::repl::Repl;
    use crate::{error, Value};
    use std::collections::HashMap;

    #[derive(Default)]
    struct Context {
        foobar: usize,
    }

    fn foo(args: HashMap<String, Value>, context: &mut Context) -> error::Result<String> {
        Ok(format!("foo {:?}", args))
    }

    #[test]
    fn it_works() -> error::Result<()> {
        let mut repl = Repl::new(Context::default());
        repl.add_command(
            "foo",
            vec![
                ParameterDefinition::new("bar", Type::String, true, None)?,
                ParameterDefinition::new("baz", Type::Int, false, Some("20"))?,
            ],
            foo,
        );
        repl.run();
        Ok(())
    }
}
