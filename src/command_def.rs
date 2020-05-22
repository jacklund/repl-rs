use crate::Callback;

#[derive(Debug)]
pub struct ParameterDefinition {
    pub name: String,
    pub required: bool,
    pub default: Option<String>,
}

impl ParameterDefinition {
    pub fn new(name: &str, required: bool, default: Option<&str>) -> Self {
        Self {
            name: name.into(),
            required,
            default: default.and_then(|s| Some(s.to_string())),
        }
    }
}

pub struct CommandDefinition<Context> {
    pub name: String,
    pub parameters: Vec<ParameterDefinition>,
    pub callback: Callback<Context>,
}

impl<Context> CommandDefinition<Context> {
    pub fn new(
        name: &str,
        parameters: Vec<ParameterDefinition>,
        callback: Callback<Context>,
    ) -> Self {
        Self {
            name: name.into(),
            parameters,
            callback,
        }
    }
}
