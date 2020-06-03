use std::convert::From;
use std::fmt;
use std::num;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    IllegalRequiredError(String),
    IllegalDefaultError(String),
    MissingRequiredArgument(String, String),
    TooManyArguments(String, usize),
    ParseIntError(num::ParseIntError),
    ParseFloatError(num::ParseFloatError),
    CommandError(String),
    UnknownCommand(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        match self {
            Error::IllegalDefaultError(parameter) => {
                write!(f, "Error: Parameter '{}' cannot have a default", parameter)
            }
            Error::IllegalRequiredError(parameter) => {
                write!(f, "Error: Parameter '{}' cannot be required", parameter)
            }
            Error::MissingRequiredArgument(command, parameter) => write!(
                f,
                "Error: Missing required argument '{}' for command '{}'",
                parameter, command
            ),
            Error::TooManyArguments(command, nargs) => write!(
                f,
                "Error: Command '{}' can have no more than {} arguments",
                command, nargs,
            ),
            Error::ParseFloatError(error) => write!(f, "Error: {}", error,),
            Error::ParseIntError(error) => write!(f, "Error: {}", error,),
            Error::CommandError(error) => write!(f, "Error: {}", error),
            Error::UnknownCommand(command) => write!(f, "Error: Unknown command '{}'", command),
        }
    }
}

impl From<num::ParseIntError> for Error {
    fn from(error: num::ParseIntError) -> Self {
        Error::ParseIntError(error)
    }
}

impl From<num::ParseFloatError> for Error {
    fn from(error: num::ParseFloatError) -> Self {
        Error::ParseFloatError(error)
    }
}
