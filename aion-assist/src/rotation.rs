//! Which skill is next, based only on the timers in the build file.
//!
//! The game client is never read. A press starts that skill's cooldown, the
//! global cooldown, and any cast lock at the moment the key goes down.

use std::time::{Duration, Instant};

use crate::keys::KeyCombo;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Priority,
    Sequence,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Priority => "priority",
            Self::Sequence => "sequence",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Skill {
    pub name: String,
    pub key: KeyCombo,
    pub cooldown: Duration,
    pub cast: Duration,
    pub priority: u32,
    pub off_gcd: bool,
    pub charges: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Build {
    pub name: String,
    pub mode: Mode,
    pub gcd: Duration,
    pub min_gap: Duration,
    pub opener: Vec<usize>,
    pub skills: Vec<Skill>,
    pub active: bool,
}

#[derive(Clone, Debug)]
struct Charges {
    available: u32,
    max: u32,
    next: Option<Instant>,
    recharge: Duration,
}

impl Charges {
    fn full(max: u32, recharge: Duration) -> Self {
        Self {
            available: max,
            max,
            next: None,
            recharge,
        }
    }

    fn refresh(&mut self, now: Instant) {
        while self.available < self.max {
            let Some(at) = self.next else {
                break;
            };
            if at > now {
                break;
            }
            self.available += 1;
            if self.available < self.max {
                self.next = Some(at + self.recharge);
            } else {
                self.next = None;
            }
        }
    }

    fn ready_at(&self) -> Option<Instant> {
        if self.available > 0 {
            None
        } else {
            self.next
        }
    }

    fn consume(&mut self, now: Instant) {
        self.refresh(now);
        self.available = self.available.saturating_sub(1);
        if self.next.is_none() && self.available < self.max {
            self.next = Some(now + self.recharge);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Engine {
    build: Build,
    charges: Vec<Charges>,
    gcd_ready_at: Instant,
    locked_until: Instant,
    earliest: Instant,
    opener_pos: usize,
    sequence_pos: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Press { index: usize },
    Wait { until: Instant },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Press {
    pub at: Duration,
    pub index: usize,
}

impl Engine {
    pub fn new(build: Build, start: Instant) -> Self {
        assert!(build.min_gap > Duration::ZERO, "min gap must be non-zero");
        let charges = build
            .skills
            .iter()
            .map(|skill| Charges::full(skill.charges, skill.cooldown))
            .collect();
        Self {
            build,
            charges,
            gcd_ready_at: start,
            locked_until: start,
            earliest: start,
            opener_pos: 0,
            sequence_pos: 0,
        }
    }

    pub fn build(&self) -> &Build {
        &self.build
    }

    pub fn poll(&mut self, now: Instant) -> Step {
        self.refresh(now);
        if self.opener_pos < self.build.opener.len() {
            return self.poll_opener(now);
        }
        match self.build.mode {
            Mode::Priority => self.poll_priority(now),
            Mode::Sequence => {
                let index = self.sequence_pos % self.build.skills.len();
                self.step_for(index, now)
            }
        }
    }

    pub fn commit(&mut self, index: usize, now: Instant) {
        self.refresh(now);
        let off_gcd = self.build.skills[index].off_gcd;
        let cast = self.build.skills[index].cast;
        let gcd = self.build.gcd;
        let min_gap = self.build.min_gap;
        self.charges[index].consume(now);
        self.earliest = now + min_gap;
        if cast > Duration::ZERO {
            let until = now + cast;
            if until > self.locked_until {
                self.locked_until = until;
            }
        }
        if !off_gcd {
            let until = now + gcd;
            if until > self.gcd_ready_at {
                self.gcd_ready_at = until;
            }
        }
        self.advance(index);
    }

    fn refresh(&mut self, now: Instant) {
        for charges in &mut self.charges {
            charges.refresh(now);
        }
    }

    fn poll_opener(&self, now: Instant) -> Step {
        let index = self.build.opener[self.opener_pos];
        let opener_at = self.gate(index);
        if opener_at <= now {
            return Step::Press { index };
        }
        if let Some(weave) = self.best_ready(now, true) {
            return Step::Press { index: weave };
        }
        let until = self
            .soonest_gate(now, true)
            .map_or(opener_at, |weave_at| opener_at.min(weave_at));
        Step::Wait { until }
    }

    fn poll_priority(&self, now: Instant) -> Step {
        if let Some(index) = self.best_ready(now, false) {
            return Step::Press { index };
        }
        let until = self
            .soonest_gate(now, false)
            .unwrap_or_else(|| now + Duration::from_secs(1));
        Step::Wait { until }
    }

    fn best_ready(&self, now: Instant, off_gcd_only: bool) -> Option<usize> {
        let mut best: Option<(u32, usize)> = None;
        for (index, skill) in self.build.skills.iter().enumerate() {
            if off_gcd_only && (!skill.off_gcd || self.reserved_by_opener(index)) {
                continue;
            }
            if self.gate(index) > now {
                continue;
            }
            let replace = best.map_or(true, |(priority, at)| {
                (skill.priority, index) < (priority, at)
            });
            if replace {
                best = Some((skill.priority, index));
            }
        }
        best.map(|(_, index)| index)
    }

    fn soonest_gate(&self, now: Instant, off_gcd_only: bool) -> Option<Instant> {
        let mut soonest = None;
        for (index, skill) in self.build.skills.iter().enumerate() {
            if off_gcd_only && (!skill.off_gcd || self.reserved_by_opener(index)) {
                continue;
            }
            let at = self.gate(index);
            if at <= now {
                continue;
            }
            soonest = Some(soonest.map_or(at, |current: Instant| current.min(at)));
        }
        soonest
    }

    fn reserved_by_opener(&self, index: usize) -> bool {
        self.build.opener[self.opener_pos..].contains(&index)
    }

    fn step_for(&self, index: usize, now: Instant) -> Step {
        let at = self.gate(index);
        if at <= now {
            Step::Press { index }
        } else {
            Step::Wait { until: at }
        }
    }

    fn gate(&self, index: usize) -> Instant {
        let skill = &self.build.skills[index];
        let mut at = self.earliest.max(self.locked_until);
        if !skill.off_gcd {
            at = at.max(self.gcd_ready_at);
        }
        if let Some(charge_at) = self.charges[index].ready_at() {
            at = at.max(charge_at);
        }
        at
    }

    fn advance(&mut self, index: usize) {
        if self.opener_pos < self.build.opener.len() {
            if self.build.opener[self.opener_pos] == index {
                self.opener_pos += 1;
            }
            return;
        }
        if self.build.mode == Mode::Sequence {
            let len = self.build.skills.len();
            if len > 0 && self.sequence_pos % len == index {
                self.sequence_pos = (self.sequence_pos + 1) % len;
            }
        }
    }
}

pub fn simulate(engine: &mut Engine, start: Instant, horizon: Duration) -> Vec<Press> {
    let end = start + horizon;
    let mut now = start;
    let mut presses = Vec::new();
    for _ in 0..100_000 {
        if now >= end {
            break;
        }
        match engine.poll(now) {
            Step::Press { index } => {
                presses.push(Press {
                    at: now.saturating_duration_since(start),
                    index,
                });
                engine.commit(index, now);
            }
            Step::Wait { until } => {
                if until <= now {
                    now += Duration::from_millis(1);
                } else if until >= end {
                    break;
                } else {
                    now = until;
                }
            }
        }
    }
    presses
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::parse;

    fn skill(name: &str, key: &str, cooldown_ms: u64, priority: u32, off_gcd: bool) -> Skill {
        Skill {
            name: name.into(),
            key: parse(key).unwrap(),
            cooldown: Duration::from_millis(cooldown_ms),
            cast: Duration::ZERO,
            priority,
            off_gcd,
            charges: 1,
        }
    }

    fn at(presses: &[Press], n: usize) -> (u128, usize) {
        (presses[n].at.as_millis(), presses[n].index)
    }

    #[test]
    fn priority_uses_gcd_and_weaves_off_gcd() {
        let build = Build {
            name: "test".into(),
            mode: Mode::Priority,
            gcd: Duration::from_millis(1000),
            min_gap: Duration::from_millis(50),
            opener: vec![],
            skills: vec![
                skill("Weave", "Q", 2000, 0, true),
                skill("High", "1", 2500, 1, false),
                skill("Low", "2", 0, 2, false),
            ],
            active: false,
        };
        let start = Instant::now();
        let mut engine = Engine::new(build, start);
        let presses = simulate(&mut engine, start, Duration::from_millis(2600));

        assert_eq!(at(&presses, 0), (0, 0));
        assert_eq!(at(&presses, 1), (50, 1));
        assert_eq!(at(&presses, 2), (1050, 2));
        assert_eq!(at(&presses, 3), (2000, 0));
        assert_eq!(at(&presses, 4), (2050, 2));
        assert_eq!(presses.len(), 5);
    }

    #[test]
    fn cast_lock_blocks_weaves() {
        let mut cast = skill("Cast", "1", 5000, 0, false);
        cast.cast = Duration::from_millis(1500);
        let weave = skill("Weave", "Q", 0, 1, true);
        let build = Build {
            name: "cast".into(),
            mode: Mode::Priority,
            gcd: Duration::from_millis(1000),
            min_gap: Duration::from_millis(20),
            opener: vec![],
            skills: vec![cast, weave],
            active: false,
        };
        let start = Instant::now();
        let mut engine = Engine::new(build, start);
        let presses = simulate(&mut engine, start, Duration::from_millis(1600));
        assert_eq!(at(&presses, 0), (0, 0));
        assert_eq!(at(&presses, 1), (1500, 1));
    }

    #[test]
    fn sequence_waits_for_the_next_skill_instead_of_skipping() {
        let build = Build {
            name: "seq".into(),
            mode: Mode::Sequence,
            gcd: Duration::from_millis(1000),
            min_gap: Duration::from_millis(10),
            opener: vec![],
            skills: vec![
                skill("A", "1", 3000, 1, false),
                skill("B", "2", 0, 1, false),
            ],
            active: false,
        };
        let start = Instant::now();
        let mut engine = Engine::new(build, start);
        let presses = simulate(&mut engine, start, Duration::from_millis(3100));
        assert_eq!(at(&presses, 0), (0, 0));
        assert_eq!(at(&presses, 1), (1000, 1));
        assert_eq!(at(&presses, 2), (3000, 0));
    }

    #[test]
    fn opener_is_strict_and_off_gcd_weaves_between_its_steps() {
        let build = Build {
            name: "open".into(),
            mode: Mode::Priority,
            gcd: Duration::from_millis(1000),
            min_gap: Duration::from_millis(15),
            opener: vec![1, 0],
            skills: vec![
                skill("Burst", "1", 8000, 1, false),
                skill("Opener", "2", 12000, 2, false),
                skill("Weave", "Q", 4000, 0, true),
            ],
            active: false,
        };
        let start = Instant::now();
        let mut engine = Engine::new(build, start);
        let presses = simulate(&mut engine, start, Duration::from_millis(1100));
        assert_eq!(at(&presses, 0), (0, 1));
        assert_eq!(at(&presses, 1), (15, 2));
        assert_eq!(at(&presses, 2), (1000, 0));
    }

    #[test]
    fn charges_fire_twice_then_wait_for_recharge() {
        let mut burst = skill("Burst", "1", 1000, 1, false);
        burst.charges = 2;
        let build = Build {
            name: "charges".into(),
            mode: Mode::Priority,
            gcd: Duration::ZERO,
            min_gap: Duration::from_millis(10),
            opener: vec![],
            skills: vec![burst],
            active: false,
        };
        let start = Instant::now();
        let mut engine = Engine::new(build, start);
        let presses = simulate(&mut engine, start, Duration::from_millis(1100));
        assert_eq!(at(&presses, 0), (0, 0));
        assert_eq!(at(&presses, 1), (10, 0));
        assert_eq!(at(&presses, 2), (1000, 0));
        assert!(presses.get(3).is_none());
    }
}
