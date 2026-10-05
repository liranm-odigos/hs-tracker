# aion-assist

A Windows program that sends one skill rotation to the focused game window. Open it and a window appears. Builds, keys, cooldowns, the trigger, and the NPC check are edited there and saved into `rotation.toml`.

It does not read the game, inject code, or attach to the client. A press is a normal keyboard event, delivered with `SendInput` as a scan code. The key goes to whichever window is focused.

Automated combat input is commonly against the game's terms of service and can cost the account. The NPC check below does not change that. It does not hide the program from anti-cheat, and it does not make the account safe from a ban.

## Window

```text
cargo build --release
target\release\aion-assist.exe
```

With no options, the window opens. The first time, it starts from the example rotation and writes `rotation.toml` when you press Save. A file already next to the executable, or in the current directory, is loaded instead.

The window has three columns:

- **Builds.** Name, global cooldown, and priority or sequence. Start runs the selected build. "Mark active for CLI" is the build a command-line run uses.
- **Skills.** Key, cooldown, cast time, charges, priority, and whether it is in the opener. Up and Down change the list order. In priority mode the priority number decides which ready skill goes first. In sequence mode the list order is the rotation.
- **Trigger and NPC check.** How the rotation starts, and the player/NPC colors.

The command line is still there when you want it:

```text
aion-assist --list
aion-assist --dry-run --ms 8000
aion-assist rotation.toml
```

`--dry-run` prints the timeline and sends nothing. It also runs on Linux, which is how the rotation clock is tested. Live sending and the window's color capture are Windows only. Passing a file with no other options runs that file in the terminal, the same as before. `--ui` opens the window for a chosen file.

## Only against NPCs

The program cannot see a monster or a player. There is no documented color that always means one or the other, so it does not guess. You teach both colors from one pixel of the target-frame name.

1. Put the game in borderless windowed mode. Exclusive fullscreen hides the pixel.
2. Turn on **Only run against NPCs**. A new file starts with this on. Start stays off until both colors exist and can be told apart.
3. Target an NPC. Move this window aside so it does not cover the target frame. Put the cursor on the name and press **Capture NPC**. You have three seconds. That saves the pixel and the NPC color.
4. Target a player and leave the target frame in the same place. Press **Capture player**. That reads the same pixel, not wherever the cursor is now.
5. The live swatch shows what that pixel is right now: NPC, player, or unknown.

The rotation runs only when the pixel matches the NPC color. A player match, an unknown color, a missing sample, or two samples that are too close all block it. A tie is treated as a player. If you capture the NPC again at a different pixel, the player color is dropped and has to be taught again.

This is a screen-color heuristic. It fails closed, and it can still be wrong if the name color shifts, the frame moves, or another window covers that pixel. Turning the check off lets the rotation run on players too. Neither choice makes automated key presses allowed.

## When it presses keys

- **Hold.** The rotation runs while a button is down. `RButton` is the usual one, and the game still receives that button.
- **Aim.** The rotation runs while a taught marker is visible (the circle or the target bar). Capture it in the window, or run `aion-assist --learn-aim`, aim at a monster, put the cursor on the marker, and press `F8`. It starts about 20 ms after the marker is visible and stops about 40 ms after it disappears.
- **Toggle.** `F8` starts and stops. Windows swallows that key, so the game does not also receive it.

`F9` stops the run. The window's Stop button does the same. Keys are sent only while the focused window title contains the text in "Window title" (default `Aion`).

The NPC check is applied on top of whichever trigger you picked. The trigger can be true and the rotation still holds because the target looks like a player.

## Why the presses stay on time

The gap between skills is the cooldown you configured. What keeps the press itself on time:

- The send is one Win32 `SendInput` call. There is no macro program, no `PostMessage`, and no script host in the middle.
- The Windows timer is set to 1 ms while the program is running. Waits longer than 2 ms sleep, and the last 2 ms spin.
- The sender thread runs at above-normal priority.
- The key is held for the key-hold time (default 15 ms). Shorter than 5 ms is rejected. The next key never goes down before the previous one has come up.

## What a build means

Cooldowns are counted from the moment a key is sent. The game is not read, so a wrong number presses too early or too late.

- Priority sends the ready skill with the lowest priority number. A skill marked off the global cooldown may fire during it.
- Sequence walks the list in order and waits instead of skipping.
- The opener plays first, in the order the skills were marked, when every opener skill is ready. Off-GCD skills can weave while the next opener step waits.
- Charges are how many times a skill can be sent before its cooldown refills one charge.
- Cast time holds every skill, including weaves, until the cast finishes.
- A cooldown of 0 on a low-priority skill is a filler.

The window writes `rotation.toml`. Saving replaces the file, including any comments that were in it. The fields look like this:

```toml
trigger = "hold"
hold = "RButton"
toggle = "F8"
cancel = "F9"
key_hold_ms = 15
min_gap_ms = 15
focus = "Aion"
guard = true
guard_x = 960
guard_y = 140
npc_color = "E6E6E6"
player_color = "DC2828"
guard_tolerance = 32

[[builds]]
name = "example"
mode = "priority"
gcd_ms = 1000
active = true
opener = ["Opener", "Burst"]

[[builds.skills]]
name = "Weave"
key = "Q"
cooldown_ms = 4000
priority = 0
off_gcd = true
```
