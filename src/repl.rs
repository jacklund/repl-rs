use crate::command_def::{CommandDefinition, ParameterDefinition};
use crate::error::*;
use crate::help::{DefaultHelpViewer, HelpContext, HelpEntry, HelpViewer};
use crate::{Callback, Value};
use std::boxed::Box;
use std::collections::HashMap;
use std::fmt::Display;
use yansi::Paint;

pub struct Repl<Context> {
    editor: rustyline::Editor<()>,
    name: String,
    version: String,
    purpose: String,
    prompt: Box<dyn Display>,
    commands: HashMap<String, CommandDefinition<Context>>,
    context: Context,
    help_context: Option<HelpContext>,
    help_viewer: Box<dyn HelpViewer>,
}

impl<Context> Repl<Context> {
    pub fn new(
        name: &str,
        version: &str,
        purpose: &str,
        context: Context,
        mut prompt: Option<&'static dyn Display>,
    ) -> Self {
        let default_prompt: Box<dyn Display> = Box::new(Paint::green(format!("{}> ", name)).bold());
        let prompt: Box<dyn Display> = if prompt.is_some() {
            Box::new(prompt.take().unwrap())
        } else {
            default_prompt
        };

        Self {
            editor: rustyline::Editor::new(),
            name: name.into(),
            version: version.into(),
            purpose: purpose.into(),
            prompt,
            commands: HashMap::new(),
            context,
            help_context: None,
            help_viewer: Box::new(DefaultHelpViewer::new()),
        }
    }

    fn validate_parameters(&self, name: &str, parameters: &[ParameterDefinition]) -> Result<()> {
        let mut can_be_required = true;
        let mut can_be_defaulted = true;
        for parameter in parameters.iter() {
            if parameter.required {
                if !can_be_required {
                    return Err(Error::IllegalRequiredError(
                        name.into(),
                        parameter.name.clone(),
                    ));
                }
                if parameter.default.is_some() {
                    return Err(Error::IllegalDefaultError(
                        name.into(),
                        parameter.name.clone(),
                    ));
                }
            } else {
                can_be_required = false;
                if parameter.default.is_some() {
                    if !can_be_defaulted {
                        return Err(Error::IllegalDefaultError(
                            name.into(),
                            parameter.name.clone(),
                        ));
                    }
                } else {
                    can_be_defaulted = false;
                }
            }
        }

        Ok(())
    }

    pub fn add_command(
        &mut self,
        name: &str,
        parameters: Vec<ParameterDefinition>,
        callback: Callback<Context>,
        help_summary: Option<String>,
    ) -> Result<()> {
        self.validate_parameters(name, &parameters)?;
        self.commands.insert(
            name.to_string(),
            CommandDefinition::new(name, parameters, callback, help_summary),
        );
        Ok(())
    }

    fn validate_arguments(
        &self,
        command: &str,
        definitions: &[ParameterDefinition],
        args: &[&str],
    ) -> Result<HashMap<String, Value>> {
        if args.len() > definitions.len() {
            return Err(Error::TooManyArguments(command.into(), definitions.len()));
        }

        let mut validated = HashMap::new();
        for (index, definition) in definitions.iter().enumerate() {
            if index < args.len() {
                let converted = match definition.datatype.convert(args[index]) {
                    Ok(value) => Ok(value),
                    Err(error) => Err(Error::CommandError(format!(
                        "Error parsing parameter '{}' in command '{}': {}",
                        definition.name, command, error
                    ))),
                }?;
                validated.insert(definition.name.clone(), converted);
            } else if definition.required {
                return Err(Error::MissingRequiredArgument(
                    command.into(),
                    definition.name.clone(),
                ));
            } else if definition.default.is_some() {
                validated.insert(definition.name.clone(), definition.default.clone().unwrap());
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
        self.editor.add_history_entry(line.clone());
        let mut args = line.trim().split_whitespace().collect::<Vec<&str>>();
        let command: String = args.drain(..1).collect();
        self.handle_command(&command, &args)?;
        Ok(())
    }

    fn construct_help_context(&mut self) {
        let mut help_entries = self
            .commands
            .iter()
            .map(|(_, definition)| HelpEntry::new(&definition))
            .collect::<Vec<HelpEntry>>();
        help_entries.sort_by_key(|d| d.command.clone());
        self.help_context = Some(HelpContext::new(
            &self.name,
            &self.version,
            &self.purpose,
            help_entries,
        ));
    }

    pub fn run(&mut self) {
        self.construct_help_context();
        println!("Welcome to {} {}", self.name, self.version);
        loop {
            let result = self.editor.readline(&format!("{}", self.prompt));
            match result {
                Ok(line) => {
                    if let Err(error) = self.process_line(line) {
                        eprintln!("{}", error);
                    }
                }
                Err(rustyline::error::ReadlineError::Eof) => break,
                Err(error) => eprintln!("Error reading line: {}", error),
            }
        }
    }
}
