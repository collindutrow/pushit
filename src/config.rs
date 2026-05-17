use std::fs;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::paths::{self, Tier};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub default_profile: Option<String>,
}

impl Config {
    pub fn load_tier(tier: Tier) -> Result<Self> {
        let path = paths::config_file(tier)?;
        match fs::read_to_string(&path) {
            Ok(s) => Ok(ron::from_str(&s)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn save_tier(&self, tier: Tier) -> Result<()> {
        let path = paths::config_file(tier)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let body = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())?;
        fs::write(&path, body)?;
        Ok(())
    }

    pub fn resolve_default_profile() -> Result<Option<String>> {
        if let Some(name) = Self::load_tier(Tier::User)?.default_profile {
            return Ok(Some(name));
        }
        Ok(Self::load_tier(Tier::System)?.default_profile)
    }
}
