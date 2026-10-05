//! Rotation files. Times are milliseconds. The program never learns cooldowns
//! from the game, so the numbers here have to match the build.

use std::fs;
use std::path::Path;
use std::time::Duration;

use serde::Deserialize;

use crate::aim::{self, Rgb};
use crate::guard::Guard;
use crate::keys::{self, KeyCombo};
use crate::rotation::{Build, Mode, Skill};

const MIN_PRESS_MS: u64 = 5;
const MAX_HOLD_MS: u64 = 500;
const MAX_GAP_MS: u64 = 2_000;
const MAX_TIME_MS: u64 = 3_600_000;
const MAX_CHARGES: u32 = 20;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub trigger: Trigger,
    pub toggle: KeyCombo,
    pub cancel: KeyCombo,
    pub hold: Duration,
    pub focus: Option<String>,
    pub min_gap_raised: bool,
    pub guard: Guard,
    pub builds: Vec<Build>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Trigger {
    Toggle,
    Hold(KeyCombo),
    Aim(AimMarker),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AimMarker {
    pub x: i32,
    pub y: i32,
    pub color: Rgb,
    pub tolerance: u8,
}

impl AppConfig {
    pub fn trigger_line(&self) -> String {
        match &self.trigger {
            Trigger::Toggle => format!("trigger: press {} to start and stop", self.toggle),
            Trigger::Hold(key) => format!("trigger: hold {key} to run the rotation"),
            Trigger::Aim(marker) => format!(
                "trigger: aim marker at {},{} color {}",
                marker.x, marker.y, marker.color
            ),
        }
    }
}

impl AppConfig {
    pub fn select(&self, name: Option<&str>) -> Result<&Build, String> {
        if let Some(name) = name {
            return self
                .builds
                .iter()
                .find(|build| build.name == name)
                .ok_or_else(|| format!("no build named \"{name}\""));
        }
        let mut active = self.builds.iter().filter(|build| build.active);
        match (active.next(), active.next()) {
            (Some(build), None) => Ok(build),
            (None, None) if self.builds.len() == 1 => Ok(&self.builds[0]),
            (None, None) => {
                Err("more than one build and none is marked active; pass --build".into())
            }
            _ => Err("more than one build is marked active; pass --build".into()),
        }
    }
}

pub fn load(path: &Path) -> Result<AppConfig, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("could not read {}: {err}", path.display()))?;
    parse(&text)
}

pub fn parse(text: &str) -> Result<AppConfig, String> {
    let raw: RawFile =
        toml::from_str(text).map_err(|err| format!("could not parse rotation file: {err}"))?;
    from_raw(raw)
}

fn from_raw(raw: RawFile) -> Result<AppConfig, String> {
    if raw.builds.is_empty() {
        return Err("rotation file has no builds".into());
    }
    let hold_ms = bounded(raw.key_hold_ms, MIN_PRESS_MS, MAX_HOLD_MS, "key_hold_ms")?;
    let mut gap_ms = bounded(raw.min_gap_ms, MIN_PRESS_MS, MAX_GAP_MS, "min_gap_ms")?;
    let min_gap_raised = if gap_ms < hold_ms {
        gap_ms = hold_ms;
        true
    } else {
        false
    };
    let toggle = keys::parse(&raw.toggle).map_err(|err| format!("toggle: {err}"))?;
    let cancel = keys::parse(&raw.cancel).map_err(|err| format!("cancel: {err}"))?;
    if toggle.is_mouse() {
        return Err(
            "toggle must be a keyboard key. A mouse button belongs on hold = \"RButton\".".into(),
        );
    }
    if cancel.is_mouse() {
        return Err("cancel must be a keyboard key".into());
    }
    if same_key(&toggle, &cancel) {
        return Err("toggle and cancel are the same key".into());
    }
    let trigger = trigger_from_raw(&raw, &cancel)?;
    let guard = guard_from_raw(&raw)?;

    let mut names = Vec::new();
    let mut builds = Vec::with_capacity(raw.builds.len());
    for build in raw.builds {
        if build.name.trim().is_empty() {
            return Err("a build is missing its name".into());
        }
        if names.iter().any(|name: &String| name == &build.name) {
            return Err(format!("build \"{}\" is listed more than once", build.name));
        }
        names.push(build.name.clone());
        builds.push(build_from_raw(build, Duration::from_millis(gap_ms))?);
    }

    let focus = raw.focus.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });

    Ok(AppConfig {
        trigger,
        toggle,
        cancel,
        hold: Duration::from_millis(hold_ms),
        focus,
        min_gap_raised,
        guard,
        builds,
    })
}

