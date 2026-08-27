mod optimization_pipeline;

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use clap::Parser;
use once_cell::sync::Lazy;

#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;

pub static EPOCH: Lazy<Instant> = Lazy::new(Instant::now);

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Parser)]
#[command(
    name = "gdrr-2bp",
    version,
    about = "Solve two-dimensional bin packing problems with guillotine constraints"
)]
struct Cli {
    /// Input problem JSON file.
    #[arg(value_name = "INPUT")]
    input: PathBuf,

    /// Solver configuration JSON file.
    #[arg(short, long, value_name = "CONFIG")]
    config: PathBuf,

    /// Write JSON and HTML solutions to this directory.
    #[arg(short, long, value_name = "DIR")]
    output: Option<PathBuf>,
}

fn main() -> ExitCode {
    let args = Cli::parse();
    match optimization_pipeline::run(args.input, args.config, args.output) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
