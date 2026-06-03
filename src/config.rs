use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::techniques::ModifierConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub input: PathBuf,
    pub output: PathBuf,
    pub modifiers: Vec<ModifierConfig>,
}

#[derive(Debug, Clone)]
pub struct ModifierUi {
    pub enabled: bool,
    pub config: ModifierConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            input: PathBuf::from("input.pcap"),
            output: PathBuf::from("output.pcap"),
            modifiers: Vec::new(),
        }
    }
}

pub fn validate(config: &Config) -> Result<(), String> {
    if config.input.as_os_str().is_empty() {
        return Err("Input path cannot be empty".into());
    }

    if config.output.as_os_str().is_empty() {
        return Err("Output path cannot be empty".into());
    }

    Ok(())
}

pub fn load_config(path: impl AsRef<Path>) -> Result<Config> {
    let path = path.as_ref();

    if !path.exists() {
        return Ok(Config::default());
    }

    let contents = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&contents)?;

    Ok(config)
}

pub fn save_config(path: impl AsRef<Path>, config: &Config) -> Result<()> {
    let contents = toml::to_string_pretty(config)?;
    fs::write(path, contents)?;
    Ok(())
}