fn build_from_raw(raw: RawBuild, min_gap: Duration) -> Result<Build, String> {
    let mode = match raw.mode.to_ascii_lowercase().as_str() {
        "priority" => Mode::Priority,
        "sequence" => Mode::Sequence,
        other => {
            return Err(format!(
                "build \"{}\" has unknown mode \"{other}\" (use priority or sequence)",
                raw.name
            ));
        }
    };
    if raw.skills.is_empty() {
        return Err(format!("build \"{}\" has no skills", raw.name));
    }
    let gcd = time_ms(raw.gcd_ms, &format!("build \"{}\" gcd_ms", raw.name))?;

    let mut seen = Vec::new();
    let mut skills = Vec::with_capacity(raw.skills.len());
    for skill in &raw.skills {
        if skill.name.trim().is_empty() {
            return Err(format!("build \"{}\" has a skill with no name", raw.name));
        }
        if seen.iter().any(|name: &String| name == &skill.name) {
            return Err(format!(
                "build \"{}\" lists skill \"{}\" more than once",
                raw.name, skill.name
            ));
        }
        seen.push(skill.name.clone());
        if skill.charges == 0 || skill.charges > MAX_CHARGES {
            return Err(format!(
                "skill \"{}\" charges must be from 1 to {MAX_CHARGES}",
                skill.name
            ));
        }
        let key =
            keys::parse(&skill.key).map_err(|err| format!("skill \"{}\": {err}", skill.name))?;
        if key.is_mouse() {
            return Err(format!(
                "skill \"{}\" key \"{}\" is a mouse button. Use it as the hold trigger, not as a skill.",
                skill.name, key
            ));
        }
        skills.push(Skill {
            name: skill.name.clone(),
            key,
            cooldown: time_ms(
                skill.cooldown_ms,
                &format!("skill \"{}\" cooldown_ms", skill.name),
            )?,
            cast: time_ms(skill.cast_ms, &format!("skill \"{}\" cast_ms", skill.name))?,
            priority: skill.priority,
            off_gcd: skill.off_gcd,
            charges: skill.charges,
        });
    }

    let mut opener = Vec::with_capacity(raw.opener.len());
    for name in &raw.opener {
        let index = skills
            .iter()
            .position(|skill| skill.name == *name)
            .ok_or_else(|| {
                format!(
                    "build \"{}\" opener \"{name}\" does not match a skill",
                    raw.name
                )
            })?;
        opener.push(index);
    }

    Ok(Build {
        name: raw.name,
        mode,
        gcd,
        min_gap,
        opener,
        skills,
        active: raw.active,
    })
}

fn bounded(value: u64, min: u64, max: u64, what: &str) -> Result<u64, String> {
    if value < min {
        return Err(format!("{what} is {value} ms; the minimum is {min} ms"));
    }
    if value > max {
        return Err(format!("{what} is {value} ms; the maximum is {max} ms"));
    }
    Ok(value)
}

fn time_ms(value: u64, what: &str) -> Result<Duration, String> {
    if value > MAX_TIME_MS {
        return Err(format!(
            "{what} is {value} ms; the maximum is {MAX_TIME_MS} ms"
        ));
    }
    Ok(Duration::from_millis(value))
}

fn default_hold() -> u64 {
    15
}

fn default_gap() -> u64 {
    15
}

fn default_priority() -> u32 {
    100
}

fn default_charges() -> u32 {
    1
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    #[serde(default = "default_trigger")]
    trigger: String,
    #[serde(default)]
    hold: Option<String>,
    #[serde(default)]
    aim_x: Option<i64>,
    #[serde(default)]
    aim_y: Option<i64>,
    #[serde(default)]
    aim_color: Option<String>,
    #[serde(default)]
    aim_tolerance: Option<u64>,
    #[serde(default)]
    guard: bool,
    #[serde(default)]
    guard_x: Option<i64>,
    #[serde(default)]
    guard_y: Option<i64>,
    #[serde(default)]
    npc_color: Option<String>,
    #[serde(default)]
    player_color: Option<String>,
    #[serde(default)]
    guard_tolerance: Option<u64>,
    toggle: String,
    cancel: String,
    #[serde(default = "default_hold")]
    key_hold_ms: u64,
    #[serde(default = "default_gap")]
    min_gap_ms: u64,
    #[serde(default)]
    focus: Option<String>,
    builds: Vec<RawBuild>,
}

fn default_trigger() -> String {
    "toggle".into()
}

fn same_key(left: &KeyCombo, right: &KeyCombo) -> bool {
    left.vk == right.vk
        && left.shift == right.shift
        && left.ctrl == right.ctrl
        && left.alt == right.alt
}

