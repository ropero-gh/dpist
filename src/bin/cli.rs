use anyhow::Result;

use dpist::{
    config, pcap,
    techniques::{self, run_pipeline},
};

fn main() -> Result<()> {
    let config = config::load_config("config.toml")?;

    let packets = pcap::read_pcap(&config.input)?;

    let mut modifiers: Vec<Box<dyn techniques::Modifier>> = config
        .modifiers
        .into_iter()
        .map(techniques::build_modifier)
        .collect();

    let packets = run_pipeline(packets, &mut modifiers);

    pcap::write_pcap(&config.output, &packets)?;

    Ok(())
}
