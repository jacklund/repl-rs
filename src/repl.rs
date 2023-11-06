use crate::error::*;
use crate::help::{DefaultHelpViewer, HelpContext, HelpEntry, HelpViewer};
use crate::Value;
use crate::{Command, Parameter};
use rustyline::completion;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline_derive::{Helper, Hinter, Validator};
use std::borrow::Cow::{self, Borrowed, Owned};
use std::boxed::Box;
use std::collections::HashMap;
use std::fmt::Display;
use yansi::Paint;

type ErrorHandler<Context, E> = fn(error: E, repl: &Repl<Context, E>) -> Result<()>;

fn default_error_handler<Context, E: std::fmt::Display>(
    error: E,
    _repl: &Repl<Context, E>,
) -> Result<()> {
    eprintln!("{}", error);
    Ok(())
}

/// Main REPL struct
pub struct Repl<Context, E: std::fmt::Display> {
    name: String,
    version: String,
    description: String,
    prompt: Box<dyn Display>,
    styled_prompt: Box<dyn Display>,
    custom_prompt: bool,
    commands: HashMap<String, Command<Context, E>>,
    context: Context,
    help_context: Option<HelpContext>,
    help_viewer: Box<dyn HelpViewer>,
    error_handler: ErrorHandler<Context, E>,
    use_completion: bool,
}

