# aion-assist

A Windows program that sends one skill rotation to the focused game window. You describe the build in a text file: keys, cooldowns, charges, an opener, and either a priority list or a fixed sequence. The program keeps those timers and presses the next key when it is due.

It does not read the game, inject code, or attach to the client. A press is a normal keyboard event, delivered with `SendInput` as a scan code, which is what games read from the keyboard. The key goes to whichever window is focused. Set `focus` if you want it to wait until the window title contains that text, so a rotation cannot type into chat by mistake.

Automated combat input is commonly against the game's terms of service and can cost the account. This tool does not hide itself from anti-cheat.

## Why this is fast

The gap between skills is the cooldown you configured, not the language. What keeps the press itself on time:

- The send is one Win32 `SendInput` call. There is no macro program, no `PostMessage`, and no script host in the middle.
- The Windows timer is set to 1 ms while the program is running. Waits longer than 2 ms sleep, and the last 2 ms spin, so the press is not late by a scheduler tick.
- The sender thread runs at above-normal priority.
- The hot path does not print, read a file, or allocate a rotation. Pass `--verbose` only while you are checking a build.
- The key is held for `key_hold_ms` (default 15). Shorter than 5 ms is rejected because clients drop taps that short. The next key never goes down before the previous one has come up.

Rust is used because that send path is a direct call with no runtime between the timer and `SendInput`, and the rotation clock can be tested on its own.

## Build

Install Rust on Windows, then from this directory:

```text
cargo build --release
```

The executable is `target\release\aion-assist.exe`. Copy `rotation.example.toml` next to it as `rotation.toml` and edit it.

## Run

```text
aion-assist
aion-assist --list
aion-assist --dry-run --ms 8000
aion-assist --build sequence-example --dry-run
```

Click the game, then press the toggle key (`F8` in the example) to start and stop. Press the cancel key (`F9`) to quit. The toggle is registered with Windows, so the game does not also receive it.

`--dry-run` prints the timeline and sends nothing. It also runs on Linux, which is how the rotation clock is tested. Live sending is Windows only.

## Rotation file

```toml
toggle = "F8"
cancel = "F9"
key_hold_ms = 15
min_gap_ms = 15
focus = "Aion"

[[builds]]
name = "example"
mode = "priority"    # or "sequence"
gcd_ms = 1000
active = true
opener = ["Opener", "Burst"]

[[builds.skills]]
name = "Weave"
key = "Q"
cooldown_ms = 4000
priority = 0         # lower number wins
off_gcd = true

[[builds.skills]]
name = "Burst"
key = "Shift+1"
cooldown_ms = 8000
cast_ms = 0
charges = 2
priority = 1
```

- `mode = "priority"` sends the ready skill with the lowest priority number. A skill with `off_gcd = true` may fire during the global cooldown.
- `mode = "sequence"` walks the list in order and waits for the next skill instead of skipping it.
- `opener` is played first, in order. Off-GCD skills can weave while the next opener step is waiting on the global cooldown.
- `charges` is how many times a skill can be sent before `cooldown_ms` refills one charge.
- `cast_ms` holds every skill, including weaves, until the cast finishes.
- `cooldown_ms = 0` on a low-priority skill is a filler: it is sent when the global cooldown is up and nothing better is ready.

Keys are written as `1`, `Q`, `F8`, `Space`, `Numpad3`, or with modifiers in front: `Shift+1`, `Ctrl+Q`, `Alt+E`. The Windows key is rejected.
