mod cli;
mod core;
mod gui;
mod tui;

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cli::Args::parse();

    if args.tui {
        let ipa = args.ipa.unwrap_or_else(|| "app.ipa".to_string());
        tui::app::run_tui(ipa).await?;
    } else {
        gui::run()?;
    }

    Ok(())
}