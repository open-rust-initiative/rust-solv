use std::io::Read;
use std::path::Path;

use crate::yum::YumVariables;
use anyhow::{Context, Result};
use configparser;
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
    #[serde(skip)]
    name: String,
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
    fn from_baseurl(repo_url: String) -> Result<Repo> {
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

    // Read the .repo config file at path,
    // then return a vector of repos in the file.
    fn from_file(path: &Path) -> Result<Vec<Repo>> {
        let mut repos: Vec<Repo> = Vec::new();
        // Parse .repo config file into a map.
        let mut config = configparser::ini::Ini::new_cs();
        let map = config.load(path.to_str().unwrap()).unwrap();
        // Iterate each repo.
        for (_, kvs) in map {
            let mut repo_name = String::new();
            let mut repo_baseurl = String::new();
            for (key, value) in kvs {
                match key.trim() {
                    "name" => {
                        repo_name = value.unwrap();
                    }
                    "baseurl" => {
                        repo_baseurl = value.unwrap();
                    }
                    "mirrorlist" => {
                        // To be done...
                    }
                    _ => (),
                }
            }
            // Replace yum variables.
            repo_name = YumVariables::replace_yum_variables(repo_name)?;
            repo_baseurl = YumVariables::replace_yum_variables(repo_baseurl)?;
            let mut repo = Repo::from_baseurl(repo_baseurl)?;
            repo.name = repo_name;
            repos.push(repo);
        }
        Ok(repos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_primary_xml() -> Result<()> {
        let repo_url = String::from("https://repo.openeuler.org/openEuler-22.03-LTS/OS/x86_64/");
        let repo: Repo = Repo::from_baseurl(repo_url)?;
        println!("{:?}", repo.packages);
        Ok(())
    }

    #[test]
    fn test_from_file() -> Result<()> {
        let path = Path::new("/etc/yum.repos.d/openEuler.repo");
        let repo = Repo::from_file(&path)?;
        println!("{:?}", repo);
        Ok(())
    }
}
