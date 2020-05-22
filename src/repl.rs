use crate::command_def::{CommandDefinition, ParameterDefinition};
use crate::error::*;
use crate::{Callback, Value};
use std::collections::HashMap;

#[derive(Debug)]
struct HelpEntry {
    command: String,
    parameters: Vec<(String, bool)>,
    summary: Option<String>,
}

impl HelpEntry {
    fn new<Context>(command: &CommandDefinition<Context>) -> Self {
        Self {
            command: command.name.clone(),
            parameters: command
                .parameters
                .iter()
                .map(|pd| (pd.name.clone(), pd.required))
                .collect(),
            summary: command.help_summary.clone(),
        }
    }
}

pub struct Repl<Context> {
    editor: rustyline::Editor<()>,
    prompt: String,
    commands: HashMap<String, CommandDefinition<Context>>,
    context: Context,
    help: Option<Vec<HelpEntry>>,
}

impl<Context> Repl<Context> {
    pub fn new(prompt: &str, context: Context) -> Self {
        Self {
            editor: rustyline::Editor::new(),
            prompt: prompt.to_string(),
            commands: HashMap::new(),
            context,
            help: None,
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
            } else {
                if definition.required {
                    return Err(Error::MissingRequiredArgument(
                        command.into(),
                        definition.name.clone(),
                    ));
                } else if definition.default.is_some() {
                    validated.insert(definition.name.clone(), definition.default.clone().unwrap());
                }
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
            for entry in self.help.as_ref().unwrap() {
                print!("{}", entry.command);
                if entry.summary.is_some() {
                    print!(" - {}", entry.summary.clone().unwrap());
                }
                println!("");
            }
        } else {
            let entry_opt = self
                .help
                .as_ref()
                .unwrap()
                .iter()
                .find(|entry| entry.command == args[0]);
            if entry_opt.is_none() {
                eprintln!("No help for {} found", args[0]);
            } else {
                let entry = entry_opt.unwrap();
                if entry.summary.is_some() {
                    println!("{}", entry.summary.clone().unwrap());
                }
                println!("Usage:");
                print!("\t{}", entry.command);
                for param in entry.parameters.clone() {
                    if param.1 {
                        print!(" {}", param.0);
                    } else {
                        print!(" [{}]", param.0);
                    }
                }
            }
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

    pub fn run(&mut self) {
        let mut help_entries = self
            .commands
            .iter()
            .map(|(_, definition)| HelpEntry::new(&definition))
            .collect::<Vec<HelpEntry>>();
        help_entries.sort_by_key(|d| d.command.clone());
        self.help = Some(help_entries);
        loop {
            let result = self.editor.readline(&self.prompt);
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
