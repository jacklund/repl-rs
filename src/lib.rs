extern crate rustyline;

mod command_def;
mod error;
pub mod repl;

use error::*;

type Callback<Context> = fn(&[String], &mut Context) -> Result<String>;

#[cfg(test)]
mod tests {
    use crate::command_def::ParameterDefinition;
    use crate::error;
    use crate::repl::Repl;

    #[derive(Default)]
    struct Context {
        foobar: usize,
    }

    fn foo(args: &[String], context: &mut Context) -> error::Result<String> {
        Ok(format!("foo {:?}", args))
    }

    #[test]
    fn it_works() {
        let mut repl = Repl::new(Context::default());
        repl.add_command(
            "foo",
            vec![
                ParameterDefinition::new("bar", true, None),
                ParameterDefinition::new("baz", false, Some("blerf")),
            ],
            foo,
        );
        repl.run();
    }
}
