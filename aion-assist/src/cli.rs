use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    List {
        path: PathBuf,
    },
    Run {
        path: PathBuf,
        build: Option<String>,
        dry_run: bool,
        horizon: Duration,
        verbose: bool,
    },
    LearnAim {
        path: PathBuf,
    },
    Gui {
        path: PathBuf,
    },
}

pub fn parse<I, S>(args: I) -> Result<Command, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut args = args.into_iter();
    let _program = args.next();

    let mut path = None;
    let mut build = None;
    let mut dry_run = false;
    let mut list = false;
    let mut help = false;
    let mut verbose = false;
    let mut learn_aim = false;
    let mut ui = false;
    let mut path_explicit = false;
    let mut horizon = None;

    while let Some(arg) = args.next() {
        let arg = arg.as_ref();
        match arg {
            "-h" | "--help" => help = true,
            "--list" => list = true,
            "--dry-run" => dry_run = true,
            "--verbose" => verbose = true,
            "--learn-aim" => learn_aim = true,
            "--ui" => ui = true,
            "--build" => {
                build = Some(
                    args.next()
                        .map(|value| value.as_ref().to_string())
                        .ok_or_else(|| "--build needs the build name".to_string())?,
                );
            }
            "--ms" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--ms needs a number of milliseconds".to_string())?;
                let ms: u64 = value
                    .as_ref()
                    .parse()
                    .map_err(|_| format!("--ms {} is not a number", value.as_ref()))?;
                if ms == 0 || ms > 3_600_000 {
                    return Err("--ms must be from 1 to 3600000".into());
                }
                horizon = Some(Duration::from_millis(ms));
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown option {other}"));
            }
            other => {
                if path.is_some() {
                    return Err("only one rotation file can be passed".into());
                }
                path_explicit = true;
                path = Some(PathBuf::from(other));
            }
        }
    }

    if help {
        return Ok(Command::Help);
    }
    let path = path.unwrap_or_else(|| PathBuf::from("rotation.toml"));
    if learn_aim && (list || dry_run || horizon.is_some() || ui) {
        return Err("--learn-aim is used on its own".into());
    }
    if ui && (list || dry_run || horizon.is_some() || verbose || build.is_some()) {
        return Err("--ui opens the window on its own".into());
    }
    if learn_aim {
        return Ok(Command::LearnAim { path });
    }
    if list {
        return Ok(Command::List { path });
    }
    if horizon.is_some() && !dry_run {
        return Err("--ms is only used with --dry-run".into());
    }
    let launch_window = ui
        || (!path_explicit && !dry_run && !verbose && build.is_none() && horizon.is_none());
    if launch_window {
        return Ok(Command::Gui { path });
    }
    Ok(Command::Run {
        path,
        build,
        dry_run,
        horizon: horizon.unwrap_or_else(|| Duration::from_secs(10)),
        verbose,
    })
}

pub fn help_text() -> &'static str {
    "\
aion-assist — send a skill rotation to the focused window

USAGE
    aion-assist
    aion-assist --ui [rotation.toml]
    aion-assist [OPTIONS] [rotation.toml]

With no options, a window opens. Builds, the trigger, and the NPC check are
edited there and saved into rotation.toml.

OPTIONS
    --ui             Open the window (this is also the default)
    --build <name>   Which build to run. Defaults to the one marked active.
    --dry-run        Print the timeline and send nothing
    --ms <n>         How much of the timeline --dry-run prints (default 10000)
    --list           Show the builds in the file
    --learn-aim      Save the aim marker under the mouse into the rotation file
    --verbose        Print each skill as it is sent
    -h, --help       Show this help

The default file is rotation.toml in the current directory, or beside the
executable. The window creates it from the example rotation the first time.

Live sending runs on Windows. A hold trigger runs while you keep a button
down. An aim trigger runs while the taught screen marker is visible. When
the NPC check is on, a second taught pixel has to match an NPC and not a
player. Keys are sent only to the window that is focused.
"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_dry_run() {
        let command = parse([
            "aion-assist",
            "--dry-run",
            "--ms",
            "2500",
            "--build",
            "example",
            "rot.toml",
        ])
        .unwrap();
        assert_eq!(
            command,
            Command::Run {
                path: PathBuf::from("rot.toml"),
                build: Some("example".into()),
                dry_run: true,
                horizon: Duration::from_millis(2500),
                verbose: false,
            }
        );
    }

    #[test]
    fn rejects_ms_without_a_dry_run() {
        let err = parse(["aion-assist", "--ms", "10"]).unwrap_err();
        assert!(err.contains("--dry-run"));
    }

    #[test]
    fn parses_learn_aim() {
        let command = parse(["aion-assist", "--learn-aim", "rotation.toml"]).unwrap();
        assert_eq!(
            command,
            Command::LearnAim {
                path: PathBuf::from("rotation.toml")
            }
        );
    }

    #[test]
    fn no_args_opens_the_window() {
        let command = parse(["aion-assist"]).unwrap();
        assert_eq!(
            command,
            Command::Gui {
                path: PathBuf::from("rotation.toml")
            }
        );
    }

    #[test]
    fn a_path_on_its_own_still_runs() {
        let command = parse(["aion-assist", "rotation.toml"]).unwrap();
        assert_eq!(
            command,
            Command::Run {
                path: PathBuf::from("rotation.toml"),
                build: None,
                dry_run: false,
                horizon: Duration::from_secs(10),
                verbose: false,
            }
        );
    }
}
