use crate::error::*;
use crate::Callback;

#[derive(Debug)]
pub struct ParameterDefinition {
    pub name: String,
    pub datatype: Type,
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Debug)]
pub enum Type {
    String,
    Int,
    Float,
}

impl ParameterDefinition {
    // TODO: Builder?
    pub fn new(name: &str, datatype: Type, required: bool, default: Option<&str>) -> Result<Self> {
        let default = if default.is_some() {
            Some(default.unwrap().to_string())
        } else {
            None
        };
        Ok(Self {
            name: name.into(),
            datatype,
            required,
            default,
        })
    }
}

pub struct CommandDefinition<Context> {
    pub name: String,
    pub parameters: Vec<ParameterDefinition>,
    pub callback: Callback<Context>,
    pub help_summary: Option<String>,
}

impl<Context> CommandDefinition<Context> {
    pub fn new(
        name: &str,
        parameters: Vec<ParameterDefinition>,
        callback: Callback<Context>,
        help_summary: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            parameters,
            callback,
            help_summary,
        }
    }
}
