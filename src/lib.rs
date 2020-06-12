//! repl-rs - [REPL](https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop) library
//! for Rust
//!
//! # Example
//!
//! ```
//! use std::collections::HashMap;
//! use repl_rs::{Command, Parameter, Result, Value};
//! use repl_rs::{Convert, Repl};
//!
//! // Write "Hello"
//! fn hello<T>(args: HashMap<String, Value>, _context: &mut T) -> Result<Option<String>> {
//!     Ok(Some(format!("Hello, {}", args["who"])))
//! }
//!
//! fn main() -> Result<()> {
//!     let mut repl = Repl::new(())
//!         .with_name("MyApp")
//!         .with_version("v0.1.0")
//!         .with_description("My very cool app")
//!         .add_command(
//!              Command::new("hello", hello)
//!                  .with_parameter(Parameter::new("who").set_required(true)?)?
//!                  .with_help("Greetings!"),
//!     );
//!     repl.run()
//!  }
//! ```
//! repl-rs uses the [builder](https://en.wikipedia.org/wiki/Builder_pattern) pattern extensively.
//! What these lines are doing is:
//! - creating a repl with an empty Context (see below)
//! - with a name of "MyApp", the given version, and the given description
//! - and adding a "hello" command which calls out to the `hello` callback function defined above
//! - the `hello` command has a single parameter, "who", which is required, and has the given help
//! message
//!
//! The `hello` function takes a HashMap of named arguments, contained in a
//! [Value](struct.Value.html) struct, and an (unused) `Context`, which is used to hold state if you
//! need to - the initial context is passed in to the call to
//! [Repl::new](struct.Repl.html#method.new), in our case, `()`.
//! Because we're not using a Context, we need to include a generic type in our `hello` function,
//! because there's no way to pass an argument of type `()` otherwise.
//!
//! All command function callbacks return a `Result<Option<String>>`. This has the following
//! effect:
//! - If the return is `Ok(Some(String))`, it prints the string to stdout
//! - If the return is `Ok(None)`, it prints nothing
//! - If the return is an error, it prints the error message to stderr
//!
//! # Conversions
//!
//! The [Value](struct.Value.html) type has conversions defined for all the primitive types. Here's
//! how that works in practice:
//! ```
//! use repl_rs::{Command, Parameter, Result, Value};
//! use repl_rs::{Convert, Repl};
//! use std::collections::HashMap;
//!
//! // Add two numbers.
//! fn add<T>(args: HashMap<String, Value>, _context: &mut T) -> Result<Option<String>> {
//!     let first: i32 = args["first"].convert()?;
//!     let second: i32 = args["second"].convert()?;
//!
//!     Ok(Some((first + second).to_string()))
//! }
//!
//! fn main() -> Result<()> {
//!     let mut repl = Repl::new(())
//!         .with_name("MyApp")
//!         .with_version("v0.1.0")
//!         .with_description("My very cool app")
//!         .add_command(
//!             Command::new("add", add)
//!                 .with_parameter(Parameter::new("first").set_required(true)?)?
//!                 .with_parameter(Parameter::new("second").set_required(true)?)?
//!                 .with_help("Add two numbers together"),
//!     );
//!     repl.run()
//! }
//! ```
//! This example adds two numbers. The `convert()` function manages the conversion for you.
//!
//! # Context
//!
//! The `Context` type is used to keep state between REPL calls. Here's an example:
//! ```
//! use repl_rs::{Command, Parameter, Result, Value};
//! use repl_rs::{Convert, Repl};
//! use std::collections::{HashMap, VecDeque};
//!
//! #[derive(Default)]
//! struct Context {
//!     list: VecDeque<String>,
//! }
//!
//! // Append name to list
//! fn append(args: HashMap<String, Value>, context: &mut Context) -> Result<Option<String>> {
//!     let name: String = args["name"].convert()?;
//!     context.list.push_back(name);
//!     let list: Vec<String> = context.list.clone().into();
//!
//!     Ok(Some(list.join(", ")))
//! }
//!
//! // Prepend name to list
//! fn prepend(args: HashMap<String, Value>, context: &mut Context) -> Result<Option<String>> {
//!     let name: String = args["name"].convert()?;
//!     context.list.push_front(name);
//!     let list: Vec<String> = context.list.clone().into();
//!
//!     Ok(Some(list.join(", ")))
//! }
//!
//! fn main() -> Result<()> {
//!     let mut repl = Repl::new(Context::default())
//!         .add_command(
//!             Command::new("append", append)
//!                 .with_parameter(Parameter::new("name").set_required(true)?)?
//!                 .with_help("Append name to end of list"),
//!         )
//!         .add_command(
//!             Command::new("prepend", prepend)
//!                 .with_parameter(Parameter::new("name").set_required(true)?)?
//!                 .with_help("Prepend name to front of list"),
//!         );
//!     repl.run()
//! }
//! ```
//! A few things to note:
//! - you pass in the initial value for your Context struct to the call to
//! [Repl::new()](struct.Repl.html#method.new)
//! - the context is passed to your command callback functions as a mutable reference
//!
//! # Help
//! repl-rs has support for supplying help commands for your REPL. This is accomplished through the
//! [HelpViewer](trait.HelpViewer.html), which is a trait that has a default implementation which should give you pretty
//! much what you expect.
//! ```bash
//! % myapp
//! Welcome to MyApp v0.1.0
//! MyApp> help
//! MyApp v0.1.0: My very cool app
//! ------------------------------
//! append - Append name to end of list
//! prepend - Prepend name to front of list
//! MyApp> help append
//! append: Append name to end of list
//! Usage:
//!         append name
//! MyApp>
//! ```
//! If you want to roll your own help, just implement [HelpViewer](trait.HelpViewer.html) and add it to your REPL using the
//! [.with_help_viewer()](struct.Repl.html#method.with_help_viewer) method.
//!
//! # Errors
//!
//! Your command functions don't need to return `repl_rs::Error`; you can return any error from
//! them. Your error will need to implement `std::fmt::Display`, so the Repl can print the error,
//! and you'll need to implement `std::convert::From` for `repl_rs::Error` to your error type.
//! This makes error handling in your command functions easier, since you can just allow whatever
//! errors your functions emit bubble up.
//!
//! ```
//! use repl_rs::{Command, Parameter, Value};
//! use repl_rs::{Convert, Repl};
//! use std::collections::HashMap;
//! use std::fmt;
//! use std::result::Result;
//!
//! // My custom error type
//! #[derive(Debug)]
//! enum Error {
//!     DivideByZeroError,
//!     ReplError(repl_rs::Error),
//! }
//!
//! // Implement conversion from repl_rs::Error to my error type
//! impl From<repl_rs::Error> for Error {
//!     fn from(error: repl_rs::Error) -> Self {
//!         Error::ReplError(error)
//!     }
//! }
//!
//! // My error has to implement Display as well
//! impl fmt::Display for Error {
//!     fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
//!         match self {
//!             Error::DivideByZeroError => write!(f, "Whoops, divided by zero!"),
//!             Error::ReplError(error) => write!(f, "{}", error),
//!         }
//!     }
//! }
//!
//! // Divide two numbers.
//! fn divide<T>(args: HashMap<String, Value>, _context: &mut T) -> Result<Option<String>, Error> {
//!     let numerator: f32 = args["numerator"].convert()?;
//!     let denominator: f32 = args["denominator"].convert()?;
//!
//!     if denominator == 0.0 {
//!         return Err(Error::DivideByZeroError);
//!     }
//!
//!     Ok(Some((numerator / denominator).to_string()))
//! }
//!
//! fn main() -> Result<(), Error> {
//!     let mut repl = Repl::new(())
//!         .with_name("MyApp")
//!         .with_version("v0.1.0")
//!         .with_description("My very cool app")
//!         .add_command(
//!             Command::new("divide", divide)
//!                 .with_parameter(Parameter::new("numerator").set_required(true)?)?
//!                 .with_parameter(Parameter::new("denominator").set_required(true)?)?
//!                 .with_help("Divide two numbers"),
//!     );
//!     Ok(repl.run()?)
//! }
//! ```
//!
extern crate clap;
extern crate rustyline;

mod command;
mod error;
mod help;
mod parameter;
mod repl;
mod value;

pub use command::Command;
pub use error::{Error, Result};
#[doc(inline)]
pub use help::{HelpContext, HelpEntry, HelpViewer};
pub use parameter::Parameter;
#[doc(inline)]
pub use repl::Repl;
#[doc(inline)]
pub use value::{Convert, Value};

use std::collections::HashMap;

/// Command callback function signature
pub type Callback<Context, Error> =
    fn(HashMap<String, Value>, &mut Context) -> std::result::Result<Option<String>, Error>;

#[macro_export]
macro_rules! initialize_repl {
    ($context: expr) => {{
        let mut repl = Repl::new($context);
        repl.name = crate_name!().to_string();
        repl.version = crate_version!().to_string();
        repl.description = crate_description!().to_string();

        repl
    }};
}
