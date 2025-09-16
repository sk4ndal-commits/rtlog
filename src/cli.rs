use clap::Parser;
use std::path::PathBuf;

/// Immutable configuration used by the application runtime
#[derive(Debug, Clone)]
pub struct Config {
    pub inputs: Vec<PathBuf>,
    pub follow: bool,
    pub regex: Option<String>,
    pub recursive: bool,
    pub alerts: Vec<String>,
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

    /// Patterns that trigger visual alerts (repeatable). Defaults to ERROR and FATAL if none provided.
    #[arg(long = "alert")]
    alerts: Vec<String>,

    /// Disable alerts entirely (no red highlights, no banner)
    #[arg(long = "no-alerts", alias = "no-alert")]
    no_alerts: bool,
}

/// Parse CLI options into an application Config
pub fn parse() -> Config {
    let args = Args::parse();
    let alerts = if args.no_alerts {
        Vec::new()
    } else if args.alerts.is_empty() {
        vec!["ERROR".into(), "FATAL".into()]
    } else {
        args.alerts
    };
    Config {
        inputs: args.inputs,
        follow: args.follow,
        regex: args.regex,
        recursive: args.recursive,
        alerts,
    }
}
