# Just Keep Scooping

A tiny incremental about garbage, capitalism, and a raccoon in a tiny tie.

You scoop a dumpster. You fill a bag. You sell the bag to **Uncle Gary**. You spend every coin on **The Scheme** — a branching upgrade tree that unlocks worse ideas the more you invest. Then you scoop again, except now the numbers are lying in a funnier font.

## Play on a Mac (or any desktop)

The game is a local web app. On macOS you only need Node:

```bash
cd just-keep-scooping
npm install
npm run dev
```

Open the URL Vite prints (usually `http://localhost:5174`). Click **I HAVE PAWS**. Safari, Chrome, and Arc are all fine. Progress is saved in that browser.

- **Space** or the orange button: scoop
- **S** or Uncle Gary: sell
- **T**: open The Scheme
- **M**: mute
- **F11** (Windows/Linux) or **Ctrl+⌘+F** (Mac): fullscreen
- Pet the alley cat. Kick the possum.

### Native Mac window (optional)

Same raccoon, in its own window instead of a tab. Needs [Rust](https://rustup.rs) and Xcode Command Line Tools (`xcode-select --install`):

```bash
npm run desktop
```

To stamp a `.app` + `.dmg` on that Mac:

```bash
npm run macos
```

The app lands at `src-tauri/target/release/bundle/macos/Just Keep Scooping.app` and the disk image at `src-tauri/target/release/bundle/dmg/`. Unsigned local builds: right-click → Open the first time, or `xattr -cr` the `.app`.

This Linux CI cannot bake a Mac binary; you build it on the Mac.

## Windows / Steam

Steam ships as a real Windows game, not a tab. Tauri wraps it as `Just Keep Scooping.exe` with an NSIS installer.

```bash
npm run windows
```

That needs a Windows box (or the GitHub Action `.github/workflows/just-keep-scooping-windows.yml`, which uploads the installer). Steam upload steps live in [STEAM.md](STEAM.md). You need a Steamworks App ID; you do not need paid art. A Mac build is for you to play and test — it is not the Steam depot.

## Why it gets sticky

- The Scheme is a big tree: spending on a node lights the children, spending enough coins unlocks whole branches (crew, empire, Dumpster God)
- Combos reward mashing, then selling at the peak
- Bottlecaps ride along for free and fill a bus ticket to the next district
- Interns start scooping for you; Uncle Gary Jr. sells when the bag pops
- Raccoon, dumpster, cat, and possum actually move (not Pixar, still a tiny tie)

## Assets

Nothing here is paid.

| Need | What this build uses |
| --- | --- |
| Art | Hand-drawn SVG raccoon, CSS dumpster / diner / neon |
| Sound | Tiny Web Audio synth (pops, coins, a sad possum thud) |
| Fonts | Google Fonts: Lilita One + Nunito |
| Music | None yet |
| Windows | Tauri 2 + WebView2 |
| macOS | Browser, or Tauri 2 + WKWebView (`.app` / `.dmg`) |

Store-page polish later: a music loop, a $5 SFX pack, one pixel pass. Optional.

## Tests

```bash
npm test
```
