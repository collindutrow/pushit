use std::collections::BTreeSet;
use std::fs;

use crate::cli::{ProfileAddArgs, ProfileCmd, ServiceKind};
use crate::config::Config;
use crate::error::{Error, Result};
use crate::paths::{self, Tier};
use crate::profile::{MessageDefaults, Profile, ServiceConfig};
use crate::service::pushover::PushoverConfig;

pub fn run(cmd: ProfileCmd) -> Result<()> {
    match cmd {
        ProfileCmd::Add(args) => add(args),
        ProfileCmd::List => list(),
        ProfileCmd::Show { name, system } => show(&name, tier_from(system)),
        ProfileCmd::Remove { name, system } => remove(&name, tier_from(system)),
        ProfileCmd::Use { name, system } => use_default(&name, tier_from(system)),
        ProfileCmd::Path { name, system } => path(name.as_deref(), tier_from(system)),
    }
}

fn tier_from(system: bool) -> Tier {
    if system { Tier::System } else { Tier::User }
}

fn add(args: ProfileAddArgs) -> Result<()> {
    paths::validate_profile_name(&args.name)?;
    let tier = tier_from(args.system);

    let service = match args.service {
        ServiceKind::Pushover => ServiceConfig::Pushover(PushoverConfig {
            token: args.token.expect("clap enforces --token for pushover"),
            user_key: args.user_key.expect("clap enforces --user-key for pushover"),
        }),
    };

    let profile = Profile {
        name: args.name.clone(),
        service,
        defaults: MessageDefaults {
            title: args.title,
            priority: args.priority,
            sound: args.sound,
            device: args.device,
            url: args.url,
            url_title: args.url_title,
        },
    };
    profile.save(tier, false)?;
    println!("created profile '{}' ({})", args.name, tier.label());

    if tier == Tier::System
        && paths::profile_file(Tier::User, &args.name)?.exists()
    {
        eprintln!(
            "warning: user profile '{}' shadows this system profile",
            args.name
        );
    }
    Ok(())
}

fn list() -> Result<()> {
    let user_default = Config::load_tier(Tier::User)?.default_profile;
    let system_default = Config::load_tier(Tier::System)?.default_profile;
    let user = Profile::list_tier(Tier::User)?;
    let system = Profile::list_tier(Tier::System)?;
    let user_set: BTreeSet<&str> = user.iter().map(String::as_str).collect();

    println!("user:");
    if user.is_empty() {
        println!("  (none)");
    } else {
        for name in &user {
            let mut tags: Vec<&str> = Vec::new();
            if user_default.as_deref() == Some(name) {
                tags.push("default");
            }
            print_entry(name, &tags);
        }
    }

    println!("system:");
    if system.is_empty() {
        println!("  (none)");
    } else {
        for name in &system {
            let mut tags: Vec<&str> = Vec::new();
            if system_default.as_deref() == Some(name) {
                tags.push("system default");
            }
            if user_set.contains(name.as_str()) {
                tags.push("shadowed");
            }
            print_entry(name, &tags);
        }
    }
    Ok(())
}

fn print_entry(name: &str, tags: &[&str]) {
    if tags.is_empty() {
        println!("  {name}");
    } else {
        println!("  {name} ({})", tags.join(", "));
    }
}

fn show(name: &str, tier: Tier) -> Result<()> {
    let path = match tier {
        Tier::System => paths::profile_file(Tier::System, name)?,
        Tier::User => {
            let (_, p) = paths::find_profile_file(name)?
                .ok_or_else(|| Error::ProfileNotFound(name.to_string()))?;
            p
        }
    };
    let body = fs::read_to_string(&path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => Error::ProfileNotFound(name.to_string()),
        _ => Error::Io(e),
    })?;
    print!("{body}");
    if !body.ends_with('\n') {
        println!();
    }
    Ok(())
}

fn remove(name: &str, tier: Tier) -> Result<()> {
    Profile::remove(tier, name)?;
    let mut config = Config::load_tier(tier)?;
    if config.default_profile.as_deref() == Some(name) {
        config.default_profile = None;
        config.save_tier(tier)?;
    }
    println!("removed profile '{name}' ({})", tier.label());

    if tier == Tier::User
        && paths::profile_file(Tier::System, name)?.exists()
    {
        println!("system profile '{name}' is now active");
    }
    Ok(())
}

fn use_default(name: &str, tier: Tier) -> Result<()> {
    let _ = Profile::load(name)?;
    let mut config = Config::load_tier(tier)?;
    config.default_profile = Some(name.to_string());
    config.save_tier(tier)?;
    println!("{} default profile is now '{name}'", tier.label());
    Ok(())
}

fn path(name: Option<&str>, tier: Tier) -> Result<()> {
    let p = match name {
        Some(n) => Profile::path_for(tier, n)?,
        None => paths::profiles_dir(tier)?,
    };
    println!("{}", p.display());
    Ok(())
}
