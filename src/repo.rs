use crate::repomd::Repomd;
use crate::yum::YumVariables;
use anyhow::{Context, Result};
use quick_xml;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct Version {
    epoch: u64,
    ver: String,
    rel: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RpmEntry {
    name: String,
    flags: Option<String>,
    epoch: Option<u64>,
    ver: Option<String>,
    rel: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entries {
    #[serde(rename = "entry")]
    entries: Vec<RpmEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Format {
    provides: Option<Entries>,
    requires: Option<Entries>,
    conflicts: Option<Entries>,
    obsoletes: Option<Entries>,
}

pub type IdT = usize;

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    name: String,
    version: Version,
    format: Format,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Repo {
    #[serde(rename = "package")]
    packages: Vec<Package>,
    #[serde(skip)]
    providers: HashMap<String, Vec<IdT>>,
}

impl Repo {
    pub fn from_baseurl(repo_baseurl: &str) -> Result<Repo> {
        let repo_baseurl = if repo_baseurl.ends_with('/') {
            repo_baseurl.to_string()
        } else {
            repo_baseurl.to_string() + "/"
        };
        let yum_variables = YumVariables::new()?;
        let repo_baseurl = yum_variables.replace_yum_variables(repo_baseurl)?;
        let primary_xml = Repomd::get_primary_xml(repo_baseurl)?;
        let mut repo: Repo =
            quick_xml::de::from_str(&primary_xml).with_context(|| "Failed to parse primary.xml")?;
        for (index, package) in repo.packages.iter().enumerate() {
            if let Some(ref provides) = package.format.provides {
                for entry in &provides.entries {
                    if let Some(ids) = repo.providers.get_mut(&entry.name) {
                        ids.push(index);
                    } else {
                        repo.providers.insert(entry.name.clone(), vec![index]);
                    }
                }
            }
        }
        Ok(repo)
    }

    pub fn get_package_id_by_name(&self, name: &str) -> Option<IdT> {
        for (id, package) in self.packages.iter().enumerate() {
            if package.name == name {
                return Some(id);
            }
        }
        None
    }

    pub fn get_package_requires_by_id<'a>(&'a self, package_id: IdT) -> Option<&'a Vec<RpmEntry>> {
        if let Some(package) = self.packages.get(package_id) {
            if let Some(ref e) = package.format.requires {
                return Some(&e.entries);
            }
        }
        None
    }

    pub fn get_package_conflicts_by_id<'a>(&'a self, package_id: IdT) -> Option<&'a Vec<RpmEntry>> {
        if let Some(package) = self.packages.get(package_id) {
            if let Some(ref e) = package.format.conflicts {
                return Some(&e.entries);
            }
        }
        None
    }

    pub fn get_package_obsoletes_by_id<'a>(&'a self, package_id: IdT) -> Option<&'a Vec<RpmEntry>> {
        if let Some(package) = self.packages.get(package_id) {
            if let Some(ref e) = package.format.obsoletes {
                return Some(&e.entries);
            }
        }
        None
    }

    pub fn get_entry_provider_id(&self, entry: &RpmEntry) -> Option<&Vec<IdT>> {
        self.providers.get(&entry.name)
    }
}

impl Package {
    pub fn requires(self) -> Option<Vec<RpmEntry>> {
        if let Some(e) = self.format.requires {
            Some(e.entries)
        } else {
            None
        }
    }

    pub fn conflicts(self) -> Option<Vec<RpmEntry>> {
        if let Some(e) = self.format.conflicts {
            Some(e.entries)
        } else {
            None
        }
    }

    pub fn obsoletes(self) -> Option<Vec<RpmEntry>> {
        if let Some(e) = self.format.obsoletes {
            Some(e.entries)
        } else {
            None
        }
    }

    pub fn provides(self) -> Option<Vec<RpmEntry>> {
        if let Some(e) = self.format.provides {
            Some(e.entries)
        } else {
            None
        }
    }
}

impl RpmEntry {
    pub fn get_name(self) -> String {
        self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_primary_xml() -> Result<()> {
        let repo_url = String::from("https://repo.openeuler.org/openEuler-22.03-LTS/OS/x86_64/");
        let repo: Repo = Repo::from_baseurl(&repo_url)?;
        println!("{:?}", repo.packages);
        Ok(())
    }
}
