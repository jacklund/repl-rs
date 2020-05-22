use crate::error::*;
use crate::{Callback, Value};

#[derive(Debug)]
pub struct ParameterDefinition {
    pub name: String,
    pub datatype: Type,
    pub required: bool,
    pub default: Option<Value>,
}

#[derive(Debug)]
pub enum Type {
    String,
    Int,
    Float,
}

impl Type {
    pub fn convert(&self, val: &str) -> Result<Value> {
        Ok(match self {
            Type::String => Value::String(val.to_string()),
            Type::Int => Value::Int(val.parse::<i32>()?),
            Type::Float => Value::Float(val.parse::<f32>()?),
        })
    }
}

impl ParameterDefinition {
    pub fn new(name: &str, datatype: Type, required: bool, default: Option<&str>) -> Result<Self> {
        let default = if default.is_some() {
            Some(datatype.convert(default.unwrap())?)
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
