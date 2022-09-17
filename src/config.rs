use anyhow::{self, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use toml;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    repoinfo: Repoinfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct Repoinfo {
    name: Option<String>,
    baseurl: Option<String>,
}

impl Config {
    fn from_str(s: &str) -> Result<Config> {
        toml::from_str(s).with_context(|| "failed to parse the config file.")
    }

    pub fn from_file(path: &Path) -> Result<Config> {
        let s = fs::read_to_string(path)
            .with_context(|| format!("failed to open the config file {:?}.", path))?;
        Config::from_str(&s)
    }

    pub fn get_repo_name(&self) -> &Option<String> {
        &self.repoinfo.name
    }

    pub fn get_repo_baseurl(&self) -> &Option<String> {
        &self.repoinfo.baseurl
    }
}
