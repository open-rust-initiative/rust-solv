use crate::repomd::Repomd;
use crate::yum::YumVariables;
use anyhow::{Context, Result};
use configparser;
use futures;
use quick_xml;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio_stream::StreamExt;
use walkdir::DirEntry;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
struct Version {
    epoch: u64,
    ver: String,
    rel: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RpmEntry {
    name: String,
    flags: Option<String>,
    epoch: Option<u64>,
    ver: Option<String>,
    rel: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Entries {
    #[serde(rename = "entry")]
    entries: Vec<RpmEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Format {
    provides: Option<Entries>,
    requires: Option<Entries>,
    conflicts: Option<Entries>,
    obsoletes: Option<Entries>,
}

type IdT = usize;

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
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
    #[serde(skip)]
    providers: HashMap<String, IdT>,
}

impl Repo {
    fn from_baseurl(repo_url: String) -> Result<Repo> {
        let primary_xml = Repomd::get_primary_xml(repo_url)?;
        let mut repo: Repo =
            quick_xml::de::from_str(&primary_xml).with_context(|| "Failed to parse primary.xml")?;
        let mut index: IdT = 0;
        for package in &repo.packages {
            if let Some(ref provides) = package.format.provides {
                for entry in &provides.entries {
                    repo.providers.insert(entry.name.clone(), index);
                }
            }
            index += 1;
        }
        Ok(repo)
    }

    async fn from_dir_entry(entry: DirEntry) -> Result<Vec<Repo>> {
        let path = entry.path();
        Repo::from_file(path).await
    }

    // Read the .repo config file at path,
    // then return a vector of repos in the file.
    async fn from_file(path: &Path) -> Result<Vec<Repo>> {
        let mut repos: Vec<Repo> = Vec::new();
        // Parse .repo config file into a map.
        let mut config = configparser::ini::Ini::new_cs();
        let map = config.load(path.to_str().unwrap()).unwrap();
        // Iterate each repo.
        let mut stream = tokio_stream::iter(map);
        while let Some((_, kvs)) = stream.next().await {
            let mut repo_name = String::new();
            let mut repo_baseurl = String::new();
            for (key, value) in kvs {
                match key.trim() {
                    "name" => {
                        repo_name = value.unwrap_or(String::new());
                    }
                    "baseurl" => {
                        repo_baseurl = match value {
                            Some(url) => {
                                if url.ends_with('/') {
                                    url
                                } else {
                                    url + "/"
                                }
                            }
                            None => String::new(),
                        }
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

    async fn from_dir(path: &Path) -> Result<Vec<Repo>> {
        let dirs: Vec<_> = WalkDir::new(path)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .collect();
        let futures: Vec<_> = dirs
            .into_iter()
            .map(|entry| Repo::from_dir_entry(entry))
            .collect();
        let results = futures::future::join_all(futures).await;
        let mut repos: Vec<Repo> = Vec::new();
        for result in results {
            if let Ok(mut repo) = result {
                repos.append(&mut repo);
            } else {
                continue;
            }
        }
        Ok(repos)
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;

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
        let repo = block_on(Repo::from_file(&path))?;
        println!("{:?}", repo);
        Ok(())
    }

    #[test]
    fn test_from_dir() -> Result<()> {
        let path = Path::new("/etc/yum.repos.d/");
        let repos = block_on(Repo::from_dir(&path))?;
        for repo in repos {
            println!("{:?}", repo.name);
        }
        Ok(())
    }
}
