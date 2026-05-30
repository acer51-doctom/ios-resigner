use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "Cross-platform iOS App Resigner")]
pub struct Args {
    #[arg(short, long)]
    pub tui: bool,

    #[arg(short, long)]
    pub ipa: Option<String>,
}