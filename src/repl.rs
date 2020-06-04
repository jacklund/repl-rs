use crate::command_def::{Command, Parameter};
use crate::error::*;
use crate::help::{DefaultHelpViewer, HelpContext, HelpEntry, HelpViewer};
use crate::Value;
use clap::{crate_description, crate_name, crate_version};
use std::boxed::Box;
use std::collections::HashMap;
use std::fmt::Display;
use yansi::Paint;

type ErrorHandler<Context> = fn(error: Error, repl: &Repl<Context>) -> Result<()>;

fn default_error_handler<Context>(error: Error, _repl: &Repl<Context>) -> Result<()> {
    eprintln!("{}", error);
    Ok(())
}

pub struct Repl<Context> {
    name: String,
    version: String,
    description: String,
    prompt: Box<dyn Display>,
    custom_prompt: bool,
    commands: HashMap<String, Command<Context>>,
    context: Context,
    help_context: Option<HelpContext>,
    help_viewer: Box<dyn HelpViewer>,
    error_handler: ErrorHandler<Context>,
}

impl<Context> Repl<Context> {
    pub fn new(context: Context) -> Self {
        let name = crate_name!().to_string();

        Self {
            name: name.clone(),
            version: crate_version!().to_string(),
            description: crate_description!().to_string(),
            prompt: Box::new(Paint::green(format!("{}> ", name)).bold()),
            custom_prompt: false,
            commands: HashMap::new(),
            context,
            help_context: None,
            help_viewer: Box::new(DefaultHelpViewer::new()),
            error_handler: default_error_handler,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        if !self.custom_prompt {
            self.prompt = Box::new(Paint::green(format!("{}> ", name)).bold());
        }

        self
    }

    pub fn with_version(mut self, version: &str) -> Self {
        self.version = version.to_string();

        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();

        self
    }

    pub fn with_prompt(mut self, prompt: &'static dyn Display) -> Self {
        self.prompt = Box::new(prompt);
        self.custom_prompt = true;

        self
    }

    pub fn with_help_viewer<V: 'static + HelpViewer>(mut self, help_viewer: V) -> Self {
        self.help_viewer = Box::new(help_viewer);

        self
    }

    pub fn with_error_handler(mut self, handler: ErrorHandler<Context>) -> Self {
        self.error_handler = handler;

        self
    }

    pub fn add_command(mut self, command: Command<Context>) -> Self {
        self.commands.insert(command.name.clone(), command);

        self
    }

    fn validate_arguments(
        &self,
        command: &str,
        parameters: &[Parameter],
        args: &[&str],
    ) -> Result<HashMap<String, Value>> {
        if args.len() > parameters.len() {
            return Err(Error::TooManyArguments(command.into(), parameters.len()));
        }

        let mut validated = HashMap::new();
        for (index, parameter) in parameters.iter().enumerate() {
            if index < args.len() {
                validated.insert(parameter.name.clone(), Value::new(&args[index]));
            } else if parameter.required {
                return Err(Error::MissingRequiredArgument(
                    command.into(),
                    parameter.name.clone(),
                ));
            } else if parameter.default.is_some() {
                validated.insert(
                    parameter.name.clone(),
                    Value::new(&parameter.default.clone().unwrap()),
                );
            }
        }
        Ok(validated)
    }

    fn handle_command(&mut self, command: &str, args: &[&str]) -> Result<()> {
        match self.commands.get(command) {
            Some(definition) => {
                let validated = self.validate_arguments(&command, &definition.parameters, args)?;
                match (definition.callback)(validated, &mut self.context) {
                    Ok(value) => println!("{}", value),
                    Err(value) => eprintln!("{}", value),
                };
            }
            None => {
                if command == "help" {
                    self.show_help(args)?;
                } else {
                    return Err(Error::UnknownCommand(command.to_string()));
                }
            }
        }

        Ok(())
    }

