use crate::error::*;
use crate::Callback;
use std::fmt;

pub struct Command<Context> {
    pub(crate) name: String,
    pub(crate) parameters: Vec<Parameter>,
    pub(crate) callback: Callback<Context>,
    pub(crate) help_summary: Option<String>,
}

impl<Context> fmt::Debug for Command<Context> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Command")
            .field("name", &self.name)
            .field("parameters", &self.parameters)
            .field("help_summary", &self.help_summary)
            .finish()
    }
}

impl<Context> std::cmp::PartialEq for Command<Context> {
    fn eq(&self, other: &Command<Context>) -> bool {
        self.name == other.name
            && self.parameters == other.parameters
            && self.help_summary == other.help_summary
    }
}

impl<Context> Command<Context> {
    pub fn new(name: &str, callback: Callback<Context>) -> Self {
        Self {
            name: name.to_string(),
            parameters: vec![],
            callback,
            help_summary: None,
        }
    }

    pub fn with_parameter(mut self, parameter: Parameter) -> Result<Command<Context>> {
        if parameter.required {
            if self
                .parameters
                .iter()
                .find(|&param| param.required == false)
                .is_some()
            {
                return Err(Error::IllegalRequiredError(parameter.name.clone()));
            }
        }

        self.parameters.push(parameter);

        Ok(self)
    }

    pub fn with_help(mut self, help: &str) -> Command<Context> {
        self.help_summary = Some(help.to_string());

        self
    }
}

#[derive(Debug, PartialEq)]
pub struct Parameter {
    pub(crate) name: String,
    pub(crate) required: bool,
    pub(crate) default: Option<String>,
}

impl Parameter {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            required: false,
            default: None,
        }
    }

    pub fn set_required(mut self, required: bool) -> Result<Self> {
        if self.default.is_some() {
            return Err(Error::IllegalRequiredError(self.name));
        }
        self.required = required;

        Ok(self)
    }

    pub fn set_default(mut self, default: &str) -> Result<Self> {
        if self.required {
            return Err(Error::IllegalDefaultError(self.name));
        }
        self.default = Some(default.to_string());

        Ok(self)
    }
}
