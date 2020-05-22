use crate::command_def::{CommandDefinition, ParameterDefinition};
use crate::error::*;
use crate::Callback;
use std::collections::HashMap;

pub struct Repl<Context> {
    editor: rustyline::Editor<()>,
    commands: HashMap<String, CommandDefinition<Context>>,
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
    ) -> Result<()> {
        self.validate_parameters(name, &parameters)?;
        self.commands.insert(
            name.to_string(),
            CommandDefinition::new(name, parameters, callback),
        );
        Ok(())
    }

    fn validate_arguments(
        &self,
        command: &str,
        definitions: &[ParameterDefinition],
        args: &[&str],
    ) -> Result<Vec<String>> {
        let mut validated = vec![];
        for (index, definition) in definitions.iter().enumerate() {
            if index < args.len() {
                validated.push(args[index].into());
            } else {
                if definition.required {
                    return Err(Error::MissingRequiredArgument(
                        command.into(),
                        definition.name.clone(),
                    ));
                } else if definition.default.is_some() {
                    validated.push(definition.default.clone().unwrap());
                }
            }
        }
        Ok(validated)
    }

    fn handle_command(&mut self, command: &str, args: &[&str]) -> Result<()> {
        match self.commands.get(command) {
            Some(definition) => {
                let validated = self.validate_arguments(&command, &definition.parameters, args)?;
                match (definition.callback)(&validated, &mut self.context) {
                    Ok(value) => println!("{}", value),
                    Err(value) => eprintln!("{}", value),
                };
            }
            None => eprintln!("Error: Unknown command {}", command),
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
        loop {
            let result = self.editor.readline(">> ");
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
