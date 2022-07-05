use std::io::Read;

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use quick_xml;
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Version {
    epoch: String,
    ver: String,
    rel: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Entry {
    name: String,
    flags: Option<String>,
    epoch: Option<String>,
    ver: Option<String>,
    rel: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Provides {
    #[serde(rename = "entry")]
    entries: Vec<Entry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Requires {
    #[serde(rename = "entry")]
    entries: Vec<Entry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Conflicts {
    #[serde(rename = "entry")]
    entries: Vec<Entry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Obsoletes {
    #[serde(rename = "entry")]
    entries: Vec<Entry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Format {
    provides: Option<Provides>,
    requires: Option<Requires>,
    conflicts: Option<Conflicts>,
    obsoletes: Option<Obsoletes>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Package {
    r#type: String,
    name: String,
    version: Version,
    format: Format,
}

#[derive(Debug, Serialize, Deserialize)]
struct Repo {
    #[serde(rename = "package")]
    packages: Vec<Package>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Repomd {
    #[serde(rename = "data")]
    datas: Vec<Data>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    r#type: String,
    location: Location,
}

#[derive(Debug, Serialize, Deserialize)]
struct Location {
    href: String,
}

impl Repo {
    fn from_url(repo_url: String) -> Result<Repo> {
        // Get repomd.xml from the repo.
        let repomd_url = repo_url.clone() + "repodata/repomd.xml";
        let repomd_xml = reqwest::blocking::get(&repomd_url)
            .with_context(|| format!("Failed to connect to {:?}", &repomd_url))?
            .text()?;
        // Deserialize repomd.xml into a structure using serde.
        let repomd: Repomd =
            quick_xml::de::from_str(&repomd_xml).with_context(|| "Failed to parse repomd.xml")?;
        // Get the url of primary.xml.gz, download and decompress it.
        let mut primary_gz_url = repo_url.clone();
        for data in &repomd.datas {
            if data.r#type == "primary" {
                primary_gz_url = primary_gz_url + &data.location.href;
                break;
            }
        }
        let primary_gz_bytes: Result<Vec<_>, _> = reqwest::blocking::get(&primary_gz_url)
            .with_context(|| format!("Failed to connect to {:?}", &primary_gz_url))?
            .bytes()?
            .bytes()
            .collect();
        let primary_gz_bytes = primary_gz_bytes.unwrap();
        let mut primary_gz = GzDecoder::new(&primary_gz_bytes[..]);
        let mut primary_xml = String::new();
        primary_gz.read_to_string(&mut primary_xml)?;
        quick_xml::de::from_str(&primary_xml).with_context(|| "Failed to parse primary.xml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_primary_xml() -> Result<()> {
        let repo_url = String::from("https://repo.openeuler.org/openEuler-22.03-LTS/OS/x86_64/");
        let repo: Repo = Repo::from_url(repo_url)?;
        println!("{:?}", repo.packages);
        Ok(())
    }
}
