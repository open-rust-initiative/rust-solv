use anyhow::{anyhow, Context, Result};
use configparser;
use std::process::Command;

pub struct YumVariables {
    arch: String,
    basearch: String,
    releasever: String,
}

impl YumVariables {
    // $arch refers to the system's CPU architecture.
    fn get_arch() -> Result<String> {
        let arch = String::from_utf8(Command::new("arch").output()?.stdout)
            .with_context(|| "Failed to get $arch")?;
        let arch = arch.trim();
        Ok(arch.to_string())
    }
    // $basearch refers to the base architecture of the system.
    // For example, i686 machines have a base architecture of i386,
    // and AMD64 and Intel 64 machines have a base architecture of x86_64.
    fn get_basearch() -> Result<String> {
        let arch = YumVariables::get_arch()?;
        match arch.as_str() {
            "i386" | "i586" | "i686" => Ok("i386".to_string()),
            "x86_64" => Ok("x86_64".to_string()),
            _ => Err(anyhow!("")),
        }
    }
    // $releasever refers to the release version of the system.
    // Yum obtains the value of $releasever from the distroverpkg=value line in the /etc/yum.conf configuration file.
    // If there is no such line in /etc/yum.conf,
    // then yum infers the correct value by deriving the version number from the system-release package.
    fn get_releasever() -> Result<String> {
        // First find distroverpkg=value line in /etc/yum.conf.
        let mut config = configparser::ini::Ini::new_cs();
        let map = config.load("/etc/yum.conf").unwrap();
        let mut releasever = String::new();
        for (_, kvs) in map {
            if releasever != "" {
                break;
            }
            for (key, value) in kvs {
                if key == "distroverpkg" {
                    releasever = value.unwrap_or(String::new());
                    break;
                } else {
                    continue;
                }
            }
        }
        // If there is no distroverpkg=value in /etc/yum.conf,
        // Get the $releasever by deriving the version number from the system-release package.
        if releasever == "" {
            let release = String::from_utf8(
                Command::new("rpm")
                    .args(["-q", "openEuler-release"])
                    .output()?
                    .stdout,
            )
            .with_context(|| "System-release package not found")?;
            // The variable "release" is a string like "system-release-version-...".
            // So we split the string by "-", then get the element with index 2.
            let release: Vec<&str> = release.split("-").collect();
            releasever = String::from(release[2]);
        }
        Ok(releasever)
    }

    fn get_yum_variables() -> Result<YumVariables> {
        Ok(YumVariables {
            arch: YumVariables::get_arch()?,
            basearch: YumVariables::get_basearch()?,
            releasever: YumVariables::get_releasever()?,
        })
    }

    pub fn replace_yum_variables(s: String) -> Result<String> {
        let yum_var = YumVariables::get_yum_variables()?;
        let mut ret = s;
        if ret.contains("$arch") {
            ret = ret.replace("$arch", &yum_var.arch);
        }
        if ret.contains("$basearch") {
            ret = ret.replace("$basearch", &yum_var.basearch);
        }
        if ret.contains("$releasever") {
            ret = ret.replace("$releasever", &yum_var.releasever);
        }
        Ok(ret)
    }
}
