use crate::cli::SendArgs;
use crate::config::Config;
use crate::error::{Error, Result};
use crate::profile::{MessageDefaults, Profile};

pub fn run(args: SendArgs) -> Result<()> {
    let profile_name = match args.profile {
        Some(name) => name,
        None => Config::resolve_default_profile()?.ok_or(Error::NoDefaultProfile)?,
    };

    let (_tier, profile) = Profile::load(&profile_name)?;

    let overrides = MessageDefaults {
        title: args.title,
        priority: args.priority,
        sound: args.sound,
        device: args.device,
        url: args.url,
        url_title: args.url_title,
    };
    let message = profile.message_with_overrides(args.message, overrides);

    profile.build_service().send(&message)
}
