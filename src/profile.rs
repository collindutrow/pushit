use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::paths::{self, Tier};
use crate::service::pushover::PushoverConfig;
use crate::service::{Message, Service};

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub service: ServiceConfig,
    #[serde(default)]
    pub defaults: MessageDefaults,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServiceConfig {
    Pushover(PushoverConfig),
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MessageDefaults {
    pub title: Option<String>,
    pub priority: Option<i8>,
    pub sound: Option<String>,
    pub device: Option<String>,
    pub url: Option<String>,
    pub url_title: Option<String>,
}

impl Profile {
    pub fn load(name: &str) -> Result<(Tier, Self)> {
        let (tier, path) =
            paths::find_profile_file(name)?.ok_or_else(|| Error::ProfileNotFound(name.to_string()))?;
        let body = fs::read_to_string(&path)?;
        let profile: Profile = ron::from_str(&body)?;
        Ok((tier, profile))
    }

    pub fn save(&self, tier: Tier, overwrite: bool) -> Result<()> {
        let path = paths::profile_file(tier, &self.name)?;
        if !overwrite && path.exists() {
            return Err(Error::ProfileExists(self.name.clone()));
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let body = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())?;
        fs::write(&path, body)?;
        Ok(())
    }

    pub fn remove(tier: Tier, name: &str) -> Result<()> {
        let path = paths::profile_file(tier, name)?;
        fs::remove_file(&path).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => Error::ProfileNotFound(name.to_string()),
            _ => Error::Io(e),
        })
    }

    pub fn list_tier(tier: Tier) -> Result<Vec<String>> {
        let dir = paths::profiles_dir(tier)?;
        let mut names = Vec::new();
        let entries = match fs::read_dir(&dir) {
            Ok(it) => it,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(names),
            Err(e) => return Err(e.into()),
        };
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("ron")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            {
                names.push(stem.to_string());
            }
        }
        names.sort();
        Ok(names)
    }

    pub fn path_for(tier: Tier, name: &str) -> Result<PathBuf> {
        paths::profile_file(tier, name)
    }

    pub fn build_service(&self) -> Box<dyn Service> {
        match &self.service {
            ServiceConfig::Pushover(cfg) => Box::new(cfg.clone()),
        }
    }

    pub fn message_with_overrides(&self, body: String, overrides: MessageDefaults) -> Message {
        Message {
            body,
            title: overrides.title.or_else(|| self.defaults.title.clone()),
            priority: overrides.priority.or(self.defaults.priority),
            sound: overrides.sound.or_else(|| self.defaults.sound.clone()),
            device: overrides.device.or_else(|| self.defaults.device.clone()),
            url: overrides.url.or_else(|| self.defaults.url.clone()),
            url_title: overrides.url_title.or_else(|| self.defaults.url_title.clone()),
        }
    }
}
