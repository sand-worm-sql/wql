use {crate::print::PrintOption, std::fmt::Debug, thiserror::Error as ThisError};

#[derive(Eq, Debug, PartialEq)]
pub enum SetOption {
    Tabular(bool),
    Colsep(String),
    Colwrap(String),
    Heading(bool),
}

impl SetOption {
    fn parse(key: &str, value: Option<&&str>, option: &PrintOption) -> Result<Self, CommandError> {
        fn bool_from(value: String) -> Result<bool, CommandError> {
            match value.to_uppercase().as_str() {
                "ON" => Ok(true),
                "OFF" => Ok(false),
                _ => Err(CommandError::WrongOption(value)),
            }
        }

        if let Some(value) = value {
            let value = match *value {
                "\"\"" => "",
                _ => value,
            }
            .to_owned();

            let set_option = match (key.to_lowercase().as_str(), &option.tabular) {
                ("tabular", _) => Self::Tabular(bool_from(value)?),
                ("colsep", false) => Self::Colsep(value),
                ("colwrap", false) => Self::Colwrap(value),
                ("heading", false) => Self::Heading(bool_from(value)?),
                (_, true) => return Err(CommandError::WrongOption("run .set tabular OFF".into())),

                _ => return Err(CommandError::WrongOption(key.into())),
            };

            Ok(set_option)
        } else {
            let payload = match key.to_lowercase().as_str() {
                "tabular" => "Usage: .set tabular {ON|OFF}",
                "colsep" => "Usage: .set colsep {\"\"|TEXT}",
                "colwrap" => "Usage: .set colwrap {\"\"|TEXT}",
                "heading" => "Usage: .set heading {ON|OFF}",

                _ => return Err(CommandError::WrongOption(key.into())),
            };

            Err(CommandError::LackOfValue(payload.into()))
        }
    }
}

#[derive(Eq, Debug, PartialEq)]
pub enum ShowOption {
    Tabular,
    Colsep,
    Colwrap,
    Heading,
    All,
}

impl ShowOption {
    fn parse(key: &str) -> Result<Self, CommandError> {
        let show_option = match key.to_lowercase().as_str() {
            "tabular" => Self::Tabular,
            "colsep" => Self::Colsep,
            "colwrap" => Self::Colwrap,
            "heading" => Self::Heading,
            "all" => Self::All,
            _ => return Err(CommandError::WrongOption(key.into())),
        };

        Ok(show_option)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Quit,
    Execute(String),
    ExecuteFromFile(String),
    SpoolOn(String),
    SpoolOff,
    Set(SetOption),
    Show(ShowOption),
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
                Some(&".version") => Ok(Self::Execute("SHOW VERSION".to_owned())),
                Some(&".spool") => match params.get(1) {
                    Some(&"off") => Ok(Self::SpoolOff),
                    Some(path) => Ok(Self::SpoolOn(path.to_string())),
                    None => Err(CommandError::LackOfFile),
                },
                Some(&".set") => match (params.get(1), params.get(2)) {
                    (Some(key), value) => Ok(Self::Set(SetOption::parse(key, value, option)?)),
                    (None, _) => Err(CommandError::LackOfOption),
                },
                Some(&".show") => match params.get(1) {
                    Some(key) => Ok(Self::Show(ShowOption::parse(key)?)),
                    None => Err(CommandError::LackOfOption),
                },
                Some(&".run") => Ok(Self::Run),
                _ => Err(CommandError::NotSupported),
            }
        } else {
            Ok(Self::Execute(line.to_owned()))
        }
    }
}
