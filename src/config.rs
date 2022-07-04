use anyhow::{self, Context, Result};
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;
use toml;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    format: String,
    repodir: String,
}

impl Config {
    fn from_str(s: &str) -> Result<Config> {
        toml::from_str(s).with_context(|| "Failed to parse the config")
    }

    fn from_file(path: &Path) -> Result<Config> {
        let s = fs::read_to_string(path)
            .with_context(|| format!("Failed to read the config file {:?}", path))?;
        Config::from_str(&s)
    }
}