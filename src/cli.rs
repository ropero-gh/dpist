use std::path::PathBuf;

#[derive(clap::Parser)]
pub struct Cli {
    #[arg(short, long)]
    pub config: PathBuf,
}
