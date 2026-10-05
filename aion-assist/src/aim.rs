//! When the rotation is allowed to run.
//!
//! A hold trigger is the button you keep down. An aim trigger is one pixel of
//! the marker the game draws while the reticle is on an enemy. The pixel is
//! taught once; this module only decides whether the sample still matches.

use std::fmt;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn near(self, other: Rgb, tolerance: u8) -> bool {
        self.r.abs_diff(other.r) <= tolerance
            && self.g.abs_diff(other.g) <= tolerance
            && self.b.abs_diff(other.b) <= tolerance
    }
}

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

pub fn parse_color(raw: &str) -> Result<Rgb, String> {
    let text = raw.trim();
    if text.is_empty() {
        return Err("aim color is empty".into());
    }
    if text.contains(',') {
        let parts: Vec<_> = text.split(',').map(str::trim).collect();
        if parts.len() != 3 {
            return Err(format!(
                "aim color \"{text}\" needs three numbers, as in 240,240,240"
            ));
        }
        let mut channels = [0u8; 3];
        for (index, part) in parts.iter().enumerate() {
            let value: u16 = part
                .parse()
                .map_err(|_| format!("aim color \"{text}\" has a value that is not a number"))?;
            if value > 255 {
                return Err(format!("aim color \"{text}\" has a value above 255"));
            }
            channels[index] = value as u8;
        }
        return Ok(Rgb {
            r: channels[0],
            g: channels[1],
            b: channels[2],
        });
    }

    let hex = text.strip_prefix('#').unwrap_or(text);
    if hex.len() != 6 || !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(format!(
            "aim color \"{text}\" should look like F2F2F2 or 242,242,242"
        ));
    }
    let value =
        u32::from_str_radix(hex, 16).map_err(|_| format!("aim color \"{text}\" is not hex"))?;
    Ok(Rgb {
        r: ((value >> 16) & 0xFF) as u8,
        g: ((value >> 8) & 0xFF) as u8,
        b: (value & 0xFF) as u8,
    })
}

/// Turns a noisy screen sample into a stable on/off.
///
/// The marker has to be there for 20 ms before the rotation starts, and gone
/// for 40 ms before it stops, so one odd frame does not cancel a cast.
#[derive(Clone, Debug)]
pub struct Gate {
    hit_since: Option<Instant>,
    miss_since: Option<Instant>,
    active: bool,
}

impl Gate {
    pub fn new() -> Self {
        Self {
            hit_since: None,
            miss_since: None,
            active: false,
        }
    }

    pub fn update(&mut self, hit: bool, now: Instant) -> bool {
        const START: Duration = Duration::from_millis(20);
        const STOP: Duration = Duration::from_millis(40);
        if hit {
            self.miss_since = None;
            match self.hit_since {
                Some(since) if now.saturating_duration_since(since) >= START => {
                    self.active = true;
                }
                Some(_) => {}
                None => self.hit_since = Some(now),
            }
        } else {
            self.hit_since = None;
            match self.miss_since {
                Some(since) if now.saturating_duration_since(since) >= STOP => {
                    self.active = false;
                }
                Some(_) => {}
                None => self.miss_since = Some(now),
            }
        }
        self.active
    }
}

impl Default for Gate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_and_decimal_colors() {
        assert_eq!(
            parse_color("#F2F2F0").unwrap(),
            Rgb {
                r: 0xF2,
                g: 0xF2,
                b: 0xF0
            }
        );
        assert_eq!(
            parse_color("240, 240, 240").unwrap(),
            Rgb {
                r: 240,
                g: 240,
                b: 240
            }
        );
        assert!(parse_color("nope").is_err());
    }

    #[test]
    fn tolerance_is_per_channel() {
        let marker = Rgb {
            r: 200,
            g: 200,
            b: 200,
        };
        assert!(marker.near(
            Rgb {
                r: 220,
                g: 190,
                b: 205
            },
            24
        ));
        assert!(!marker.near(
            Rgb {
                r: 230,
                g: 200,
                b: 200
            },
            24
        ));
    }

    #[test]
    fn a_single_frame_does_not_start_or_stop_the_rotation() {
        let start = Instant::now();
        let mut gate = Gate::new();
        assert!(!gate.update(true, start));
        assert!(!gate.update(true, start + Duration::from_millis(10)));
        assert!(gate.update(true, start + Duration::from_millis(20)));
        assert!(gate.update(false, start + Duration::from_millis(30)));
        assert!(!gate.update(false, start + Duration::from_millis(70)));
    }
}
