//! Window for editing a rotation and teaching the NPC check.
//!
//! The rotation file is still the saved form of a build. This window is the
//! way that file gets edited. Live presses stay on the background thread.

use eframe::egui;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::aim::Rgb;
use crate::config::{self, AimMarker, AppConfig, Trigger};
use crate::guard::Verdict;
use crate::rotation::{Build, Mode, Skill};

#[cfg(windows)]
const CAPTURE_DELAY: Duration = Duration::from_secs(3);

pub fn launch(path: &Path) -> Result<(), String> {
    let app = AssistApp::open(path)?;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(1180.0, 780.0))
            .with_min_inner_size(egui::vec2(980.0, 680.0)),
        ..Default::default()
    };
    eframe::run_native(
        "aion-assist",
        options,
        Box::new(move |cc| {
            install_theme(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
    .map_err(|err| err.to_string())
}

fn install_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    if let Some(font) = style.text_styles.get_mut(&egui::TextStyle::Body) {
        font.size = 14.5;
    }
    if let Some(font) = style.text_styles.get_mut(&egui::TextStyle::Button) {
        font.size = 14.5;
    }
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = egui::Color32::from_rgb(16, 20, 28);
    visuals.panel_fill = egui::Color32::from_rgb(20, 25, 34);
    visuals.extreme_bg_color = egui::Color32::from_rgb(12, 15, 20);
    visuals.faint_bg_color = egui::Color32::from_rgb(30, 36, 48);
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(28, 34, 46);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(38, 46, 60);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(58, 70, 92);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(176, 142, 78);
    visuals.widgets.open.bg_fill = egui::Color32::from_rgb(48, 58, 76);
    visuals.selection.bg_fill = egui::Color32::from_rgb(176, 142, 78);
    visuals.hyperlink_color = egui::Color32::from_rgb(222, 186, 112);
    visuals.warn_fg_color = egui::Color32::from_rgb(222, 176, 96);
    visuals.error_fg_color = egui::Color32::from_rgb(220, 104, 96);
    visuals.window_rounding = egui::Rounding::same(8.0);
    style.visuals = visuals;
    ctx.set_style(style);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TriggerKind {
    Toggle,
    Hold,
    Aim,
}

#[derive(Clone, Copy)]
enum CaptureKind {
    Npc,
    Player,
    Aim,
}

#[derive(Clone, Copy)]
struct Capture {
    kind: CaptureKind,
    deadline: Instant,
}

struct Draft {
    build_name: String,
    gcd_ms: String,
    skill_name: String,
    skill_key: String,
    cooldown_ms: String,
    cast_ms: String,
    priority: String,
    charges: String,
    toggle: String,
    cancel: String,
    hold_key: String,
    focus: String,
    key_hold_ms: String,
    min_gap_ms: String,
    aim_tolerance: u8,
    trigger: TriggerKind,
}

struct Worker {
    handle: JoinHandle<Result<(), String>>,
}

struct AssistApp {
    path: PathBuf,
    config: AppConfig,
    saved_toml: String,
    persisted: bool,
    build_index: usize,
    skill_index: usize,
    draft: Draft,
    aim: Option<AimMarker>,
    confirm_delete_build: bool,
    error: Option<String>,
    notice: Option<String>,
    live_status: String,
    status_slot: Arc<Mutex<String>>,
    worker: Option<Worker>,
    capture: Option<Capture>,
    preview: Option<Rgb>,
    preview_at: Instant,
}

impl AssistApp {
    fn open(path: &Path) -> Result<Self, String> {
        if path.exists() {
            let config = config::load(path)?;
            Ok(Self::from_config(path.to_path_buf(), config, true))
        } else {
            Ok(Self::from_config(
                path.to_path_buf(),
                config::starter(),
                false,
            ))
        }
    }

    fn from_config(path: PathBuf, config: AppConfig, persisted: bool) -> Self {
        let saved_toml = config::to_toml(&config);
        let build_index = config
            .builds
            .iter()
            .position(|build| build.active)
            .unwrap_or(0);
        let aim = match &config.trigger {
            Trigger::Aim(marker) => Some(*marker),
            Trigger::Hold(_) | Trigger::Toggle => None,
        };
        let draft = draft_from(&config, build_index, 0, aim);
        Self {
            path,
            config,
            saved_toml,
            persisted,
            build_index,
            skill_index: 0,
            draft,
            aim,
            confirm_delete_build: false,
            error: None,
            notice: None,
            live_status: String::new(),
            status_slot: Arc::new(Mutex::new(String::new())),
            worker: None,
            capture: None,
            preview: None,
            preview_at: Instant::now()
                .checked_sub(Duration::from_secs(1))
                .unwrap_or_else(Instant::now),
        }
    }

    fn editing(&self) -> bool {
        self.worker.is_none()
    }

    fn dirty(&self) -> bool {
        if !self.persisted {
            return true;
        }
        match self.assemble_config() {
            Ok(config) => config::to_toml(&config) != self.saved_toml,
            Err(_) => true,
        }
    }

    fn npc_blocks_start(&self) -> Option<&'static str> {
        if self.config.guard.enabled {
            self.config.guard.ready().err()
        } else {
            None
        }
    }

    fn guard_warning(&self) -> Option<&'static str> {
        if self.config.guard.enabled {
            self.config.guard.ready().err()
        } else {
            Some("NPC check is off, so the rotation can also run while a player is targeted.")
        }
    }

    fn assemble_config(&self) -> Result<AppConfig, String> {
        let mut config = self.config.clone();
        let build = config
            .builds
            .get_mut(self.build_index)
            .ok_or_else(|| "Choose a build.".to_string())?;
        let build_name = self.draft.build_name.trim();
        if build_name.is_empty() {
            return Err("Give the build a name.".into());
        }
        build.name = build_name.to_string();
        build.gcd = Duration::from_millis(parse_u64(&self.draft.gcd_ms, "global cooldown")?);

        let skill = build
            .skills
            .get_mut(self.skill_index)
            .ok_or_else(|| "Choose a skill.".to_string())?;
        let skill_name = self.draft.skill_name.trim();
        if skill_name.is_empty() {
            return Err("Give the skill a name.".into());
        }
        skill.name = skill_name.to_string();
        skill.key = crate::keys::parse(self.draft.skill_key.trim())
            .map_err(|err| format!("skill key: {err}"))?;
        skill.cooldown = Duration::from_millis(parse_u64(&self.draft.cooldown_ms, "cooldown")?);
        skill.cast = Duration::from_millis(parse_u64(&self.draft.cast_ms, "cast time")?);
        skill.priority = parse_u32(&self.draft.priority, "priority")?;
        skill.charges = parse_u32(&self.draft.charges, "charges")?;

        let gap = Duration::from_millis(parse_u64(&self.draft.min_gap_ms, "minimum gap")?);
        for build in &mut config.builds {
            build.min_gap = gap;
        }
        config.hold = Duration::from_millis(parse_u64(&self.draft.key_hold_ms, "key hold")?);
        config.toggle = crate::keys::parse(self.draft.toggle.trim())
            .map_err(|err| format!("toggle: {err}"))?;
        config.cancel = crate::keys::parse(self.draft.cancel.trim())
            .map_err(|err| format!("cancel: {err}"))?;
        let focus = self.draft.focus.trim();
        config.focus = if focus.is_empty() {
            None
        } else {
            Some(focus.to_string())
        };
        config.trigger = match self.draft.trigger {
            TriggerKind::Toggle => Trigger::Toggle,
            TriggerKind::Hold => Trigger::Hold(
                crate::keys::parse(self.draft.hold_key.trim())
                    .map_err(|err| format!("hold: {err}"))?,
            ),
            TriggerKind::Aim => {
                let mut marker = self.aim.ok_or_else(|| {
                    "Capture the aim marker first. Aim at a monster and put the cursor on the marker.".to_string()
                })?;
                marker.tolerance = self.draft.aim_tolerance;
                Trigger::Aim(marker)
            }
        };
        config::parse(&config::to_toml(&config))
    }

    fn commit_drafts(&mut self) -> Result<(), String> {
        self.config = self.assemble_config()?;
        self.reload_drafts();
        Ok(())
    }

    fn reload_drafts(&mut self) {
        if let Trigger::Aim(marker) = &self.config.trigger {
            self.aim = Some(*marker);
        }
        let aim_tolerance = self.draft.aim_tolerance;
        self.draft = draft_from(
            &self.config,
            self.build_index,
            self.skill_index,
            self.aim,
        );
        if self.aim.is_none() {
            self.draft.aim_tolerance = aim_tolerance;
        }
    }

    fn select_build(&mut self, index: usize) {
        if index == self.build_index {
            return;
        }
        if let Err(err) = self.commit_drafts() {
            self.fail(err);
            return;
        }
        self.build_index = index;
        self.skill_index = 0;
        self.confirm_delete_build = false;
        self.reload_drafts();
    }

    fn select_skill(&mut self, index: usize) {
        if index == self.skill_index {
            return;
        }
        if let Err(err) = self.commit_drafts() {
            self.fail(err);
            return;
        }
        self.skill_index = index;
        self.reload_drafts();
    }

    fn add_build(&mut self) {
        if let Err(err) = self.commit_drafts() {
            self.fail(err);
            return;
        }
        let mut name = format!("build {}", self.config.builds.len() + 1);
        while self.config.builds.iter().any(|build| build.name == name) {
            name.push('+');
        }
        self.config.builds.push(Build {
            name,
            mode: Mode::Priority,
            gcd: Duration::from_millis(1000),
            min_gap: self.config.builds[0].min_gap,
            opener: Vec::new(),
            skills: vec![default_skill()],
            active: false,
        });
        self.build_index = self.config.builds.len() - 1;
        self.skill_index = 0;
        self.reload_drafts();
        self.note("Build added. Save when you want it written to the file.".into());
    }

    fn delete_selected_build(&mut self) {
        if !self.confirm_delete_build {
            self.confirm_delete_build = true;
            return;
        }
        self.confirm_delete_build = false;
        if self.config.builds.len() == 1 {
            self.fail("Keep at least one build.".into());
            return;
        }
        self.config.builds.remove(self.build_index);
        if self.build_index >= self.config.builds.len() {
            self.build_index = self.config.builds.len() - 1;
        }
        self.skill_index = 0;
        if !self.config.builds.iter().any(|build| build.active) {
            self.config.builds[self.build_index].active = true;
        }
        self.reload_drafts();
    }

    fn mark_active(&mut self) {
        for (index, build) in self.config.builds.iter_mut().enumerate() {
            build.active = index == self.build_index;
        }
        self.note("This build is the one a command-line run uses.".into());
    }

    fn add_skill(&mut self) {
        if let Err(err) = self.commit_drafts() {
            self.fail(err);
            return;
        }
        let build = &mut self.config.builds[self.build_index];
        let mut skill = default_skill();
        let mut number = 2u32;
        while build.skills.iter().any(|other| other.name == skill.name) {
            skill.name = format!("Skill {number}");
            number += 1;
        }
        build.skills.push(skill);
        self.skill_index = build.skills.len() - 1;
        self.reload_drafts();
    }

    fn remove_selected_skill(&mut self) {
        if let Err(err) = self.commit_drafts() {
            self.fail(err);
            return;
        }
        let build = &mut self.config.builds[self.build_index];
        if build.skills.len() == 1 {
            self.fail("A build needs at least one skill.".into());
            return;
        }
        build.remove_skill(self.skill_index);
        if self.skill_index >= build.skills.len() {
            self.skill_index = build.skills.len() - 1;
        }
        self.reload_drafts();
    }

    fn move_selected(&mut self, to: usize) {
        if let Err(err) = self.commit_drafts() {
            self.fail(err);
            return;
        }
        self.config.builds[self.build_index].move_skill(self.skill_index, to);
        self.skill_index = to;
        self.reload_drafts();
    }

    fn save_file(&mut self) {
        let config = match self.assemble_config() {
            Ok(config) => config,
            Err(err) => {
                self.fail(err);
                return;
            }
        };
        if let Err(err) = config::save(&self.path, &config) {
            self.fail(err);
            return;
        }
        self.saved_toml = config::to_toml(&config);
        self.config = config;
        self.persisted = true;
        self.reload_drafts();
        self.note(format!("Saved {}", self.path.display()));
    }

    fn start(&mut self) {
        let config = match self.assemble_config() {
            Ok(config) => config,
            Err(err) => {
                self.fail(err);
                return;
            }
        };
        if let Err(err) = config.guard.ready() {
            self.fail(err.to_string());
            return;
        }
        self.config = config.clone();
        self.reload_drafts();
        let Some(build_name) = self
            .config
            .builds
            .get(self.build_index)
            .map(|build| build.name.clone())
        else {
            self.fail("Choose a build.".into());
            return;
        };
        if let Ok(mut slot) = self.status_slot.lock() {
            slot.clear();
        }
        match spawn_live(config, build_name, Arc::clone(&self.status_slot)) {
            Ok(handle) => {
                self.worker = Some(Worker { handle });
                self.error = None;
                self.notice = None;
                self.live_status = "Starting…".into();
            }
            Err(err) => self.fail(err),
        }
    }

    fn stop_worker(&mut self) {
        if self.worker.is_none() {
            return;
        }
        crate::send::request_stop();
        let Some(worker) = self.worker.take() else {
            return;
        };
        match worker.handle.join() {
            Ok(Ok(())) => self.live_status = "Stopped.".into(),
            Ok(Err(err)) => self.fail(err),
            Err(_) => self.fail("The rotation thread stopped unexpectedly.".into()),
        }
    }

    fn begin_capture(&mut self, kind: CaptureKind) {
        if !self.editing() {
            return;
        }
        #[cfg(not(windows))]
        {
            let _ = kind;
            self.fail(
                "Color capture runs on Windows, with the game in borderless windowed mode.".into(),
            );
        }
        #[cfg(windows)]
        {
            if matches!(kind, CaptureKind::Player)
                && (self.config.guard.x.is_none() || self.config.guard.y.is_none())
            {
                self.fail(
                    "Capture an NPC first. That chooses the pixel on the target name.".into(),
                );
                return;
            }
            self.capture = Some(Capture {
                kind,
                deadline: Instant::now() + CAPTURE_DELAY,
            });
            self.error = None;
        }
    }

    fn poll_capture(&mut self) {
        let Some(capture) = self.capture else {
            return;
        };
        if Instant::now() < capture.deadline {
            return;
        }
        self.capture = None;
        self.finish_capture(capture.kind);
    }

    fn finish_capture(&mut self, kind: CaptureKind) {
        let sampled = match kind {
            CaptureKind::Aim | CaptureKind::Npc => crate::send::sample_cursor(),
            CaptureKind::Player => {
                let (Some(x), Some(y)) = (self.config.guard.x, self.config.guard.y) else {
                    self.fail(
                        "Capture an NPC first. That chooses the pixel on the target name.".into(),
                    );
                    return;
                };
                crate::send::sample_at(x, y).map(|color| AimMarker {
                    x,
                    y,
                    color,
                    tolerance: self.draft.aim_tolerance,
                })
            }
        };
        match (kind, sampled) {
            (_, Err(err)) => self.fail(err),
            (CaptureKind::Aim, Ok(marker)) => self.store_aim(marker),
            (CaptureKind::Npc, Ok(marker)) => self.store_npc(marker),
            (CaptureKind::Player, Ok(marker)) => self.store_player(marker.color),
        }
    }

    fn store_npc(&mut self, marker: AimMarker) {
        let moved = self.config.guard.x != Some(marker.x) || self.config.guard.y != Some(marker.y);
        self.config.guard.x = Some(marker.x);
        self.config.guard.y = Some(marker.y);
        self.config.guard.npc = Some(marker.color);
        if moved {
            self.config.guard.player = None;
        }
        self.config.guard.enabled = true;
        self.note(
            "NPC color saved. Target a player, keep the target frame in the same place, and capture the player color.".into(),
        );
    }

    fn store_player(&mut self, color: Rgb) {
        self.config.guard.player = Some(color);
        self.config.guard.enabled = true;
        match self.config.guard.ready() {
            Ok(()) => self.note(
                "Player color saved. The rotation runs only when the name matches the NPC.".into(),
            ),
            Err(err) => self.fail(err.to_string()),
        }
    }

    fn store_aim(&mut self, mut marker: AimMarker) {
        marker.tolerance = self.draft.aim_tolerance;
        self.aim = Some(marker);
        self.draft.trigger = TriggerKind::Aim;
        self.note(format!(
            "Aim marker saved at {}, {} color {}.",
            marker.x, marker.y, marker.color
        ));
    }

    fn poll_preview(&mut self) {
        if self.preview_at.elapsed() < Duration::from_millis(200) {
            return;
        }
        self.preview_at = Instant::now();
        #[cfg(windows)]
        {
            self.preview = match (self.config.guard.x, self.config.guard.y) {
                (Some(x), Some(y)) => crate::send::sample_at(x, y).ok(),
                _ => None,
            };
        }
    }

    fn poll_worker(&mut self) {
        if let Ok(text) = self.status_slot.lock() {
            if !text.is_empty() && *text != self.live_status {
                self.live_status = text.clone();
            }
        }
        let finished = self
            .worker
            .as_ref()
            .is_some_and(|worker| worker.handle.is_finished());
        if !finished {
            return;
        }
        let Some(worker) = self.worker.take() else {
            return;
        };
        match worker.handle.join() {
            Ok(Ok(())) => self.live_status = "Stopped.".into(),
            Ok(Err(err)) => self.fail(err),
            Err(_) => self.fail("The rotation thread stopped unexpectedly.".into()),
        }
    }

    fn fail(&mut self, err: String) {
        self.notice = None;
        self.error = Some(err);
    }

    fn note(&mut self, text: String) {
        self.error = None;
        self.notice = Some(text);
    }

    fn header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("aion-assist")
                    .size(22.0)
                    .strong()
                    .color(egui::Color32::from_rgb(222, 186, 112)),
            );
            ui.label(
                egui::RichText::new(self.path.display().to_string())
                    .color(egui::Color32::from_gray(170)),
            );
            if self.dirty() {
                ui.label(
                    egui::RichText::new("unsaved")
                        .small()
                        .color(egui::Color32::from_rgb(222, 176, 96)),
                );
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.worker.is_some() {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("Stop").color(egui::Color32::WHITE),
                            )
                            .fill(egui::Color32::from_rgb(150, 64, 58)),
                        )
                        .clicked()
                    {
                        self.stop_worker();
                    }
                } else if self.npc_blocks_start().is_some() {
                    ui.add_enabled(
                        false,
                        egui::Button::new("Start").fill(egui::Color32::from_rgb(70, 64, 48)),
                    );
                    ui.label(
                        egui::RichText::new("Teach an NPC and a player")
                            .small()
                            .color(egui::Color32::from_rgb(222, 176, 96)),
                    );
                } else if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("Start")
                                .strong()
                                .color(egui::Color32::from_rgb(24, 18, 10)),
                        )
                        .fill(egui::Color32::from_rgb(222, 186, 112)),
                    )
                    .clicked()
                {
                    self.start();
                }
                if ui
                    .add_enabled(self.editing(), egui::Button::new("Save"))
                    .clicked()
                {
                    self.save_file();
                }
            });
        });
    }

    fn builds_column(&mut self, ui: &mut egui::Ui) {
        section_title(ui, "Builds");
        let rows: Vec<(String, bool, String)> = self
            .config
            .builds
            .iter()
            .enumerate()
            .map(|(index, build)| {
                let name = if index == self.build_index {
                    self.draft.build_name.clone()
                } else {
                    build.name.clone()
                };
                (name, build.active, build.mode.as_str().to_string())
            })
            .collect();
        let mut selected = None;
        egui::ScrollArea::vertical()
            .id_salt("build-list")
            .max_height(220.0)
            .show(ui, |ui| {
                for (index, (name, active, mode)) in rows.iter().enumerate() {
                    let mark = if *active { "active" } else { "" };
                    let text = format!("{name}   {mode}  {mark}");
                    if ui
                        .selectable_label(index == self.build_index, text)
                        .clicked()
                    {
                        selected = Some(index);
                    }
                }
            });
        if let Some(index) = selected {
            self.select_build(index);
        }
        ui.horizontal(|ui| {
            if ui
                .add_enabled(self.editing(), egui::Button::new("Add build"))
                .clicked()
            {
                self.add_build();
            }
            let delete_label = if self.confirm_delete_build {
                "Click again to delete"
            } else {
                "Delete build"
            };
            if ui
                .add_enabled(self.editing(), egui::Button::new(delete_label))
                .clicked()
            {
                self.delete_selected_build();
            }
        });
        ui.add_space(8.0);
        egui::Grid::new("build-fields")
            .num_columns(2)
            .spacing(egui::vec2(12.0, 8.0))
            .show(ui, |ui| {
                ui.label("Name");
                ui.add_enabled(
                    self.editing(),
                    egui::TextEdit::singleline(&mut self.draft.build_name).desired_width(180.0),
                );
                ui.end_row();
                ui.label("Global cooldown");
                ui.add_enabled(
                    self.editing(),
                    egui::TextEdit::singleline(&mut self.draft.gcd_ms).desired_width(180.0),
                );
                ui.end_row();
            });
        ui.label(egui::RichText::new("milliseconds").small().weak());
        let mut mode = self.config.builds[self.build_index].mode;
        ui.horizontal(|ui| {
            ui.label("Order");
            let mode_label = mode.as_str().to_string();
            ui.add_enabled_ui(self.editing(), |ui| {
                egui::ComboBox::from_id_salt("build-mode")
                    .selected_text(mode_label)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut mode, Mode::Priority, "priority");
                        ui.selectable_value(&mut mode, Mode::Sequence, "sequence");
                    });
            });
        });
        if self.editing() {
            self.config.builds[self.build_index].mode = mode;
        }
        if ui
            .add_enabled(self.editing(), egui::Button::new("Mark active for CLI"))
            .clicked()
        {
            self.mark_active();
        }
        ui.label(
            egui::RichText::new(
                "Start runs the selected build. Priority uses the number on each skill. Sequence follows this list.",
            )
            .small()
            .weak(),
        );
    }

    fn skills_column(&mut self, ui: &mut egui::Ui) {
        section_title(ui, "Skills");
        let build = &self.config.builds[self.build_index];
        let rows: Vec<(String, String, bool)> = build
            .skills
            .iter()
            .enumerate()
            .map(|(index, skill)| {
                let name = if index == self.skill_index {
                    self.draft.skill_name.clone()
                } else {
                    skill.name.clone()
                };
                (name, skill.key.label.clone(), build.is_opener(index))
            })
            .collect();
        let count = rows.len();
        let mut selected = None;
        let mut move_to = None;
        egui::ScrollArea::vertical()
            .id_salt("skill-list")
            .max_height(320.0)
            .show(ui, |ui| {
                for (index, (name, key, opener)) in rows.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let tag = if *opener { "opener" } else { "" };
                        if ui
                            .selectable_label(
                                index == self.skill_index,
                                format!("{name}    {key}  {tag}"),
                            )
                            .clicked()
                        {
                            selected = Some(index);
                        }
                        if ui
                            .add_enabled(self.editing() && index > 0, egui::Button::new("Up"))
                            .clicked()
                        {
                            move_to = Some(index - 1);
                        }
                        if ui
                            .add_enabled(self.editing() && index + 1 < count, egui::Button::new("Down"))
                            .clicked()
                        {
                            move_to = Some(index + 1);
                        }
                    });
                }
            });
        if let Some(index) = move_to {
            self.move_selected(index);
        } else if let Some(index) = selected {
            self.select_skill(index);
        }
        ui.horizontal(|ui| {
            if ui
                .add_enabled(self.editing(), egui::Button::new("Add skill"))
                .clicked()
            {
                self.add_skill();
            }
            if ui
                .add_enabled(self.editing(), egui::Button::new("Remove skill"))
                .clicked()
            {
                self.remove_selected_skill();
            }
        });
    }

    fn detail_column(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .id_salt("detail")
            .show(ui, |ui| {
                self.skill_form(ui);
                ui.add_space(10.0);
                self.trigger_form(ui);
                ui.add_space(10.0);
                self.guard_form(ui);
            });
    }

    fn skill_form(&mut self, ui: &mut egui::Ui) {
        section_title(ui, "Skill");
        card(ui, |ui| {
            let editing = self.editing();
            egui::Grid::new("skill-fields")
                .num_columns(2)
                .spacing(egui::vec2(12.0, 8.0))
                .show(ui, |ui| {
                    ui.label("Name");
                    ui.add_enabled(
                        editing,
                        egui::TextEdit::singleline(&mut self.draft.skill_name).desired_width(200.0),
                    );
                    ui.end_row();
                    ui.label("Key");
                    ui.add_enabled(
                        editing,
                        egui::TextEdit::singleline(&mut self.draft.skill_key).desired_width(200.0),
                    );
                    ui.end_row();
                    ui.label("Cooldown ms");
                    ui.add_enabled(
                        editing,
                        egui::TextEdit::singleline(&mut self.draft.cooldown_ms).desired_width(200.0),
                    );
                    ui.end_row();
                    ui.label("Cast ms");
                    ui.add_enabled(
                        editing,
                        egui::TextEdit::singleline(&mut self.draft.cast_ms).desired_width(200.0),
                    );
                    ui.end_row();
                    ui.label("Priority");
                    ui.add_enabled(
                        editing,
                        egui::TextEdit::singleline(&mut self.draft.priority).desired_width(200.0),
                    );
                    ui.end_row();
                    ui.label("Charges");
                    ui.add_enabled(
                        editing,
                        egui::TextEdit::singleline(&mut self.draft.charges).desired_width(200.0),
                    );
                    ui.end_row();
                });
            let mut off_gcd = self.config.builds[self.build_index].skills[self.skill_index].off_gcd;
            if ui
                .add_enabled(editing, egui::Checkbox::new(&mut off_gcd, "Off the global cooldown"))
                .changed()
                && editing
            {
                self.config.builds[self.build_index].skills[self.skill_index].off_gcd = off_gcd;
            }
            let mut opener = self.config.builds[self.build_index].is_opener(self.skill_index);
            if ui
                .add_enabled(editing, egui::Checkbox::new(&mut opener, "Part of the opener"))
                .changed()
                && editing
            {
                self.config.builds[self.build_index].set_opener(self.skill_index, opener);
            }
            ui.label(
                egui::RichText::new(
                    "Keys look like 1, Q, F8, or Shift+1. A lower priority number goes first. Cast time holds every skill, including weaves.",
                )
                .small()
                .weak(),
            );
        });
    }

    fn trigger_form(&mut self, ui: &mut egui::Ui) {
        section_title(ui, "Trigger");
        card(ui, |ui| {
            let editing = self.editing();
            ui.horizontal(|ui| {
                ui.add_enabled_ui(editing, |ui| {
                    ui.radio_value(&mut self.draft.trigger, TriggerKind::Hold, "Hold");
                    ui.radio_value(&mut self.draft.trigger, TriggerKind::Aim, "Aim");
                    ui.radio_value(&mut self.draft.trigger, TriggerKind::Toggle, "Toggle");
                });
            });
            match self.draft.trigger {
                TriggerKind::Hold => {
                    labeled_edit(ui, "Hold button", &mut self.draft.hold_key, editing);
                    ui.label(
                        egui::RichText::new(
                            "The rotation runs while this button is down. The game still receives it. RButton is the usual choice.",
                        )
                        .small()
                        .weak(),
                    );
                }
                TriggerKind::Aim => {
                    if let Some(marker) = self.aim {
                        ui.label(format!(
                            "Marker {}, {}   {}",
                            marker.x, marker.y, marker.color
                        ));
                    } else {
                        ui.label("No aim marker captured yet.");
                    }
                    ui.add_enabled_ui(editing, |ui| {
                        ui.add(egui::Slider::new(&mut self.draft.aim_tolerance, 0..=80).text("tolerance"));
                    });
                    if ui
                        .add_enabled(editing, egui::Button::new("Capture aim marker"))
                        .clicked()
                    {
                        self.begin_capture(CaptureKind::Aim);
                    }
                    ui.label(
                        egui::RichText::new(
                            "Aim at a monster in borderless windowed mode, move this window aside, and put the cursor on the marker.",
                        )
                        .small()
                        .weak(),
                    );
                }
                TriggerKind::Toggle => {
                    ui.label(
                        egui::RichText::new(
                            "The toggle key starts and stops. Windows swallows it, so the game does not also receive it.",
                        )
                        .small()
                        .weak(),
                    );
                }
            }
            ui.add_space(6.0);
            labeled_edit(ui, "Toggle key", &mut self.draft.toggle, editing);
            labeled_edit(ui, "Stop key", &mut self.draft.cancel, editing);
            labeled_edit(ui, "Window title", &mut self.draft.focus, editing);
            labeled_edit(ui, "Key hold ms", &mut self.draft.key_hold_ms, editing);
            labeled_edit(ui, "Min gap ms", &mut self.draft.min_gap_ms, editing);
            ui.label(
                egui::RichText::new(
                    "Keys are sent only while the focused window title contains that text. Leave it blank to send to whatever is focused.",
                )
                .small()
                .weak(),
            );
        });
    }

    fn guard_form(&mut self, ui: &mut egui::Ui) {
        section_title(ui, "Players and NPCs");
        card(ui, |ui| {
            let editing = self.editing();
            ui.label(
                "The program cannot see the game's target. It compares one pixel of the target-frame name with two colors you capture. A player color, an unknown color, or two colors that are too close all block the rotation.",
            );
            ui.label(
                egui::RichText::new(
                    "This does not make the tool allowed, and it does not remove the chance of a ban. It only refuses to press keys when that pixel does not look like the NPC you taught.",
                )
                .color(egui::Color32::from_rgb(222, 176, 96)),
            );
            let mut enabled = self.config.guard.enabled;
            if ui
                .add_enabled(
                    editing,
                    egui::Checkbox::new(&mut enabled, "Only run against NPCs"),
                )
                .changed()
                && editing
            {
                self.config.guard.enabled = enabled;
            }
            if let Some(warning) = self.guard_warning() {
                ui.label(
                    egui::RichText::new(warning).color(egui::Color32::from_rgb(222, 176, 96)),
                );
            } else if self.config.guard.enabled {
                ui.label(
                    egui::RichText::new(
                        "NPC check is on. Anything that is not the NPC color stops the rotation.",
                    )
                    .color(egui::Color32::from_rgb(126, 184, 120)),
                );
            }
            ui.add_space(4.0);
            color_row(ui, "NPC", self.config.guard.npc);
            color_row(ui, "Player", self.config.guard.player);
            if let (Some(x), Some(y)) = (self.config.guard.x, self.config.guard.y) {
                ui.horizontal(|ui| {
                    ui.label(format!("Live at {x}, {y}"));
                    color_chip(ui, self.preview);
                    let (text, color) = verdict_style(self.config.guard.verdict(self.preview));
                    ui.label(egui::RichText::new(text).color(color));
                });
            }
            ui.add_enabled_ui(editing, |ui| {
                ui.add(
                    egui::Slider::new(&mut self.config.guard.tolerance, 0..=80).text("color tolerance"),
                );
            });
            ui.horizontal(|ui| {
                if ui.button("Capture NPC").clicked() {
                    self.begin_capture(CaptureKind::Npc);
                }
                if ui.button("Capture player").clicked() {
                    self.begin_capture(CaptureKind::Player);
                }
            });
            ui.label(
                egui::RichText::new(
                    "Use borderless windowed mode. Move this window so it does not cover the target frame. Target an NPC, put the cursor on the name, and capture. Then target a player without moving the frame and capture again. The player sample uses that same pixel, not the cursor.",
                )
                .small()
                .weak(),
            );
        });
    }

    fn status_bar(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if let Some(text) = &self.error {
                ui.label(
                    egui::RichText::new(text).color(egui::Color32::from_rgb(220, 104, 96)),
                );
                return;
            }
            if let Some(text) = self.capture_banner() {
                ui.label(
                    egui::RichText::new(text).color(egui::Color32::from_rgb(222, 186, 112)),
                );
                return;
            }
            if self.worker.is_some() || !self.live_status.is_empty() {
                ui.label(self.live_status.as_str());
                return;
            }
            if let Some(text) = &self.notice {
                ui.label(text);
                return;
            }
            ui.label(
                egui::RichText::new("Edit a build, teach an NPC and a player, then Start.")
                    .weak(),
            );
        });
    }

    fn capture_banner(&self) -> Option<String> {
        let capture = self.capture?;
        let left = capture
            .deadline
            .saturating_duration_since(Instant::now())
            .as_secs_f32();
        let what = match capture.kind {
            CaptureKind::Npc => "Put the cursor on the NPC name in the target frame",
            CaptureKind::Player => {
                "Target a player and leave the target frame where it is"
            }
            CaptureKind::Aim => "Put the cursor on the aim marker",
        };
        Some(format!("{what}. Capturing in {left:.1}s"))
    }
}

