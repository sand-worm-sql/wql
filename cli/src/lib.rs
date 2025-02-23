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
    let args = Args::parse();

    // println!("{}", args.execute.unwrap_or("".to_string()));

   // println!("sand-worm-sql {}", path.unwrap_or(""));

    // match (path, args.dump) {
    //     (None, None) => {
    //         eprintln!("Error: No path or dump specified.");
    //         return Ok(()); // Exit early
    //     }
    //     (None, Some(dump_path)) => {
    //         dump_data(dump_path)?; 
    //     }
    //     (Some(_), None) => {
    //         execute_program(args.execute); 
    //     }
    //     (Some(_), Some(_)) => {
    //         eprintln!("Error: Cannot specify both a path and a dump at the same time.");
    //         return Ok(()); // Exit early
    //     }
    // }

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
