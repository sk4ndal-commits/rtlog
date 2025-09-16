use clap::Parser;
use std::path::PathBuf;

/// Immutable configuration used by the application runtime
#[derive(Debug, Clone)]
pub struct Config {
    pub inputs: Vec<PathBuf>,
    pub follow: bool,
    pub regex: Option<String>,
    pub recursive: bool,
}

/// User-facing CLI arguments (kept private to the CLI layer)
#[derive(Parser, Debug)]
#[command(name = "rtlog", version, about = "Real-time log viewer")]
struct Args {
    /// Paths to log files or directories to read
    #[arg(value_name = "PATH", num_args = 1.., required=true)]
    inputs: Vec<PathBuf>,

    /// Follow the files for appended lines (like tail -f)
    #[arg(short = 'f', long = "follow")]
    follow: bool,

    /// Regex filter to highlight matches (case-insensitive)
    #[arg(short = 'r', long = "regex")]
    regex: Option<String>,

    /// Recurse into directories when PATH is a directory
    #[arg(short = 'R', long = "recursive")]
    recursive: bool,
}

/// Parse CLI options into an application Config
pub fn parse() -> Config {
    let args = Args::parse();
    Config {
        inputs: args.inputs,
        follow: args.follow,
        regex: args.regex,
        recursive: args.recursive,
    }
}
