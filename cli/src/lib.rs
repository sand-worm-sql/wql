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
        path::{Path, PathBuf},
    },
};

#[derive(Parser, Debug)]
#[clap(name = "sand-worm-cli", about = "Sand Worm SQL CLI", version = "1.0")]
struct Args {
    /// SQL file to execute
    #[arg(short, long)]
    execute: Option<PathBuf>,

    /// PATH to dump whole database
    #[arg(short, long)]
    dump: Option<PathBuf>,

    /// Storage path to load
    #[arg(short, long)]
    path: Option<PathBuf>,

}

pub fn run() -> Result<()> {
    execute_program(None);
    Ok(())
}

fn execute_program(input: Option<PathBuf>) {
    println!("sand-worm-sql");
    let output = std::io::stdout();
    let mut cli = Cli::new(output);

    if let Some(path) = input {
        if let Err(e) = cli.load(path.as_path()) {
            eprintln!("[error] {}\n", e);
        }
    }

    if let Err(e) = cli.run() {
        eprintln!("{}", e);
    }
}

pub fn dump_data(dump_path: PathBuf) -> Result<()> {
    let _file = File::create(&dump_path)?;
    println!("sand-worm-sql {}", dump_path.display());
    Ok(())
}
