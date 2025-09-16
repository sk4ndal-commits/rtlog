//! rtlog entry point: parses CLI and starts the async application runtime.
//! The main function is intentionally thin and delegates to the runtime in `app`.

mod app;
mod cli;
mod filter;
mod log;
mod state;
mod ui;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config = cli::parse();
    app::run(config).await
}