impl<Context, E> Repl<Context, E>
where
    E: Display + From<Error>,
{
    /// Create a new Repl with the given context's initial value.
    pub fn new(context: Context) -> Self {
        let name = String::new();

        Self {
            name: name.clone(),
            version: String::new(),
            description: String::new(),
            prompt: Box::new(format!("{}> ", name)),
            styled_prompt: Box::new(Paint::green(format!("{}> ", name)).bold()),
            custom_prompt: false,
            commands: HashMap::new(),
            context,
            help_context: None,
            help_viewer: Box::new(DefaultHelpViewer::new()),
            error_handler: default_error_handler,
            use_completion: false,
        }
    }

    /// Give your Repl a name. This is used in the help summary for the Repl.
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        if !self.custom_prompt {
            self.prompt = Box::new(format!("{}> ", name));
            self.styled_prompt = Box::new(Paint::green(format!("{}> ", name)).bold());
        }

        self
    }

    /// Give your Repl a version. This is used in the help summary for the Repl.
    pub fn with_version(mut self, version: &str) -> Self {
        self.version = version.to_string();

        self
    }

    /// Give your Repl a description. This is used in the help summary for the Repl.
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();

        self
    }

    /// Give your Repl a custom prompt. This prompt should not contain any escape sequences (e.g. for coloring).
    /// In case you want to use a styled prompt use this function together with with_styled_prompt. Both functions
    /// should then receive the same prompt as argument but only `with_styled_prompt` should contain the escape
    /// sequences for styling.
    ///
    /// The default prompt is the Repl name, followed by a `>`, all in green, followed by a space.
    pub fn with_prompt(mut self, prompt: &'static dyn Display) -> Self {
        self.prompt = Box::new(prompt);
        self.styled_prompt = Box::new(prompt);
        self.custom_prompt = true;

        self
    }

    /// Give your Repl a custom prompt which allows styling. In case you wan't to use a prompt which uses escape
    /// sequences you have to use both functions `with_prompt` and `with_styled_prompt`.
    pub fn with_styled_prompt(mut self, prompt: &'static dyn Display) -> Self {
        self.styled_prompt = Box::new(prompt);

        self
    }

    /// Pass in a custom help viewer
    pub fn with_help_viewer<V: 'static + HelpViewer>(mut self, help_viewer: V) -> Self {
        self.help_viewer = Box::new(help_viewer);

        self
    }

    /// Pass in a custom error handler. This is really only for testing - the default
    /// error handler simply prints the error to stderr and then returns
    pub fn with_error_handler(mut self, handler: ErrorHandler<Context, E>) -> Self {
        self.error_handler = handler;

        self
    }

    /// Set whether to use command completion when tab is hit. Defaults to false.
    pub fn use_completion(mut self, value: bool) -> Self {
        self.use_completion = value;

        self
    }

    /// Add a command to your REPL
    pub fn add_command(mut self, command: Command<Context, E>) -> Self {
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
                validated.insert(parameter.name.clone(), Value::new(args[index]));
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

    fn handle_command(&mut self, command: &str, args: &[&str]) -> core::result::Result<(), E> {
        match self.commands.get(command) {
            Some(definition) => {
                let validated = self.validate_arguments(command, &definition.parameters, args)?;
                match (definition.callback)(validated, &mut self.context) {
                    Ok(Some(value)) => println!("{}", value),
                    Ok(None) => (),
                    Err(error) => return Err(error),
                };
            }
            None => {
                if command == "help" {
                    self.show_help(args)?;
                } else {
                    return Err(Error::UnknownCommand(command.to_string()).into());
                }
            }
        }

        Ok(())
    }

    fn show_help(&self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            self.help_viewer
                .help_general(self.help_context.as_ref().unwrap())?;
        } else {
            let entry_opt = self
                .help_context
                .as_ref()
                .unwrap()
                .help_entries
                .iter()
                .find(|entry| entry.command == args[0]);
            match entry_opt {
                Some(entry) => {
                    self.help_viewer.help_command(entry)?;
                }
                None => eprintln!("Help not found for command '{}'", args[0]),
            };
        }
        Ok(())
    }

    fn process_line(&mut self, line: String) -> core::result::Result<(), E> {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let r = regex::Regex::new(r#"("[^"\n]+"|[\S]+)"#).unwrap();
            let args = r
                .captures_iter(trimmed)
                .map(|a| a[0].to_string().replace('\"', ""))
                .collect::<Vec<String>>();
            let mut args = args.iter().fold(vec![], |mut state, a| {
                state.push(a.as_str());
                state
            });
            let command: String = args.drain(..1).collect();
            self.handle_command(&command, &args)?;
        }
        Ok(())
    }

    fn construct_help_context(&mut self) {
        let mut help_entries = self
            .commands
            .values()
            .map(|definition| {
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

    fn create_helper(&mut self) -> Helper {
        let mut helper = Helper::new(self.styled_prompt.to_string());
        if self.use_completion {
            for name in self.commands.keys() {
                helper.add_command(name.to_string());
            }
        }

        helper
    }

    pub fn run(&mut self) -> Result<()> {
        self.construct_help_context();
        let mut editor: rustyline::Editor<Helper> = rustyline::Editor::new();
        let helper = Some(self.create_helper());
        editor.set_helper(helper);
        println!("Welcome to {} {}", self.name, self.version);
        let mut eof = false;
        while !eof {
            self.handle_line(&mut editor, &mut eof)?;
        }

        Ok(())
    }

    fn handle_line(
        &mut self,
        editor: &mut rustyline::Editor<Helper>,
        eof: &mut bool,
    ) -> Result<()> {
        match editor.readline(&format!("{}", self.prompt)) {
            Ok(line) => {
                editor.add_history_entry(line.clone());
                if let Err(error) = self.process_line(line) {
                    (self.error_handler)(error, self)?;
                }
                *eof = false;
                Ok(())
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                *eof = true;
                Ok(())
            }
            Err(error) => {
                eprintln!("Error reading line: {}", error);
                *eof = false;
                Ok(())
            }
        }
    }
}

// rustyline Helper struct
// Currently just does command completion with <tab>, if
// use_completion() is set on the REPL
#[derive(Helper, Hinter, Validator)]
struct Helper {
    commands: Vec<String>,
    highlighter: MatchingBracketHighlighter,
    colored_prompt: String,
}

impl Helper {
    fn new(styled_prompt: String) -> Self {
        Self {
            commands: vec![],
            highlighter: MatchingBracketHighlighter::new(),
            colored_prompt: styled_prompt,
        }
    }

    fn add_command(&mut self, command: String) {
        self.commands.push(command);
    }
}

impl Highlighter for Helper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

impl completion::Completer for Helper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        _pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        // Complete based on whether the current line is a substring
        // of one of the set commands
        let ret: Vec<Self::Candidate> = self
            .commands
            .iter()
            .filter(|cmd| cmd.contains(line))
            .map(|s| s.to_string())
            .collect();
        Ok((0, ret))
    }
}

#[cfg(all(test, unix))]
mod tests {
    use crate::error::*;
    use crate::repl::{Helper, Repl};
    use crate::{initialize_repl, Value};
    use crate::{Command, Parameter};
    use clap::{crate_description, crate_name, crate_version};
    use nix::sys::wait::{waitpid, WaitStatus};
    use nix::unistd::{close, dup2, fork, pipe, ForkResult};
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;
    use std::os::unix::io::FromRawFd;