fn trigger_from_raw(raw: &RawFile, cancel: &KeyCombo) -> Result<Trigger, String> {
    match raw.trigger.to_ascii_lowercase().as_str() {
        "toggle" => Ok(Trigger::Toggle),
        "hold" => {
            let name = raw.hold.as_deref().ok_or_else(|| {
                "trigger = \"hold\" needs hold = \"RButton\" (or whichever button you keep down)"
                    .to_string()
            })?;
            let key = keys::parse(name).map_err(|err| format!("hold: {err}"))?;
            if same_key(&key, cancel) {
                return Err("hold and cancel are the same key".into());
            }
            Ok(Trigger::Hold(key))
        }
        "aim" => {
            let x = raw.aim_x.ok_or_else(|| {
                "trigger = \"aim\" needs aim_x, aim_y, and aim_color. Run aion-assist --learn-aim while aiming at a monster.".to_string()
            })?;
            let y = raw.aim_y.ok_or_else(|| {
                "trigger = \"aim\" needs aim_y. Run aion-assist --learn-aim while aiming at a monster.".to_string()
            })?;
            let color = raw.aim_color.as_deref().ok_or_else(|| {
                "trigger = \"aim\" needs aim_color. Run aion-assist --learn-aim while aiming at a monster.".to_string()
            })?;
            let tolerance = match raw.aim_tolerance {
                None => 32,
                Some(value) if value <= 255 => value as u8,
                Some(value) => {
                    return Err(format!("aim_tolerance is {value}; the maximum is 255"));
                }
            };
            Ok(Trigger::Aim(AimMarker {
                x: screen_coord(x, "aim_x")?,
                y: screen_coord(y, "aim_y")?,
                color: aim::parse_color(color)?,
                tolerance,
            }))
        }
        other => Err(format!(
            "trigger \"{other}\" is unknown (use toggle, hold, or aim)"
        )),
    }
}

fn screen_coord(value: i64, what: &str) -> Result<i32, String> {
    i32::try_from(value).map_err(|_| format!("{what} is out of range"))
}

fn guard_from_raw(raw: &RawFile) -> Result<Guard, String> {
    let tolerance = match raw.guard_tolerance {
        None => 32,
        Some(value) if value <= 255 => value as u8,
        Some(value) => return Err(format!("guard_tolerance is {value}; the maximum is 255")),
    };
    let npc = match &raw.npc_color {
        Some(color) => Some(aim::parse_color(color).map_err(|err| format!("npc_color: {err}"))?),
        None => None,
    };
    let player = match &raw.player_color {
        Some(color) => {
            Some(aim::parse_color(color).map_err(|err| format!("player_color: {err}"))?)
        }
        None => None,
    };
    Ok(Guard {
        enabled: raw.guard,
        x: raw.guard_x.map(|value| screen_coord(value, "guard_x")).transpose()?,
        y: raw.guard_y.map(|value| screen_coord(value, "guard_y")).transpose()?,
        npc,
        player,
        tolerance,
    })
}

pub fn starter() -> AppConfig {
    let mut config = parse(include_str!("../rotation.example.toml"))
        .expect("the example rotation is valid");
    config.guard.enabled = true;
    config
}

pub fn save(path: &Path, config: &AppConfig) -> Result<(), String> {
    let text = to_toml(config);
    parse(&text)?;
    fs::write(path, text).map_err(|err| format!("could not write {}: {err}", path.display()))
}

