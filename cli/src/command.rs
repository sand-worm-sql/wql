use {crate::print::PrintOption, std::fmt::Debug, thiserror::Error as ThisError};

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Quit,
    Execute(String),
    ExecuteFromFile(String),
    SpoolOn(String),
    SpoolOff,
    Edit(Option<String>),
    Run,
}

#[derive(ThisError, Debug, PartialEq, Eq)]
pub enum CommandError {
    #[error("should specify table")]
    LackOfTable,
    #[error("should specify file path")]
    LackOfFile,
    #[error("should specify value for option")]
    LackOfValue(String),
    #[error("should specify option")]
    LackOfOption,
    #[error("cannot support option: {0}")]
    WrongOption(String),
    #[error("command not supported")]
    NotSupported,
    #[error("Nothing in SQL history to run.")]
    LackOfSQLHistory,
}

impl Command {
    pub fn parse(line: &str, option: &PrintOption) -> Result<Self, CommandError> {
        let line = line.trim_start().trim_end_matches(|c| c == ' ' || c == ';');
        // We detect if the line is a command or not
        if line.starts_with('.') {
            let params: Vec<&str> = line.split_whitespace().collect();
            match params.first() {
                Some(&".quit") => Ok(Self::Quit),
                Some(&".help") => Ok(Self::Help),
                _ => Err(CommandError::NotSupported),
            }
        } else {
            Ok(Self::Execute(line.to_owned()))
        }
    }
}
