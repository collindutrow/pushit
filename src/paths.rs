use std::env;
use std::path::PathBuf;

use crate::error::{Error, Result};

const APP_DIR: &str = "pushit";

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tier {
    User,
    System,
}

impl Tier {
    pub fn label(self) -> &'static str {
        match self {
            Tier::User => "user",
            Tier::System => "system",
        }
    }
}

pub fn config_dir(tier: Tier) -> Result<PathBuf> {
    match tier {
        Tier::User => user_config_dir(),
        Tier::System => Ok(system_config_dir()),
    }
}

pub fn profiles_dir(tier: Tier) -> Result<PathBuf> {
    Ok(config_dir(tier)?.join("profiles"))
}

pub fn config_file(tier: Tier) -> Result<PathBuf> {
    Ok(config_dir(tier)?.join("config.ron"))
}

pub fn profile_file(tier: Tier, name: &str) -> Result<PathBuf> {
    validate_profile_name(name)?;
    Ok(profiles_dir(tier)?.join(format!("{name}.ron")))
}

pub fn find_profile_file(name: &str) -> Result<Option<(Tier, PathBuf)>> {
    for tier in [Tier::User, Tier::System] {
        let path = profile_file(tier, name)?;
        if path.exists() {
            return Ok(Some((tier, path)));
        }
    }
    Ok(None)
}

pub fn validate_profile_name(name: &str) -> Result<()> {
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(Error::InvalidProfileName(name.to_string()));
    }
    Ok(())
}

fn user_config_dir() -> Result<PathBuf> {
    let base = if cfg!(target_os = "windows") {
        env::var_os("APPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        env::var_os("HOME").map(|h| PathBuf::from(h).join("Library").join("Application Support"))
    } else {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
    };
    base.map(|b| b.join(APP_DIR)).ok_or(Error::NoConfigHome)
}

fn system_config_dir() -> PathBuf {
    let base: PathBuf = if cfg!(target_os = "windows") {
        env::var_os("PROGRAMDATA")
            .or_else(|| env::var_os("ALLUSERSPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"))
    } else if cfg!(target_os = "macos") {
        PathBuf::from("/Library/Application Support")
    } else {
        PathBuf::from("/etc/xdg")
    };
    base.join(APP_DIR)
}
