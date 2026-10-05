//! Decide whether the aimed target looks like an NPC or a player.
//!
//! The decision is one pixel of the target frame, compared with an NPC color
//! and a player color the user captured. Anything that is not clearly the NPC
//! blocks the rotation.

use crate::aim::Rgb;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Guard {
    pub enabled: bool,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub npc: Option<Rgb>,
    pub player: Option<Rgb>,
    pub tolerance: u8,
}

impl Default for Guard {
    fn default() -> Self {
        Self {
            enabled: false,
            x: None,
            y: None,
            npc: None,
            player: None,
            tolerance: 32,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Npc,
    Player,
    Unknown,
    Untaught,
}

impl Guard {
    pub fn ready(&self) -> Result<(), &'static str> {
        if !self.enabled {
            return Ok(());
        }
        if self.x.is_none() || self.y.is_none() || self.npc.is_none() || self.player.is_none() {
            return Err("Teach an NPC and a player on the target name before the rotation can run.");
        }
        if !self.samples_differ() {
            return Err(
                "The NPC and player colors are too close to tell apart. Capture them again on the target name.",
            );
        }
        Ok(())
    }

    pub fn samples_differ(&self) -> bool {
        match (self.npc, self.player) {
            (Some(npc), Some(player)) => !npc.near(player, self.tolerance),
            _ => false,
        }
    }

    pub fn verdict(&self, sample: Option<Rgb>) -> Verdict {
        if self.ready().is_err() {
            return Verdict::Untaught;
        }
        let (Some(npc), Some(player)) = (self.npc, self.player) else {
            return Verdict::Untaught;
        };
        let Some(sample) = sample else {
            return Verdict::Unknown;
        };
        let npc_hit = sample.near(npc, self.tolerance);
        let player_hit = sample.near(player, self.tolerance);
        match (npc_hit, player_hit) {
            (true, false) => Verdict::Npc,
            (false, true) => Verdict::Player,
            (true, true) if channel_distance(sample, player) <= channel_distance(sample, npc) => {
                Verdict::Player
            }
            (true, true) => Verdict::Npc,
            (false, false) => Verdict::Unknown,
        }
    }
}

fn channel_distance(left: Rgb, right: Rgb) -> u16 {
    u16::from(left.r.abs_diff(right.r))
        + u16::from(left.g.abs_diff(right.g))
        + u16::from(left.b.abs_diff(right.b))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn color(r: u8, g: u8, b: u8) -> Rgb {
        Rgb { r, g, b }
    }

    fn taught() -> Guard {
        Guard {
            enabled: true,
            x: Some(100),
            y: Some(40),
            npc: Some(color(220, 220, 220)),
            player: Some(color(220, 40, 40)),
            tolerance: 24,
        }
    }

    #[test]
    fn only_a_clear_npc_is_allowed() {
        let guard = taught();
        assert_eq!(guard.verdict(Some(color(210, 215, 225))), Verdict::Npc);
        assert_eq!(guard.verdict(Some(color(230, 50, 45))), Verdict::Player);
        assert_eq!(guard.verdict(Some(color(20, 20, 20))), Verdict::Unknown);
        assert_eq!(guard.verdict(None), Verdict::Unknown);
    }

    #[test]
    fn a_tie_blocks_as_a_player() {
        let mut guard = taught();
        guard.npc = Some(color(100, 100, 100));
        guard.player = Some(color(140, 100, 100));
        guard.tolerance = 24;
        assert!(guard.ready().is_ok());
        assert_eq!(guard.verdict(Some(color(120, 100, 100))), Verdict::Player);
    }

    #[test]
    fn missing_or_similar_samples_are_not_ready() {
        let mut guard = taught();
        guard.player = None;
        assert!(guard.ready().is_err());
        assert_eq!(guard.verdict(Some(color(220, 220, 220))), Verdict::Untaught);

        let mut guard = taught();
        guard.player = Some(color(225, 225, 225));
        assert!(guard.ready().is_err());
    }

    #[test]
    fn a_disabled_guard_does_not_need_samples() {
        let guard = Guard::default();
        assert!(guard.ready().is_ok());
    }
}
