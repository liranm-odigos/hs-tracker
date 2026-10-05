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

`--dry-run` prints the timeline and sends nothing. It also runs on Linux, which is how the rotation clock is tested. Live sending is Windows only.

## Add a rotation

1. Copy `rotation.example.toml` to `rotation.toml` next to the executable.
2. Set `gcd_ms` to your global cooldown.
3. Give each skill its own `[[builds.skills]]` block. `key` is what the game has bound. `cooldown_ms` is that skill's cooldown. `priority` is the order: 0 is first.
4. List the opening skills in `opener`, in the order you press them on a fresh pull.
5. Mark the build you want with `active = true`. `--build name` selects another one. `--list` shows them.

A second build is another `[[builds]]` block with its own skills. Cooldowns are counted from the moment a key is sent. The game is not read, so a wrong number presses too early or too late.

Check the order before you send anything:

```text
aion-assist --dry-run --ms 8000
```

## Start when you aim at a monster

The program cannot see a monster. It can watch one pixel of the marker the game draws while your reticle is on an enemy (the circle around it, or the target bar).

1. In the game, set the reticle to hostile targets, and use borderless windowed mode. Exclusive fullscreen does not share that pixel.
2. Aim at a monster so the marker is visible. Show the cursor, put it on the marker, and run:

```text
aion-assist --learn-aim
```

3. Press `F8`. That writes `trigger = "aim"` plus the pixel into `rotation.toml`.
4. Run `aion-assist`. The rotation starts about 20 ms after that marker is visible, and stops about 40 ms after it disappears. `F9` quits.

It cannot tell a monster from a player when both use the same marker. In a PvE area the marker you taught is the monster under the reticle.

`trigger = "hold"` and `hold = "RButton"` is the other trigger: the rotation runs only while that button is down, and the game still receives the button. Use it when you want a button instead of the aim marker.

`trigger = "toggle"` keeps the old behavior: `F8` starts and stops. Windows swallows that key, so the game does not also receive it.

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