    fn show_help(&self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            self.help_viewer
                .help(None, &self.help_context.as_ref().unwrap())?;
        } else {
            self.help_viewer
                .help(Some(args[0]), &self.help_context.as_ref().unwrap())?;
        }
        Ok(())
    }

    fn process_line(&mut self, line: String) -> Result<()> {
        let mut args = line.trim().split_whitespace().collect::<Vec<&str>>();
        let command: String = args.drain(..1).collect();
        self.handle_command(&command, &args)?;
        Ok(())
    }

    fn construct_help_context(&mut self) {
        let mut help_entries = self
            .commands
            .iter()
            .map(|(_, definition)| {
                HelpEntry::new(
                    &definition.name,
                    &definition.parameters,
                    &definition.help_summary,
                )
            })
            .collect::<Vec<HelpEntry>>();
        help_entries.sort_by_key(|d| d.command.clone());
        self.help_context = Some(HelpContext::new(
            &self.name,
            &self.version,
            &self.description,
            help_entries,
        ));
    }

    pub fn run(&mut self) -> Result<()> {
        self.construct_help_context();
        let mut editor: rustyline::Editor<()> = rustyline::Editor::new();
        println!("Welcome to {} {}", self.name, self.version);
        loop {
            match editor.readline(&format!("{}", self.prompt)) {
                Ok(line) => {
                    editor.add_history_entry(line.clone());
                    if let Err(error) = self.process_line(line) {
                        (self.error_handler)(error, self)?;
                    }
                }
                Err(rustyline::error::ReadlineError::Eof) => break,
                Err(error) => eprintln!("Error reading line: {}", error),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::command_def::{Command, Parameter};
    use crate::error::*;
    use crate::repl::Repl;
    use crate::Value;
    use nix::sys::wait::{waitpid, WaitStatus};
    use nix::unistd::{close, dup2, fork, pipe, ForkResult};
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;
    use std::os::unix::io::FromRawFd;

    fn test_error_handler<Context>(error: Error, _repl: &Repl<Context>) -> Result<()> {
        Err(error)
    }

    #[derive(Default)]
    struct Context {
        _foobar: usize,
    }

    fn foo(args: HashMap<String, Value>, _context: &mut Context) -> Result<String> {
        Ok(format!("foo {:?}", args))
    }

    fn run_repl<Context>(mut repl: Repl<Context>, input: &str, expected: Result<()>) {
        let (rdr, wrtr) = pipe().unwrap();
        match fork() {
            Ok(ForkResult::Parent { child, .. }) => {
                // Parent
                let mut f = unsafe { File::from_raw_fd(wrtr) };
                write!(f, "{}", input).unwrap();
                if let WaitStatus::Exited(_, exit_code) = waitpid(child, None).unwrap() {
                    assert!(exit_code == 0);
                };
            }
            Ok(ForkResult::Child) => {
                // Child
                dup2(rdr, 0).unwrap();
                close(rdr).unwrap();
                let result = repl.run();
                if expected == result {
                    std::process::exit(0);
                } else {
                    eprintln!("Expected {:?}, got {:?}", expected, result);
                    std::process::exit(1);
                }
            }
            Err(_) => println!("Fork failed"),
        }
    }

    #[test]
    fn test_missing_required_arg_fails() -> Result<()> {
        let mut repl = Repl::new(
            "test",
            "v0.1.0",
            "Testing 1, 2, 3...",
            Context::default(),
            None,
        );
        repl.set_error_handler(test_error_handler);
        repl.add_command(
            Command::new("foo", foo)
                .with_parameter(Parameter::new("bar").set_required(true)?)?
                .with_parameter(Parameter::new("baz").set_required(true)?)?
                .with_help("Do foo when you can"),
        );
        run_repl(
            repl,
            "foo bar\n",
            Err(Error::MissingRequiredArgument("foo".into(), "baz".into())),
        );

        Ok(())
    }

    #[test]
    fn test_unknown_command_fails() -> Result<()> {
        let mut repl = Repl::new(
            "test",
            "v0.1.0",
            "Testing 1, 2, 3...",
            Context::default(),
            None,
        );
        repl.set_error_handler(test_error_handler);
        repl.add_command(
            Command::new("foo", foo)
                .with_parameter(Parameter::new("bar").set_required(true)?)?
                .with_parameter(Parameter::new("baz").set_default("20")?)?
                .with_help("Do foo when you can"),
        );
        run_repl(
            repl,
            "bar baz\n",
            Err(Error::UnknownCommand("bar".to_string())),
        );

        Ok(())
    }

    #[test]
    fn test_no_required_after_optional() -> Result<()> {
        assert_eq!(
            Err(Error::IllegalRequiredError("bar".into())),
            Command::new("foo", foo)
                .with_parameter(Parameter::new("baz").set_default("20")?)?
                .with_parameter(Parameter::new("bar").set_required(true)?)
        );

        Ok(())
    }

    #[test]
    fn test_required_cannot_be_defaulted() -> Result<()> {
        assert_eq!(
            Err(Error::IllegalDefaultError("bar".into())),
            Parameter::new("bar").set_required(true)?.set_default("foo")
        );

        Ok(())
    }
}
