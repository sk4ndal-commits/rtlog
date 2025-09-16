mod app;
mod cli;
mod filter;
mod log;
mod state;
mod ui;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // main is now a thin entrypoint delegating to dedicated modules
    let config = cli::parse();
    app::run(config).await
}
