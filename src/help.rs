use crate::command_def::CommandDefinition;
use crate::error::*;
use yansi::Paint;

#[derive(Debug)]
pub struct HelpEntry {
    pub command: String,
    pub parameters: Vec<(String, bool)>,
    pub summary: Option<String>,
}

impl HelpEntry {
    pub fn new<Context>(command: &CommandDefinition<Context>) -> Self {
        Self {
            command: command.name.clone(),
            parameters: command
                .parameters
                .iter()
                .map(|pd| (pd.name.clone(), pd.required))
                .collect(),
            summary: command.help_summary.clone(),
        }
    }
}

pub struct HelpContext {
    app_name: String,
    app_version: String,
    app_purpose: String,
    help_entries: Vec<HelpEntry>,
}

impl HelpContext {
    pub fn new(
        app_name: &str,
        app_version: &str,
        app_purpose: &str,
        help_entries: Vec<HelpEntry>,
    ) -> Self {
        Self {
            app_name: app_name.into(),
            app_version: app_version.into(),
            app_purpose: app_purpose.into(),
            help_entries,
        }
    }
}

pub trait HelpViewer {
    fn help(&self, command: Option<&str>, context: &HelpContext) -> Result<()>;
}

pub struct DefaultHelpViewer {}

impl DefaultHelpViewer {
    pub fn new() -> Self {
        Self {}
    }
}

impl HelpViewer for DefaultHelpViewer {
    fn help(&self, command: Option<&str>, context: &HelpContext) -> Result<()> {
        if command.is_none() {
            self.print_help_header(context);
            for entry in &context.help_entries {
                print!("{}", entry.command);
                if entry.summary.is_some() {
                    print!(" - {}", entry.summary.clone().unwrap());
                }
                println!();
            }
        } else {
            let entry_opt = context
                .help_entries
                .iter()
                .find(|entry| entry.command == command.clone().unwrap());
            match entry_opt {
                Some(entry) => {
                    if entry.summary.is_some() {
                        println!("{}: {}", entry.command, entry.summary.clone().unwrap());
                    }
                    println!("Usage:");
                    print!("\t{}", entry.command);
                    for param in entry.parameters.clone() {
                        if param.1 {
                            print!(" {}", param.0);
                        } else {
                            print!(" [{}]", param.0);
                        }
                    }
                }
                None => eprintln!("No help for {} found", command.unwrap()),
            }
        }
        Ok(())
    }
}

impl DefaultHelpViewer {
    fn print_help_header(&self, context: &HelpContext) {
        let header = format!(
            "{} {}: {}",
            context.app_name, context.app_version, context.app_purpose
        );
        let underline = Paint::new(
            std::iter::repeat(" ")
                .take(header.len())
                .collect::<String>(),
        )
        .strikethrough();
        println!("{}", header);
        println!("{}", underline);
    }
}
