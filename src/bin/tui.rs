use anyhow::Result;

use dpist::tui;

fn main() -> Result<()> {
    tui::run("config.toml")
}
