use clap::Parser;
use std::path::PathBuf;

/// Immutable configuration used by the application runtime
#[derive(Debug, Clone)]
pub struct Config {
    pub file: PathBuf,
    pub follow: bool,
    pub regex: Option<String>,
}

/// User-facing CLI arguments (kept private to the CLI layer)
#[derive(Parser, Debug)]
#[command(name = "rtlog", version, about = "Real-time log viewer")]
struct Args {
    /// Path to the log file to read
    #[arg(value_name = "FILE")]
    file: PathBuf,

    /// Follow the file for appended lines (like tail -f)
    #[arg(short = 'f', long = "follow")]
    follow: bool,

    /// Regex filter to highlight matches (case-insensitive)
    #[arg(short = 'r', long = "regex")]
    regex: Option<String>,
}

/// Parse CLI options into an application Config
pub fn parse() -> Config {
    let args = Args::parse();
    Config {
        file: args.file,
        follow: args.follow,
        regex: args.regex,
    }
}
