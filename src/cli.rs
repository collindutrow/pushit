use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "pushit", version, about = "Send push notifications from the CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Send a notification.
    Send(SendArgs),
    /// Manage profiles.
    Profile {
        #[command(subcommand)]
        command: ProfileCmd,
    },
}

#[derive(Args, Debug)]
pub struct SendArgs {
    /// Profile to send through. Defaults to the configured default profile.
    #[arg(short, long)]
    pub profile: Option<String>,

    #[arg(short, long)]
    pub title: Option<String>,

    #[arg(short = 'P', long)]
    pub priority: Option<i8>,

    #[arg(short, long)]
    pub sound: Option<String>,

    #[arg(short, long)]
    pub device: Option<String>,

    #[arg(short, long)]
    pub url: Option<String>,

    #[arg(long = "url-title")]
    pub url_title: Option<String>,

    /// The notification body.
    pub message: String,
}

#[derive(Subcommand, Debug)]
pub enum ProfileCmd {
    /// Create a new profile.
    Add(ProfileAddArgs),
    /// List all profiles from both tiers. The default profile is marked.
    List,
    /// Print a profile's .ron file contents.
    Show {
        name: String,
        /// Force the system-tier copy (default: user tier wins).
        #[arg(long)]
        system: bool,
    },
    /// Delete a profile.
    Remove {
        name: String,
        /// Remove from the system tier instead of the user tier.
        #[arg(long)]
        system: bool,
    },
    /// Set a profile as the default.
    Use {
        name: String,
        /// Write the system-tier default instead of the user default.
        #[arg(long)]
        system: bool,
    },
    /// Print the on-disk path of a profile (or the profiles dir if no name).
    Path {
        name: Option<String>,
        /// Show the system-tier path instead of the user path.
        #[arg(long)]
        system: bool,
    },
}

#[derive(Args, Debug)]
pub struct ProfileAddArgs {
    pub name: String,

    #[arg(long, value_enum)]
    pub service: ServiceKind,

    /// Pushover application API token.
    #[arg(long, required_if_eq("service", "pushover"))]
    pub token: Option<String>,

    /// Pushover user or group key.
    #[arg(long, required_if_eq("service", "pushover"))]
    pub user_key: Option<String>,

    #[arg(long)]
    pub title: Option<String>,

    #[arg(long)]
    pub priority: Option<i8>,

    #[arg(long)]
    pub sound: Option<String>,

    #[arg(long)]
    pub device: Option<String>,

    #[arg(long)]
    pub url: Option<String>,

    #[arg(long = "url-title")]
    pub url_title: Option<String>,

    /// Create the profile in the system tier instead of the user tier.
    #[arg(long)]
    pub system: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum ServiceKind {
    Pushover,
}
