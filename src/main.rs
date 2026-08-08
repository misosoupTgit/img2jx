use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use img2jx::{decode_image, encode_image, Backend};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum BackendArg {
    Cpu,
    Gpu,
}

impl From<BackendArg> for Backend {
    fn from(value: BackendArg) -> Self {
        match value {
            BackendArg::Cpu => Backend::Cpu,
            BackendArg::Gpu => Backend::Gpu,
        }
    }
}

#[derive(Parser)]
#[command(name = "img2jx", version, about = "Image ↔ JSON converter")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert image to JSON
    Encode {
        /// Input image path
        input: PathBuf,
        /// Output JSON path
        output: PathBuf,
        /// Pretty-print JSON output
        #[arg(long)]
        pretty: bool,
        /// Processing backend
        #[arg(long, value_enum, default_value = "cpu")]
        backend: BackendArg,
        /// Number of CPU threads (default: logical cores)
        #[arg(long)]
        threads: Option<usize>,
    },
    /// Convert JSON to image
    Decode {
        /// Input JSON path
        input: PathBuf,
        /// Output image path (format from extension)
        output: PathBuf,
        /// Processing backend
        #[arg(long, value_enum, default_value = "cpu")]
        backend: BackendArg,
        /// Number of CPU threads (default: logical cores)
        #[arg(long)]
        threads: Option<usize>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Encode {
            input,
            output,
            pretty,
            backend,
            threads,
        } => {
            if let Err(e) = img2jx::parallel::configure_threads(threads) {
                eprintln!("error: {e:#}");
                return ExitCode::from(1);
            }
            encode_image(&input, &output, pretty, backend.into())
        }
        Commands::Decode {
            input,
            output,
            backend,
            threads,
        } => {
            if let Err(e) = img2jx::parallel::configure_threads(threads) {
                eprintln!("error: {e:#}");
                return ExitCode::from(1);
            }
            decode_image(&input, &output, backend.into())
        }
    };

    if let Err(e) = result {
        eprintln!("error: {e:#}");
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
