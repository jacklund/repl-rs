use std::convert::From;
use std::fmt;
use std::num;

/// Result type
pub type Result<T> = std::result::Result<T, Error>;

/// Error type
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Parameter is required when it shouldn't be
    IllegalRequiredError(String),

    /// Parameter is defaulted when it's also required
    IllegalDefaultError(String),

    /// A required argument is missing
    MissingRequiredArgument(String, String),

    /// Too many arguments were provided
    TooManyArguments(String, usize),

    /// Error parsing a bool value
    ParseBoolError(std::str::ParseBoolError),

    /// Error parsing an int value
    ParseIntError(num::ParseIntError),

    /// Error parsing a float value
    ParseFloatError(num::ParseFloatError),

    /// Generic error on command
    CommandError(String),

    /// Command not found
    UnknownCommand(String),
}

impl std::error::Error for Error {}

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
            Error::ParseBoolError(error) => write!(f, "Error: {}", error,),
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

impl From<std::str::ParseBoolError> for Error {
    fn from(error: std::str::ParseBoolError) -> Self {
        Error::ParseBoolError(error)
    }
}
