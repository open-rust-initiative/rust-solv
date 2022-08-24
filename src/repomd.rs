use std::io::Read;

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use serde::{self, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Repomd {
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

impl Repomd {
    pub fn get_primary_xml(repo_url: String) -> Result<String> {
        // Get repomd.xml from the repo.
        let repomd_url = repo_url.clone() + "repodata/repomd.xml";
        let repomd_xml = reqwest::blocking::get(&repomd_url)
            .with_context(|| format!("Failed to connect to {:?}", &repomd_url))?
            .text()?;
        // Deserialize repomd.xml into a structure using serde.
        let repomd: Repomd =
            quick_xml::de::from_str(&repomd_xml).with_context(|| "Failed to parse repomd.xml")?;
        // Get the url of primary.xml.gz, download and decompress it.
        let primary_data: Vec<Data> = repomd.datas.into_iter().filter(|data| data.r#type == "primary").collect();
        let primary_gz_url = repo_url.clone() + &primary_data[0].location.href;
        let primary_gz_bytes: Result<Vec<_>, _> = reqwest::blocking::get(&primary_gz_url)
            .with_context(|| format!("Failed to connect to {:?}", &primary_gz_url))?
            .bytes()?
            .bytes()
            .collect();
        let primary_gz_bytes = primary_gz_bytes.unwrap();
        let mut primary_gz = GzDecoder::new(&primary_gz_bytes[..]);
        let mut primary_xml = String::new();
        primary_gz.read_to_string(&mut primary_xml)?;
        Ok(primary_xml)
    }
}
