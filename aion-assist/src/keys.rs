//! Key names from the rotation file, mapped to Windows virtual-key codes.
//!
//! The codes are the stable Win32 values. Live sending turns them into scan
//! codes at the moment of the press.

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyCombo {
    pub label: String,
    pub vk: u16,
    pub extended: bool,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MainKey {
    vk: u16,
    extended: bool,
}

pub fn parse(raw: &str) -> Result<KeyCombo, String> {
    let label = raw.trim();
    if label.is_empty() {
        return Err("key is empty".into());
    }

    let mut shift = false;
    let mut ctrl = false;
    let mut alt = false;
    let mut main: Option<MainKey> = None;

    for token in label.split('+') {
        let piece = token.trim();
        if piece.is_empty() {
            return Err(format!("key \"{label}\" has an empty piece around '+'"));
        }
        if is_windows_key(piece) {
            return Err(format!(
                "key \"{label}\" uses the Windows key, which opens the Start menu"
            ));
        }
        match modifier(piece) {
            Some(Modifier::Shift) if shift => {
                return Err(format!("key \"{label}\" repeats Shift"));
            }
            Some(Modifier::Ctrl) if ctrl => {
                return Err(format!("key \"{label}\" repeats Ctrl"));
            }
            Some(Modifier::Alt) if alt => {
                return Err(format!("key \"{label}\" repeats Alt"));
            }
            Some(Modifier::Shift) => shift = true,
            Some(Modifier::Ctrl) => ctrl = true,
            Some(Modifier::Alt) => alt = true,
            None => {
                if main.is_some() {
                    return Err(format!(
                        "key \"{label}\" has more than one key; put modifiers first, as in Shift+1"
                    ));
                }
                main = Some(
                    main_key(piece)
                        .ok_or_else(|| format!("key \"{label}\" has unknown key \"{piece}\""))?,
                );
            }
        }
    }

    let Some(main) = main else {
        return Err(format!("key \"{label}\" is only modifiers"));
    };

    Ok(KeyCombo {
        label: label.to_string(),
        vk: main.vk,
        extended: main.extended,
        shift,
        ctrl,
        alt,
    })
}

enum Modifier {
    Shift,
    Ctrl,
    Alt,
}

fn modifier(token: &str) -> Option<Modifier> {
    match token.to_ascii_lowercase().as_str() {
        "shift" => Some(Modifier::Shift),
        "ctrl" | "control" => Some(Modifier::Ctrl),
        "alt" => Some(Modifier::Alt),
        _ => None,
    }
}

fn is_windows_key(token: &str) -> bool {
    matches!(
        token.to_ascii_lowercase().as_str(),
        "win" | "windows" | "super" | "meta" | "lwin" | "rwin"
    )
}

fn main_key(token: &str) -> Option<MainKey> {
    let lower = token.to_ascii_lowercase();
    let named = match lower.as_str() {
        "space" => Some((0x20, false)),
        "tab" => Some((0x09, false)),
        "esc" | "escape" => Some((0x1B, false)),
        "enter" | "return" => Some((0x0D, false)),
        "backspace" => Some((0x08, false)),
        "delete" | "del" => Some((0x2E, true)),
        "insert" | "ins" => Some((0x2D, true)),
        "home" => Some((0x24, true)),
        "end" => Some((0x23, true)),
        "pageup" | "pgup" => Some((0x21, true)),
        "pagedown" | "pgdn" => Some((0x22, true)),
        "left" => Some((0x25, true)),
        "up" => Some((0x26, true)),
        "right" => Some((0x27, true)),
        "down" => Some((0x28, true)),
        "minus" | "-" => Some((0xBD, false)),
        "equals" | "=" | "plus" => Some((0xBB, false)),
        "comma" | "," => Some((0xBC, false)),
        "period" | "." => Some((0xBE, false)),
        "slash" | "/" => Some((0xBF, false)),
        "semicolon" | ";" => Some((0xBA, false)),
        "quote" | "'" => Some((0xDE, false)),
        "lbracket" | "[" => Some((0xDB, false)),
        "rbracket" | "]" => Some((0xDD, false)),
        "backslash" | "\\" => Some((0xDC, false)),
        "grave" | "backquote" | "`" => Some((0xC0, false)),
        "numpadadd" | "num+" => Some((0x6B, false)),
        "numpadsub" | "num-" => Some((0x6D, false)),
        _ => None,
    };
    if let Some((vk, extended)) = named {
        return Some(MainKey { vk, extended });
    }

    if let Some(rest) = lower
        .strip_prefix("numpad")
        .or_else(|| lower.strip_prefix("num"))
    {
        if let Ok(n) = rest.parse::<u8>() {
            if n <= 9 {
                return Some(MainKey {
                    vk: 0x60 + u16::from(n),
                    extended: false,
                });
            }
        }
    }

    if let Some(rest) = lower.strip_prefix('f') {
        if let Ok(n) = rest.parse::<u8>() {
            if (1..=24).contains(&n) {
                return Some(MainKey {
                    vk: 0x70 + u16::from(n - 1),
                    extended: false,
                });
            }
        }
    }

    let mut chars = lower.chars();
    let ch = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    if ch.is_ascii_digit() {
        return Some(MainKey {
            vk: ch as u16,
            extended: false,
        });
    }
    if ch.is_ascii_alphabetic() {
        return Some(MainKey {
            vk: ch.to_ascii_uppercase() as u16,
            extended: false,
        });
    }
    None
}

impl fmt::Display for KeyCombo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_letter_digit_and_function_keys() {
        let q = parse("q").unwrap();
        assert_eq!(q.vk, b'Q' as u16);
        assert!(!q.shift && !q.ctrl && !q.alt);

        let one = parse("1").unwrap();
        assert_eq!(one.vk, b'1' as u16);

        let f8 = parse("F8").unwrap();
        assert_eq!(f8.vk, 0x77);
    }

    #[test]
    fn parses_modifiers_in_any_modifier_order() {
        let key = parse("ctrl+shift+1").unwrap();
        assert!(key.ctrl && key.shift && !key.alt);
        assert_eq!(key.vk, b'1' as u16);
        assert_eq!(key.label, "ctrl+shift+1");
    }

    #[test]
    fn rejects_windows_key_duplicate_modifiers_and_unknown_names() {
        assert!(parse("win+1").is_err());
        assert!(parse("shift+shift+1").is_err());
        assert!(parse("shift").is_err());
        assert!(parse("not-a-key").is_err());
        assert!(parse("1+Q").is_err());
    }

    #[test]
    fn marks_navigation_keys_extended() {
        assert!(parse("left").unwrap().extended);
        assert!(!parse("space").unwrap().extended);
        assert_eq!(parse("numpad3").unwrap().vk, 0x63);
    }
}