pub fn to_toml(config: &AppConfig) -> String {
    let mut out = String::new();
    match &config.trigger {
        Trigger::Toggle => out.push_str("trigger = \"toggle\"\n"),
        Trigger::Hold(key) => {
            out.push_str("trigger = \"hold\"\n");
            out.push_str(&format!("hold = {}\n", toml_string(&key.label)));
        }
        Trigger::Aim(marker) => {
            out.push_str("trigger = \"aim\"\n");
            out.push_str(&format!("aim_x = {}\n", marker.x));
            out.push_str(&format!("aim_y = {}\n", marker.y));
            out.push_str(&format!("aim_color = \"{}\"\n", marker.color));
            out.push_str(&format!("aim_tolerance = {}\n", marker.tolerance));
        }
    }
    out.push_str(&format!("toggle = {}\n", toml_string(&config.toggle.label)));
    out.push_str(&format!("cancel = {}\n", toml_string(&config.cancel.label)));
    out.push_str(&format!("key_hold_ms = {}\n", config.hold.as_millis()));
    let gap = config
        .builds
        .first()
        .map(|build| build.min_gap.as_millis())
        .unwrap_or(15);
    out.push_str(&format!("min_gap_ms = {gap}\n"));
    if let Some(focus) = &config.focus {
        out.push_str(&format!("focus = {}\n", toml_string(focus)));
    }
    out.push_str(&format!("guard = {}\n", config.guard.enabled));
    if let Some(x) = config.guard.x {
        out.push_str(&format!("guard_x = {x}\n"));
    }
    if let Some(y) = config.guard.y {
        out.push_str(&format!("guard_y = {y}\n"));
    }
    if let Some(color) = config.guard.npc {
        out.push_str(&format!("npc_color = \"{color}\"\n"));
    }
    if let Some(color) = config.guard.player {
        out.push_str(&format!("player_color = \"{color}\"\n"));
    }
    out.push_str(&format!(
        "guard_tolerance = {}\n",
        config.guard.tolerance
    ));
    for build in &config.builds {
        out.push_str("\n[[builds]]\n");
        out.push_str(&format!("name = {}\n", toml_string(&build.name)));
        out.push_str(&format!("mode = \"{}\"\n", build.mode.as_str()));
        out.push_str(&format!("gcd_ms = {}\n", build.gcd.as_millis()));
        if build.active {
            out.push_str("active = true\n");
        }
        if !build.opener.is_empty() {
            let names: Vec<String> = build
                .opener
                .iter()
                .filter_map(|index| build.skills.get(*index).map(|skill| toml_string(&skill.name)))
                .collect();
            out.push_str(&format!("opener = [{}]\n", names.join(", ")));
        }
        for skill in &build.skills {
            out.push_str("\n[[builds.skills]]\n");
            out.push_str(&format!("name = {}\n", toml_string(&skill.name)));
            out.push_str(&format!("key = {}\n", toml_string(&skill.key.label)));
            out.push_str(&format!("cooldown_ms = {}\n", skill.cooldown.as_millis()));
            if skill.cast != Duration::ZERO {
                out.push_str(&format!("cast_ms = {}\n", skill.cast.as_millis()));
            }
            out.push_str(&format!("priority = {}\n", skill.priority));
            if skill.off_gcd {
                out.push_str("off_gcd = true\n");
            }
            if skill.charges != 1 {
                out.push_str(&format!("charges = {}\n", skill.charges));
            }
        }
    }
    out.push('\n');
    out
}

fn toml_string(value: &str) -> String {
    format!(
        "\"{}\"",
        value.replace('\\', "\\\\").replace('"', "\\\"")
    )
}

/// Writes the aim marker into a rotation file, replacing an older marker.
pub fn upsert_aim(text: &str, marker: AimMarker) -> String {
    let lines = [
        ("trigger", "trigger = \"aim\"".to_string()),
        ("aim_x", format!("aim_x = {}", marker.x)),
        ("aim_y", format!("aim_y = {}", marker.y)),
        ("aim_color", format!("aim_color = \"{}\"", marker.color)),
        (
            "aim_tolerance",
            format!("aim_tolerance = {}", marker.tolerance),
        ),
    ];
    let mut found = [false; 5];
    let mut out = String::new();
    for line in text.lines() {
        let mut replaced = false;
        for (index, (key, value)) in lines.iter().enumerate() {
            if line_sets(line, key) {
                out.push_str(value);
                out.push('\n');
                found[index] = true;
                replaced = true;
                break;
            }
        }
        if !replaced {
            out.push_str(line);
            out.push('\n');
        }
    }
    for (index, (_, value)) in lines.iter().enumerate() {
        if !found[index] {
            out.push_str(value);
            out.push('\n');
        }
    }
    out
}

