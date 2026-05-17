# pushit

A small, cross-platform command-line tool for sending push notifications.

```sh
pushit send "build finished"
pushit send --title "deploy" --priority 1 "v1.2.3 is live"
long-running-job && pushit send "done" || pushit send --priority 1 "failed"
```

Runs on **Linux, macOS, and Windows**, has no runtime dependencies beyond the
binary itself, stores configuration as [RON](https://github.com/ron-rs/ron)
files.

> **Service support:** [Pushover](https://pushover.net) is the only backend as
> of now, but `pushit` is designed to support multiple backends.

## Table of contents

- [Installation](#installation)
- [Quick start](#quick-start)
- [Profiles](#profiles)
  - [User vs. system profiles](#user-vs-system-profiles)
  - [Filesystem locations](#filesystem-locations)
  - [Profile file format](#profile-file-format)
- [Command reference](#command-reference)
  - [`pushit send`](#pushit-send)
  - [`pushit profile add`](#pushit-profile-add)
  - [`pushit profile list`](#pushit-profile-list)
  - [`pushit profile show`](#pushit-profile-show)
  - [`pushit profile use`](#pushit-profile-use)
  - [`pushit profile remove`](#pushit-profile-remove)
  - [`pushit profile path`](#pushit-profile-path)
- [Pushover specifics](#pushover-specifics)
- [Security note](#security-note)
- [Exit codes](#exit-codes)
- [Building from source](#building-from-source)

## Installation

```shell
cargo install pushit
```

## Quick start

Get a [Pushover](https://pushover.net) account, create an application, and grab
two strings:

- your **user key** (one per Pushover account, shown on the dashboard), and
- an **application API token** (one per application you register).

Create a profile and set it as the default:

```shell
pushit profile add personal \
    --service pushover \
    --token  aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
    --user-key bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb

pushit profile use personal
```

Send something:

```shell
pushit send "hello from pushit"
```

Your phone or browser should receive a push notification within a second or two.

You can override per-call attributes with flags:

```shell
pushit send --title "ci" --priority 1 --sound siren "build failed on main"
```

## Profiles

A **profile** bundles everything needed to send a notification through one
account on one service. You can create as many profiles as you want.

### User vs. system profiles

Profiles live in one of two **tiers**:

- The **user tier** is per-user, writable without elevation.
- The **system tier** is machine-wide, pass `--system` to operate on this tier.

On name collisions the **user tier always wins**, `pushit` falls back to the
system value only if the user hasn't set one.

### Filesystem locations

| OS          | User tier                                                | System tier                            |
| ----------- | -------------------------------------------------------- | -------------------------------------- |
| Windows     | `%APPDATA%\pushit\`                                      | `%PROGRAMDATA%\pushit\`                |
| macOS       | `~/Library/Application Support/pushit/`                  | `/Library/Application Support/pushit/` |
| Linux (XDG) | `$XDG_CONFIG_HOME/pushit/` (default `~/.config/pushit/`) | `/etc/xdg/pushit/`                     |

Within each tier:

```
<tier_root>/
    config.ron           # tracks default_profile
    profiles/
        personal.ron
        alerts.ron
        ...
```

You can ask `pushit` directly:

```shell
pushit profile path             # user profiles directory
pushit profile path --system    # system profiles directory
pushit profile path personal    # exact path to one profile
```

### Profile file format

Profiles are stored as [RON](https://github.com/ron-rs/ron) (Rusty Object
Notation.)

Example profile:

```ron
(
    name: "personal",
    service: Pushover((
        token: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        user_key: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    )),
    defaults: (
        title: Some("home server"),
        priority: Some(0),
        sound: None,
        device: None,
        url: None,
        url_title: None,
    ),
)
```

The `service` field is a tagged enum.

The top-level `config.ron` in each tier is even simpler:

```ron
(default_profile: Some("personal"))
```

## Command reference

### `pushit send`

```text
pushit send [OPTIONS] <MESSAGE>
```

Sends a notification through a profile.

| Flag                    | Description                                                                  |
| ----------------------- | ---------------------------------------------------------------------------- |
| `-p`, `--profile NAME`  | Profile to use. Defaults to the resolved default (user > system).            |
| `-t`, `--title TITLE`   | Notification title.                                                          |
| `-P`, `--priority N`    | `-2` lowest … `2` emergency (see [Pushover specifics](#pushover-specifics)). |
| `-s`, `--sound SOUND`   | One of Pushover's [named sounds](https://pushover.net/api#sounds).           |
| `-d`, `--device DEVICE` | Restrict delivery to a single device name.                                   |
| `-u`, `--url URL`       | Supplementary URL.                                                           |
| `--url-title TITLE`     | Display text for `--url`.                                                    |
| `<MESSAGE>`             | The notification body (required, positional).                                |

Example:

```shell
pushit send -t "ci" -P 1 -u "https://ci.example.com/builds/1337" \
            --url-title "view build" "build 1337 failed"
```

### `pushit profile add`

```text
pushit profile add <NAME> --service pushover --token <TOKEN> --user-key <KEY>
                          [--title T] [--priority N] [--sound S]
                          [--device D] [--url U] [--url-title T] [--system]
```

Creates a new profile and writes it to disk.

- `NAME` must match `[A-Za-z0-9_-]+`.
- `--token` and `--user-key` are required when `--service pushover`.
- Any of `--title …` through `--url-title …` are stored as the profile's
  defaults.
- `--system` writes to the system tier instead of the user tier (requires
  elevation).

### `pushit profile list`

```text
pushit profile list
```

Prints profiles from both tiers, with the user's default marked `(default)`, the
system default marked `(system default)`, and any system profile shadowed by a
same-named user profile marked `(shadowed)`:

```text
user:
  alerts (default)
  personal
system:
  alerts (shadowed)
  corporate (system default)
```

### `pushit profile show`

```text
pushit profile show <NAME> [--system]
```

Prints the raw `.ron` contents of a profile. Without `--system`, the user tier
wins on name collisions. With `--system`, the system copy is always shown.

### `pushit profile use`

```text
pushit profile use <NAME> [--system]
```

Sets `NAME` as the default profile. Writes to the user tier by default;
`--system` writes the system-tier default. The named profile must exist in
_some_ tier.

### `pushit profile remove`

```text
pushit profile remove <NAME> [--system]
```

Deletes a profile from the chosen tier (user by default).

### `pushit profile path`

```text
pushit profile path [NAME] [--system]
```

Prints the on-disk location of:

- the profiles directory (when no `NAME` is given), or
- a specific profile's `.ron` file.

`--system` selects the system tier.

Useful for piping into editors, file managers, or shell scripts:

```shell
$EDITOR "$(pushit profile path personal)"
```

## Pushover specifics

`pushit` talks to `https://api.pushover.net/1/messages.json` over HTTPS using a
form-encoded POST. The fields it sends, mapped to
[Pushover's API](https://pushover.net/api):

| `pushit` flag          | Pushover field |
| ---------------------- | -------------- |
| _(profile credential)_ | `token`        |
| _(profile credential)_ | `user`         |
| `<MESSAGE>`            | `message`      |
| `--title`              | `title`        |
| `--priority`           | `priority`     |
| `--sound`              | `sound`        |
| `--device`             | `device`       |
| `--url`                | `url`          |
| `--url-title`          | `url_title`    |

When Pushover returns a non-2xx response, the JSON `errors[]` array is parsed
and surfaced verbatim:

```text
$ pushit send "test"
error: service error: application token is invalid, see https://pushover.net/api
```

## Security note

Profile files containing API keys are stored in **plaintext** on disk.

## Exit codes

| Code | Meaning                                                                                                                                              |
| ---- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `0`  | Success.                                                                                                                                             |
| `1`  | Any error: invalid arguments, missing profile, failed HTTP request, non-2xx response from the service, etc. The error reason is printed to `stderr`. |

## Building from source

```shell
git clone https://github.com/collindutrow/pushit.git
cd pushit
cargo build --release
./target/release/pushit --help
```

---

Issues and PRs welcome.
