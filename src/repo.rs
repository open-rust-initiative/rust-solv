use crate::repomd::Repomd;
use crate::version::{version_compare, Flag};
use crate::yum::YumVariables;
use anyhow::{anyhow, Context, Result};
use quick_xml;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct Version {
    epoch: i32,
    ver: String,
    rel: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RpmEntry {
    pub name: String,
    pub flags: Option<String>,
    pub epoch: Option<i32>,
    pub ver: Option<String>,
    pub rel: Option<String>,
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

    fn get_entry_by_provider_id(&self, provider_id: IdT, entry_name: &str) -> Option<&RpmEntry> {
        for entry in &self.packages[provider_id]
            .format
            .provides
            .as_ref()
            .unwrap()
            .entries
        {
            if entry.name == entry_name {
                return Some(entry);
            }
        }
        None
    }

    // Package requires entry x, and it is provided by another package as entry y.
    pub fn check_version_constraint(
        &self,
        entry_required: &RpmEntry,
        provider_id: &IdT,
    ) -> Result<bool> {
        if let Some(flags) = &entry_required.flags {
            let entry_provided = self
                .get_entry_by_provider_id(*provider_id, &entry_required.name)
                .unwrap();
            match flags.as_str() {
                "LT" => match &entry_provided.flags {
                    Some(flags) => match flags.as_str() {
                        "GE" | "EQ" | "GT" => {
                            version_compare(entry_required, entry_provided, Flag::GT)
                        }
                        _ => Ok(true),
                    },
                    _ => Ok(true),
                },
                "LE" => match &entry_provided.flags {
                    Some(flags) => match flags.as_str() {
                        "GE" | "EQ" => {
                            version_compare(entry_required, entry_provided, Flag::GE)
                        },
                        "GT" => {
                            version_compare(entry_required, entry_provided, Flag::GT)
                        },
                        _ => Ok(true),
                    },
                    _ => Ok(true),
                },
                "EQ" => match &entry_provided.flags {
                    Some(flags) => match flags.as_str() {
                        "LE" => {
                            version_compare(entry_required, entry_provided, Flag::GE)
                        },
                        "LT" => {
                            version_compare(entry_required, entry_provided, Flag::GT)
                        },
                        "EQ" => {
                            version_compare(entry_required, entry_provided, Flag::EQ)
                        },
                        "GT" => {
                            version_compare(entry_required, entry_provided, Flag::LT)
                        },
                        "GE" => {
                            version_compare(entry_required, entry_provided, Flag::LE)
                        }
                        _ => Ok(true),
                    },
                    _ => Ok(true),
                },
                "GE" => match &entry_provided.flags {
                    Some(flags) => match flags.as_str() {
                        "LE" | "EQ" => {
                            version_compare(entry_required, entry_provided, Flag::LE)
                        },
                        "LT" => {
                            version_compare(entry_required, entry_provided, Flag::LT)
                        },
                        _ => Ok(true),
                    },
                    _ => Ok(true),
                },
                "GT" => match &entry_provided.flags {
                    Some(flags) => match flags.as_str() {
                        "LE" | "EQ" | "LT" => {
                            version_compare(entry_required, entry_provided, Flag::LT)
                        }
                        _ => Ok(true),
                    },
                    _ => Ok(true),
                },
                _ => Err(anyhow!("invalid flags of entry")),
            }
        } else {
            Ok(true)
        }
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
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_epoch(&self) -> Option<i32> {
        self.epoch
    }

    pub fn get_ver(&self) -> Option<&String> {
        self.ver.as_ref()
    }

    pub fn get_rel(&self) -> Option<&String> {
        self.rel.as_ref()
    }

    pub fn get_flags(&self) -> Option<&String> {
        self.flags.as_ref()
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