    fn test_error_handler<Context>(error: Error, _repl: &Repl<Context, Error>) -> Result<()> {
        Err(error)
    }

    fn foo<T>(args: HashMap<String, Value>, _context: &mut T) -> Result<Option<String>> {
        Ok(Some(format!("foo {:?}", args)))
    }

    fn run_repl<Context>(mut repl: Repl<Context, Error>, input: &str, expected: Result<()>) {
        let (rdr, wrtr) = pipe().unwrap();
        unsafe {
            match fork() {
                Ok(ForkResult::Parent { child, .. }) => {
                    // Parent
                    let mut f = File::from_raw_fd(wrtr);
                    write!(f, "{}", input).unwrap();
                    if let WaitStatus::Exited(_, exit_code) = waitpid(child, None).unwrap() {
                        assert!(exit_code == 0);
                    };
                }
                Ok(ForkResult::Child) => {
                    std::panic::set_hook(Box::new(|panic_info| {
                        println!("Caught panic: {:?}", panic_info);
                        if let Some(location) = panic_info.location() {
                            println!(
                                "panic occurred in file '{}' at line {}",
                                location.file(),
                                location.line(),
                            );
                        } else {
                            println!("panic occurred but can't get location information...");
                        }
                    }));

                    dup2(rdr, 0).unwrap();
                    close(rdr).unwrap();
                    let mut editor: rustyline::Editor<Helper> = rustyline::Editor::new();
                    let mut eof = false;
                    let result = repl.handle_line(&mut editor, &mut eof);
                    let _ = std::panic::take_hook();
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
    }

    #[test]
    fn test_initialize_sets_crate_values() -> Result<()> {
        let repl: Repl<(), Error> = initialize_repl!(());

        assert_eq!(crate_name!(), repl.name);
        assert_eq!(crate_version!(), repl.version);
        assert_eq!(crate_description!(), repl.description);

        Ok(())
    }

    #[test]
    fn test_empty_line_does_nothing() -> Result<()> {
        let repl = Repl::new(())
            .with_name("test")
            .with_version("v0.1.0")
            .with_description("Testing 1, 2, 3...")
            .with_error_handler(test_error_handler)
            .add_command(
                Command::new("foo", foo)
                    .with_parameter(Parameter::new("bar").set_required(true)?)?
                    .with_parameter(Parameter::new("baz").set_required(true)?)?
                    .with_help("Do foo when you can"),
            );
        run_repl(repl, "\n", Ok(()));

        Ok(())
    }

    #[test]
    fn test_missing_required_arg_fails() -> Result<()> {
        let repl = Repl::new(())
            .with_name("test")
            .with_version("v0.1.0")
            .with_description("Testing 1, 2, 3...")
            .with_error_handler(test_error_handler)
            .add_command(
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
        let repl = Repl::new(())
            .with_name("test")
            .with_version("v0.1.0")
            .with_description("Testing 1, 2, 3...")
            .with_error_handler(test_error_handler)
            .add_command(
                Command::new("foo", foo)
                    .with_parameter(Parameter::new("bar").set_required(true)?)?
                    .with_parameter(Parameter::new("baz").set_required(true)?)?
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
            Command::<(), Error>::new("foo", foo)
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

    #[test]
    fn test_string_with_spaces_for_argument() -> Result<()> {
        let repl = Repl::new(())
            .with_name("test")
            .with_version("v0.1.0")
            .with_description("Testing 1, 2, 3...")
            .with_error_handler(test_error_handler)
            .add_command(
                Command::new("foo", foo)
                    .with_parameter(Parameter::new("bar").set_required(true)?)?
                    .with_parameter(Parameter::new("baz").set_required(true)?)?
                    .with_help("Do foo when you can"),
            );
        run_repl(repl, "foo \"baz test 123\" foo\n", Ok(()));

        Ok(())
    }

    #[test]
    fn test_string_with_spaces_for_argument_last() -> Result<()> {
        let repl = Repl::new(())
            .with_name("test")
            .with_version("v0.1.0")
            .with_description("Testing 1, 2, 3...")
            .with_error_handler(test_error_handler)
            .add_command(
                Command::new("foo", foo)
                    .with_parameter(Parameter::new("bar").set_required(true)?)?
                    .with_parameter(Parameter::new("baz").set_required(true)?)?
                    .with_help("Do foo when you can"),
            );
        run_repl(repl, "foo foo \"baz test 123\"\n", Ok(()));

        Ok(())
    }
}
