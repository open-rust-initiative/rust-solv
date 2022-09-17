use anyhow::{anyhow, Context, Result};
use configparser;
use indexmap::IndexMap;
use std::process::Command;

#[derive(Debug)]
pub struct YumVariables {
    arch: String,
    basearch: String,
    releasever: String,
}

impl YumVariables {
    // $arch refers to the system's CPU architecture.
    fn get_arch() -> Result<String> {
        let arch = String::from_utf8(Command::new("arch").output()?.stdout)
            .with_context(|| "Error: failed to get $arch")?;
        Ok(arch.trim().to_string())
    }

    // $basearch refers to the base architecture of the system.
    // For example, i686 machines have a base architecture of i386,
    // and AMD64 and Intel 64 machines have a base architecture of x86_64.
    fn get_basearch() -> Result<String> {
        let arch = YumVariables::get_arch()?;
        match arch.as_str() {
            "i386" | "i586" | "i686" => Ok("i386".to_string()),
            "x86_64" => Ok("x86_64".to_string()),
            "aarch64" => Ok("aarch64".to_string()),
            _ => Err(anyhow!("Error: unknown basearch.")),
        }
    }

    // $releasever refers to the release version of the system.
    // Yum obtains the value of $releasever from the distroverpkg=value line in the /etc/yum.conf configuration file.
    // If there is no such line in /etc/yum.conf,
    // then yum infers the correct value by deriving the version number from the system-release package.
    fn get_releasever() -> Result<String> {
        // First find distroverpkg=value line in /etc/yum.conf.
        let mut config_loader = configparser::ini::Ini::new_cs();
        // Create a vector which contains maps with key "distroverpkg".
        let maps_with_distroverpkg: Vec<IndexMap<String, Option<String>>> = config_loader
            .load("/etc/yum.conf")
            .unwrap()
            .into_iter()
            .map(|(_, kvs)| kvs)
            .filter(|kvs| kvs.contains_key("distroverpkg"))
            .collect();
        match maps_with_distroverpkg.get(0) {
            Some(kvs) => Ok(kvs["distroverpkg"].to_owned().unwrap()),
            None => {
                let release = String::from_utf8(
                    Command::new("rpm")
                        .args(["-q", "openEuler-release"])
                        .output()?
                        .stdout,
                )
                .with_context(|| "Error: system-release package not found.")?;
                // The variable "release" is a string like "system-release-version-...".
                // So we split the string by "-", then get the element with index 2.
                let release: Vec<&str> = release.split("-").collect();
                Ok(release[2].to_string())
            }
        }
    }

    pub fn new() -> Result<YumVariables> {
        Ok(YumVariables {
            arch: YumVariables::get_arch()?,
            basearch: YumVariables::get_basearch()?,
            releasever: YumVariables::get_releasever()?,
        })
    }

    pub fn replace_yum_variables(&self, s: String) -> Result<String> {
        let mut ret = s;
        if ret.contains("$arch") {
            ret = ret.replace("$arch", &self.arch);
        }
        if ret.contains("$basearch") {
            ret = ret.replace("$basearch", &self.basearch);
        }
        if ret.contains("$releasever") {
            ret = ret.replace("$releasever", &self.releasever);
        }
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yum_variables() -> Result<()> {
        let yum_var = YumVariables::new()?;
        println!("{:?}", yum_var);
        Ok(())
    }
}