fn line_sets(line: &str, key: &str) -> bool {
    let trimmed = line.trim();
    let Some(rest) = trimmed.strip_prefix(key) else {
        return false;
    };
    rest.trim_start().starts_with('=')
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBuild {
    name: String,
    mode: String,
    gcd_ms: u64,
    #[serde(default)]
    opener: Vec<String>,
    #[serde(default)]
    active: bool,
    skills: Vec<RawSkill>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSkill {
    name: String,
    key: String,
    cooldown_ms: u64,
    #[serde(default)]
    cast_ms: u64,
    #[serde(default = "default_priority")]
    priority: u32,
    #[serde(default)]
    off_gcd: bool,
    #[serde(default = "default_charges")]
    charges: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rotation::simulate;
    use std::time::Instant;

    #[test]
    fn example_file_matches_the_rotation_clock() {
        let config = parse(include_str!("../rotation.example.toml")).unwrap();
        let build = config.select(Some("example")).unwrap();
        let start = Instant::now();
        let mut engine = crate::rotation::Engine::new(build.clone(), start);
        let presses = simulate(&mut engine, start, Duration::from_millis(5100));
        let names: Vec<(u128, &str)> = presses
            .iter()
            .map(|press| {
                (
                    press.at.as_millis(),
                    build.skills[press.index].name.as_str(),
                )
            })
            .collect();
        assert_eq!(
            names,
            vec![
                (0, "Opener"),
                (15, "Weave"),
                (1000, "Burst"),
                (2000, "Filler"),
                (3000, "Filler"),
                (4000, "Filler"),
                (4015, "Weave"),
                (5000, "Filler"),
            ]
        );
    }

    #[test]
    fn rejects_a_short_hold_and_an_unknown_opener() {
        let hold = r#"
            toggle = "F8"
            cancel = "F9"
            key_hold_ms = 1
            [[builds]]
            name = "x"
            mode = "priority"
            gcd_ms = 1000
            [[builds.skills]]
            name = "Only"
            key = "1"
            cooldown_ms = 1000
        "#;
        assert!(parse(hold).unwrap_err().contains("minimum"));

        let opener = r#"
            toggle = "F8"
            cancel = "F9"
            [[builds]]
            name = "x"
            mode = "priority"
            gcd_ms = 1000
            opener = ["Missing"]
            [[builds.skills]]
            name = "Only"
            key = "1"
            cooldown_ms = 1000
        "#;
        assert!(parse(opener).unwrap_err().contains("Missing"));
    }

    #[test]
    fn hold_and_aim_triggers_parse() {
        let hold = r#"
            toggle = "F8"
            cancel = "F9"
            trigger = "hold"
            hold = "RButton"
            [[builds]]
            name = "x"
            mode = "priority"
            gcd_ms = 1000
            [[builds.skills]]
            name = "Only"
            key = "1"
            cooldown_ms = 1000
        "#;
        let config = parse(hold).unwrap();
        match config.trigger {
            Trigger::Hold(key) => assert_eq!(key.vk, 0x02),
            other => panic!("expected hold, got {other:?}"),
        }

        let aim = r#"
            toggle = "F8"
            cancel = "F9"
            trigger = "aim"
            aim_x = 960
            aim_y = 140
            aim_color = "F2F2F0"
            [[builds]]
            name = "x"
            mode = "priority"
            gcd_ms = 1000
            [[builds.skills]]
            name = "Only"
            key = "1"
            cooldown_ms = 1000
        "#;
        let config = parse(aim).unwrap();
        match config.trigger {
            Trigger::Aim(marker) => {
                assert_eq!(marker.x, 960);
                assert_eq!(marker.y, 140);
                assert_eq!(marker.tolerance, 32);
            }
            other => panic!("expected aim, got {other:?}"),
        }
    }

    #[test]
    fn upsert_aim_replaces_an_existing_marker() {
        let marker = AimMarker {
            x: 10,
            y: 20,
            color: crate::aim::Rgb { r: 1, g: 2, b: 3 },
            tolerance: 32,
        };
        let once = upsert_aim("toggle = \"F8\"\n# trigger = \"leave me\"\n", marker);
        assert!(once.contains("trigger = \"aim\"\n"));
        assert!(once.contains("# trigger = \"leave me\"\n"));
        let again = upsert_aim(&once, AimMarker { x: 11, ..marker });
        assert_eq!(again.matches("aim_x").count(), 1);
        assert!(again.contains("aim_x = 11\n"));
    }

    #[test]
    fn a_saved_file_loads_the_npc_guard() {
        let mut config = starter();
        config.guard.x = Some(12);
        config.guard.y = Some(34);
        config.guard.npc = Some(crate::aim::Rgb {
            r: 10,
            g: 20,
            b: 30,
        });
        config.guard.player = Some(crate::aim::Rgb {
            r: 200,
            g: 30,
            b: 30,
        });
        let loaded = parse(&to_toml(&config)).unwrap();
        assert!(loaded.guard.enabled);
        assert_eq!(loaded.guard.x, Some(12));
        assert_eq!(loaded.guard.npc, config.guard.npc);
        assert_eq!(loaded.guard.player, config.guard.player);
        assert_eq!(loaded.builds[0].name, config.builds[0].name);
        assert_eq!(loaded.builds[0].opener, config.builds[0].opener);
    }

    #[test]
    fn the_example_file_leaves_the_guard_off() {
        let config = parse(include_str!("../rotation.example.toml")).unwrap();
        assert!(!config.guard.enabled);
        assert!(config.guard.npc.is_none());
        assert!(starter().guard.enabled);
    }
}
