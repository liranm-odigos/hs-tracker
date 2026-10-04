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
    let mut horizon = None;

    while let Some(arg) = args.next() {
        let arg = arg.as_ref();
        match arg {
            "-h" | "--help" => help = true,
            "--list" => list = true,
            "--dry-run" => dry_run = true,
            "--verbose" => verbose = true,
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
                path = Some(PathBuf::from(other));
            }
        }
    }

    if help {
        return Ok(Command::Help);
    }
    let path = path.unwrap_or_else(|| PathBuf::from("rotation.toml"));
    if list {
        return Ok(Command::List { path });
    }
    if horizon.is_some() && !dry_run {
        return Err("--ms is only used with --dry-run".into());
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
    aion-assist [OPTIONS] [rotation.toml]

OPTIONS
    --build <name>   Which build to run. Defaults to the one marked active.
    --dry-run        Print the timeline and send nothing
    --ms <n>         How much of the timeline --dry-run prints (default 10000)
    --list           Show the builds in the file
    --verbose        Print each skill as it is sent
    -h, --help       Show this help

The default file is rotation.toml in the current directory, or beside the
executable. Copy rotation.example.toml to start a build.

Live sending runs on Windows. Press the toggle key to start and stop. Keys
are sent only to the window that is focused.
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
}
