extern crate rustyline;

use std::collections::HashMap;
use std::result::Result;

type Callback<Context> = fn(&[&str], &mut Context) -> Result<String, String>;

pub struct Repl<Context> {
    editor: rustyline::Editor<()>,
    commands: HashMap<String, Callback<Context>>,
    context: Context,
}

impl<Context> Repl<Context> {
    pub fn new(context: Context) -> Self {
        Self {
            editor: rustyline::Editor::new(),
            commands: HashMap::new(),
            context,
        }
    }

    pub fn add_command(&mut self, name: &str, callback: Callback<Context>) {
        self.commands.insert(name.to_string(), callback);
    }

    pub fn run(&mut self) {
        loop {
            let result = self.editor.readline(">> ");
            match result {
                Ok(line) => {
                    self.editor.add_history_entry(line.clone());
                    let mut args = line.trim().split_whitespace().collect::<Vec<&str>>();
                    let command: String = args.drain(..1).collect();
                    match self.commands.get(&command) {
                        Some(callback) => match callback(&args, &mut self.context) {
                            Ok(value) => println!("{}", value),
                            Err(value) => eprintln!("{}", value),
                        },
                        None => eprintln!("Error: Unknown command {}", command),
                    }
                }
                Err(rustyline::error::ReadlineError::Eof) => break,
                Err(error) => eprintln!("Error reading line: {}", error),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Repl;

    #[derive(Default)]
    struct Context {
        foobar: usize,
    }

    fn foo(args: &[&str], context: &mut Context) -> Result<String, String> {
        if args.is_empty() {
            Err("foo is missing args".into())
        } else {
            Ok(format!("foo {:?}", args))
        }
    }

    #[test]
    fn it_works() {
        let mut repl = Repl::new(Context::default());
        repl.add_command("foo", foo);
        repl.run();
    }
}