impl Drop for AssistApp {
    fn drop(&mut self) {
        if self.worker.is_some() {
            crate::send::request_stop();
            if let Some(worker) = self.worker.take() {
                let _ = worker.handle.join();
            }
        }
    }
}

impl eframe::App for AssistApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_capture();
        self.poll_preview();
        self.poll_worker();

        egui::TopBottomPanel::top("header")
            .exact_height(52.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                self.header(ui);
            });
        egui::TopBottomPanel::bottom("status")
            .exact_height(40.0)
            .show(ctx, |ui| {
                ui.add_space(6.0);
                self.status_bar(ui);
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(3, |columns| {
                self.builds_column(&mut columns[0]);
                self.skills_column(&mut columns[1]);
                self.detail_column(&mut columns[2]);
            });
        });

        let interval = if self.capture.is_some() || self.worker.is_some() {
            Duration::from_millis(50)
        } else if self.config.guard.x.is_some() {
            Duration::from_millis(200)
        } else {
            Duration::from_millis(400)
        };
        ctx.request_repaint_after(interval);
    }
}

fn draft_from(config: &AppConfig, build_index: usize, skill_index: usize, aim: Option<AimMarker>) -> Draft {
    let build = &config.builds[build_index];
    let skill = &build.skills[skill_index];
    let trigger = match &config.trigger {
        Trigger::Toggle => TriggerKind::Toggle,
        Trigger::Hold(_) => TriggerKind::Hold,
        Trigger::Aim(_) => TriggerKind::Aim,
    };
    let hold_key = match &config.trigger {
        Trigger::Hold(key) => key.label.clone(),
        Trigger::Toggle | Trigger::Aim(_) => "RButton".to_string(),
    };
    Draft {
        build_name: build.name.clone(),
        gcd_ms: build.gcd.as_millis().to_string(),
        skill_name: skill.name.clone(),
        skill_key: skill.key.label.clone(),
        cooldown_ms: skill.cooldown.as_millis().to_string(),
        cast_ms: skill.cast.as_millis().to_string(),
        priority: skill.priority.to_string(),
        charges: skill.charges.to_string(),
        toggle: config.toggle.label.clone(),
        cancel: config.cancel.label.clone(),
        hold_key,
        focus: config.focus.clone().unwrap_or_default(),
        key_hold_ms: config.hold.as_millis().to_string(),
        min_gap_ms: build.min_gap.as_millis().to_string(),
        aim_tolerance: aim.map(|marker| marker.tolerance).unwrap_or(32),
        trigger,
    }
}

