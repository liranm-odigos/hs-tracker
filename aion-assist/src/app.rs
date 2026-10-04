use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::cli::{self, Command};
use crate::config::{self, AppConfig};
use crate::rotation::{self, Build};

pub enum Outcome {
    Done,
    Help(String),
}

pub fn run() -> Result<Outcome, String> {
    let command = cli::parse(std::env::args())?;
    dispatch(command)
}

fn dispatch(command: Command) -> Result<Outcome, String> {
    match command {
        Command::Help => Ok(Outcome::Help(cli::help_text().to_string())),
        Command::List { path } => {
            let path = locate(&path)?;
            let config = config::load(&path)?;
            print_list(&path, &config);
            Ok(Outcome::Done)
        }
        Command::Run {
            path,
            build,
            dry_run,
            horizon,
            verbose,
        } => {
            let path = locate(&path)?;
            let config = config::load(&path)?;
            let selected = config.select(build.as_deref())?;
            if dry_run {
                print_dry_run(&config, selected, horizon);
                Ok(Outcome::Done)
            } else {
                start_live(&config, selected, verbose)
            }
        }
    }
}

fn locate(path: &Path) -> Result<PathBuf, String> {
    if path.exists() {
        return Ok(path.to_path_buf());
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let beside = dir.join(path);
            if beside.exists() {
                return Ok(beside);
            }
        }
    }
    Err(format!(
        "could not find {}. Pass the path to your rotation file, or copy rotation.example.toml to rotation.toml.",
        path.display()
    ))
}

fn print_list(path: &Path, config: &AppConfig) {
    println!("builds in {}", path.display());
    for build in &config.builds {
        let mark = if build.active { "active" } else { "" };
        println!(
            "  {:<20} {:<10} {:>2} skills  {mark}",
            build.name,
            build.mode.as_str(),
            build.skills.len()
        );
    }
}

#[cfg(windows)]
fn start_live(config: &AppConfig, build: &Build, verbose: bool) -> Result<Outcome, String> {
    print_live_banner(config, build);
    crate::send::run_live(
        &config.toggle,
        &config.cancel,
        build,
        config.hold,
        config.focus.as_deref(),
        verbose,
    )?;
    Ok(Outcome::Done)
}

#[cfg(not(windows))]
fn start_live(config: &AppConfig, build: &Build, verbose: bool) -> Result<Outcome, String> {
    let _ = (config, build, verbose);
    Err("live key sending is built for Windows. Re-run with --dry-run to preview the timeline on this machine.".into())
}

#[cfg(windows)]
fn print_live_banner(config: &AppConfig, build: &Build) {
    println!("aion-assist");
    print_build_summary(config, build);
    match &config.focus {
        Some(focus) => {
            println!("sending only while the focused window title contains \"{focus}\"");
        }
        None => println!("sending keys to whatever window is focused"),
    }
    println!(
        "click the game, then press {} to start and stop, {} to quit",
        config.toggle, config.cancel
    );
}

fn print_dry_run(config: &AppConfig, build: &Build, horizon: Duration) {
    println!(
        "dry run for {} ms — no keys will be sent",
        horizon.as_millis()
    );
    print_build_summary(config, build);
    let start = std::time::Instant::now();
    let mut engine = rotation::Engine::new(build.clone(), start);
    for press in rotation::simulate(&mut engine, start, horizon) {
        let skill = &build.skills[press.index];
        println!(
            "{:>6} ms  {}  [{}]",
            press.at.as_millis(),
            skill.name,
            skill.key
        );
    }
}

fn print_build_summary(config: &AppConfig, build: &Build) {
    println!(
        "build: {} ({}, {} skills, gcd {} ms, key hold {} ms)",
        build.name,
        build.mode.as_str(),
        build.skills.len(),
        build.gcd.as_millis(),
        config.hold.as_millis()
    );
    if config.min_gap_raised {
        println!(
            "min gap raised to {} ms so the next key waits until the previous key is released",
            build.min_gap.as_millis()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Command;

    #[test]
    fn help_does_not_touch_the_filesystem() {
        let outcome = dispatch(Command::Help).unwrap();
        match outcome {
            Outcome::Help(text) => assert!(text.contains("--dry-run")),
            Outcome::Done => panic!("expected help"),
        }
    }
}
