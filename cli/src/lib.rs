mod cli;
mod command;
mod helper;
mod print;

use {
    crate::cli::Cli,
    anyhow::Result,
    clap::Parser,
    std::{
        fmt::Debug,
        fs::File,
        io::Write,
        path::{Path, PathBuf},
    },
};

#[derive(Parser, Debug)]
#[clap(name = "sand-worm-cli", about, version)]
struct Args {
    /// SQL file to execute
    #[clap(short, long, value_parser)]
    execute: Option<PathBuf>,

    /// PATH to dump whole database
    #[clap(short, long, value_parser)]
    dump: Option<PathBuf>,

    /// Storage path to load
    #[clap(short, long, value_parser)]
    path: Option<PathBuf>,
}

pub fn run() -> Result<()> {
    let args = Args::parse();
    let path = args.path.as_deref().and_then(Path::to_str);

    match (path, args.dump) {
        (None, None) => {
            
        }
        (None, Some(_)) => {}
        (Some(_), None) => {}
        (Some(_), Some(_)) => {}
    }

    fn exacute_program(input: Option<PathBuf>) {
        println!("sand-worm-sql");
        let output = std::io::stdout();
        let mut cli = Cli::new(output);

        if let Some(path) = input {
            if let Err(e) = cli.load(path.as_path()) {
                println!("[error] {}\n", e);
            };
        }

        if let Err(e) = cli.run() {
            eprintln!("{}", e);
        }
    }

    Ok(())
}