fn default_skill() -> Skill {
    Skill {
        name: "Skill".to_string(),
        key: crate::keys::parse("1").expect("digit keys parse"),
        cooldown: Duration::from_secs(1),
        cast: Duration::ZERO,
        priority: 100,
        off_gcd: false,
        charges: 1,
    }
}

fn parse_u64(text: &str, what: &str) -> Result<u64, String> {
    text.trim()
        .parse()
        .map_err(|_| format!("{what} needs a whole number"))
}

fn parse_u32(text: &str, what: &str) -> Result<u32, String> {
    text.trim()
        .parse()
        .map_err(|_| format!("{what} needs a whole number"))
}

fn section_title(ui: &mut egui::Ui, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .strong()
            .color(egui::Color32::from_rgb(222, 186, 112)),
    );
    ui.add_space(2.0);
}

fn card(ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(28, 34, 46))
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::same(12.0))
        .show(ui, body);
}

fn labeled_edit(ui: &mut egui::Ui, label: &str, text: &mut String, enabled: bool) {
    ui.horizontal(|ui| {
        ui.add_sized([110.0, 18.0], egui::Label::new(label));
        ui.add_enabled(
            enabled,
            egui::TextEdit::singleline(text).desired_width(180.0),
        );
    });
}

fn color_row(ui: &mut egui::Ui, label: &str, color: Option<Rgb>) {
    ui.horizontal(|ui| {
        ui.add_sized([70.0, 18.0], egui::Label::new(label));
        color_chip(ui, color);
        match color {
            Some(color) => ui.label(color.to_string()),
            None => ui.label(egui::RichText::new("not captured").weak()),
        };
    });
}

