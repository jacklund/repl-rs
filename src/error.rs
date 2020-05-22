use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    IllegalRequiredError(String, String),
    IllegalDefaultError(String, String),
    MissingRequiredArgument(String, String),
    TooManyArguments(String, usize),
    CommandError(String),
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
            Error::CommandError(error) => write!(f, "Error: {}", error),
        }
    }
}
