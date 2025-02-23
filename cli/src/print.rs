use {
    crate::command::{
        SetOption,
        ShowOption,
    },
    std::{
        fs::File,
        io::{Result as IOResult, Write},
        path::Path,
        fmt::Display,
    },
    tabled::{builder::Builder, Style, Table},
};

pub struct Print<W: Write> {
    pub output: W,
    spool_file: Option<File>,
    pub option: PrintOption,
}

pub struct PrintOption {
    pub tabular: bool,
    colsep: String,
    colwrap: String,
    heading: bool,
}

impl PrintOption {
    pub fn tabular(&mut self, tabular: bool) {
        match tabular {
            true => {
                self.tabular = tabular;
                self.colsep("|".into());
                self.colwrap("".into());
                self.heading(true);
            }
            false => self.tabular = tabular,
        }
    }

    fn colsep(&mut self, colsep: String) {
        self.colsep = colsep;
    }

    fn colwrap(&mut self, colwrap: String) {
        self.colwrap = colwrap;
    }

    fn heading(&mut self, heading: bool) {
        self.heading = heading;
    }

    fn format(&self, option: ShowOption) -> String {
        fn string_from(value: &bool) -> String {
            match value {
                true => "ON".into(),
                false => "OFF".into(),
            }
        }
        match option {
            ShowOption::Tabular => format!("tabular {}", string_from(&self.tabular)),
            ShowOption::Colsep => format!("colsep \"{}\"", self.colsep),
            ShowOption::Colwrap => format!("colwrap \"{}\"", self.colwrap),
            ShowOption::Heading => format!("heading {}", string_from(&self.heading)),
            ShowOption::All => format!(
                "{}\n{}\n{}\n{}",
                self.format(ShowOption::Tabular),
                self.format(ShowOption::Colsep),
                self.format(ShowOption::Colwrap),
                self.format(ShowOption::Heading),
            ),
        }
    }
}

impl Default for PrintOption {
    fn default() -> Self {
        Self {
            tabular: true,
            colsep: "|".into(),
            colwrap: "".into(),
            heading: true,
        }
    }
}

impl<'a, W: Write> Print<W> {
    pub fn new(output: W, spool_file: Option<File>, option: PrintOption) -> Self {
        Print {
            output,
            spool_file,
            option,
        }
    }

    pub fn help(&mut self) -> IOResult<()> {
        const HEADER: [&str; 2] = ["command", "description"];
        const CONTENT: [[&str; 2]; 12] = [
            [".help", "show help"],
            [".quit", "quit program"],
            [".tables", "show table names"],
            [".functions", "show function names"],
            [".columns TABLE", "show columns from TABLE"],
            [".version", "show version"],
            [".execute PATH", "execute SQL from PATH"],
            [".spool PATH|off", "spool to PATH or off"],
            [".show OPTION", "show print option eg).show all"],
            [".set OPTION", "set print option eg).set tabular off"],
            [".edit [PATH]", "open editor with last command or PATH"],
            [".run ", "execute last command"],
        ];

        let mut table = self.get_table(HEADER);
        for row in CONTENT {
            table.add_record(row);
        }
        let table = self.build_table(table);

        writeln!(self.output, "{}\n", table)
    }

    // pub fn clear(&mut self)  -> IOResult<()>  {

    // //   Ok(())
    // }

    fn write_rows(
        &mut self,
        rows: impl Iterator<Item = impl Iterator<Item = String>>,
    ) -> IOResult<()> {
        for row in rows {
            let row = row
                .map(|v| format!("{c}{v}{c}", c = self.option.colwrap))
                .collect::<Vec<_>>()
                .join(self.option.colsep.as_str());

            self.write(row)?;
        }

        Ok(())
    }

    fn write_lf(&mut self, payload: impl Display, lf: &str) -> IOResult<()> {
        if let Some(file) = &self.spool_file {
            writeln!(file.to_owned(), "{payload}{lf}")?;
        };

        writeln!(self.output, "{payload}{lf}")
    }

    fn write(&mut self, payload: impl Display) -> IOResult<()> {
        self.write_lf(payload, "")
    }

    fn writeln(&mut self, payload: impl Display) -> IOResult<()> {
        self.write_lf(payload, "\n")
    }

    fn write_header<'b>(&mut self, labels: impl Iterator<Item = &'b str>) -> IOResult<()> {
        let PrintOption {
            heading,
            colsep,
            colwrap,
            ..
        } = &self.option;

        if !heading {
            return Ok(());
        }

        let labels = labels
            .map(|v| format!("{colwrap}{v}{colwrap}"))
            .collect::<Vec<_>>()
            .join(colsep.as_str());

        self.write(labels)
    }

    pub fn show_option(&mut self, option: ShowOption) -> IOResult<()> {
        let payload = self.option.format(option);
        self.writeln(payload)?;

        Ok(())
    }

    pub fn set_option(&mut self, option: SetOption) {
        match option {
            SetOption::Tabular(value) => self.option.tabular(value),
            SetOption::Colsep(value) => self.option.colsep(value),
            SetOption::Colwrap(value) => self.option.colwrap(value),
            SetOption::Heading(value) => self.option.heading(value),
        }
    }



    pub fn spool_on<P: AsRef<Path>>(&mut self, filename: P) -> IOResult<()> {
        let file = File::create(filename)?;
        self.spool_file = Some(file);

        Ok(())
    }

    pub fn spool_off(&mut self) {
        self.spool_file = None;
    }

    fn get_table<T: IntoIterator<Item = &'a str>>(&self, headers: T) -> Builder {
        let mut table = Builder::default();
        table.set_columns(headers);

        table
    }

    fn build_table(&self, builder: Builder) -> Table {
        builder.build().with(Style::markdown())
    }
}

#[cfg(test)]
mod tests {}
