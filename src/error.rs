use std::convert::From;
use std::fmt;
use std::num;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IllegalRequiredError(String, String),
    IllegalDefaultError(String, String),
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
            Error::IllegalDefaultError(command, parameter) => write!(
                f,
                "Error: Parameter '{}' in command '{}' cannot have a default",
                parameter, command
            ),
            Error::IllegalRequiredError(command, parameter) => write!(
                f,
                "Error: Parameter '{}' in command '{}' cannot be required",
                parameter, command
            ),
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