fn color_chip(ui: &mut egui::Ui, color: Option<Rgb>) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(36.0, 18.0), egui::Sense::hover());
    let fill = match color {
        Some(color) => egui::Color32::from_rgb(color.r, color.g, color.b),
        None => egui::Color32::from_rgb(16, 18, 24),
    };
    ui.painter().rect(
        rect,
        egui::Rounding::same(4.0),
        fill,
        egui::Stroke::new(1.0, egui::Color32::from_rgb(90, 96, 110)),
    );
}

fn verdict_style(verdict: Verdict) -> (&'static str, egui::Color32) {
    match verdict {
        Verdict::Npc => (
            "NPC — rotation can run",
            egui::Color32::from_rgb(126, 184, 120),
        ),
        Verdict::Player => (
            "Player — rotation will not run",
            egui::Color32::from_rgb(214, 104, 96),
        ),
        Verdict::Unknown => (
            "Not a taught color — rotation will not run",
            egui::Color32::from_rgb(214, 168, 92),
        ),
        Verdict::Untaught => (
            "Teach both colors",
            egui::Color32::from_gray(160),
        ),
    }
}

fn spawn_live(
    config: AppConfig,
    build_name: String,
    status: Arc<Mutex<String>>,
) -> Result<JoinHandle<Result<(), String>>, String> {
    config.guard.ready().map_err(|err| err.to_string())?;
    #[cfg(not(windows))]
    {
        let _ = (config, build_name, status);
        Err("live key sending is built for Windows. Re-run with --dry-run to preview the timeline on this machine.".into())
    }
    #[cfg(windows)]
    {
        Ok(std::thread::spawn(move || {
            let build = config
                .builds
                .iter()
                .find(|build| build.name == build_name)
                .cloned()
                .ok_or_else(|| format!("no build named \"{build_name}\""))?;
            crate::send::run_live(crate::send::Session {
                trigger: &config.trigger,
                toggle: &config.toggle,
                cancel: &config.cancel,
                build: &build,
                key_hold: config.hold,
                focus: config.focus.as_deref(),
                guard: &config.guard,
                status: Some(status),
                verbose: false,
            })
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> AssistApp {
        AssistApp::from_config(PathBuf::from("rotation.toml"), config::starter(), false)
    }

    #[test]
    fn a_new_window_refuses_to_run_until_both_colors_are_taught() {
        let app = app();
        let reason = app.npc_blocks_start().unwrap();
        assert!(reason.contains("Teach"));
    }

    #[test]
    fn a_moved_capture_forgets_the_player_color() {
        let mut app = app();
        app.store_npc(AimMarker {
            x: 10,
            y: 20,
            color: Rgb {
                r: 240,
                g: 240,
                b: 240,
            },
            tolerance: 32,
        });
        app.store_player(Rgb {
            r: 220,
            g: 40,
            b: 40,
        });
        assert!(app.config.guard.ready().is_ok());
        assert!(app.npc_blocks_start().is_none());
        app.store_npc(AimMarker {
            x: 11,
            y: 20,
            color: Rgb {
                r: 240,
                g: 240,
                b: 240,
            },
            tolerance: 32,
        });
        assert!(app.config.guard.player.is_none());
        assert!(app.npc_blocks_start().unwrap().contains("Teach"));
    }

    #[test]
    fn colors_that_match_each_other_block_the_rotation() {
        let mut app = app();
        app.store_npc(AimMarker {
            x: 10,
            y: 20,
            color: Rgb {
                r: 240,
                g: 240,
                b: 240,
            },
            tolerance: 32,
        });
        app.store_player(Rgb {
            r: 245,
            g: 242,
            b: 240,
        });
        assert!(app.config.guard.ready().is_err());
        assert!(app.error.as_deref().unwrap().contains("too close"));
    }

    #[test]
    fn renaming_a_skill_round_trips() {
        let mut app = app();
        app.draft.skill_name = "Slice".into();
        let config = app.assemble_config().unwrap();
        assert_eq!(config.builds[app.build_index].skills[app.skill_index].name, "Slice");
        assert!(config.guard.enabled);
    }

    #[test]
    fn a_mouse_button_cannot_be_a_skill_key() {
        let mut app = app();
        app.draft.skill_key = "RButton".into();
        let err = app.assemble_config().unwrap_err();
        assert!(err.contains("mouse"));
    }
}
