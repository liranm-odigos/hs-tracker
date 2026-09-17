# Just Keep Scooping

A tiny incremental about garbage, capitalism, and a raccoon in a tiny tie.

You scoop a dumpster. You fill a bag. You sell the bag to **Uncle Gary**. You spend every coin on **The Scheme** — a branching upgrade tree that unlocks worse ideas the more you invest. Then you scoop again, except now the numbers are lying in a funnier font.

## Play in the browser

```bash
cd just-keep-scooping
npm install
npm run dev
```

Open the URL Vite prints (usually `http://localhost:5174`). Click **I HAVE PAWS**.

- **Space** or the orange button: scoop
- **S** or Uncle Gary: sell
- **T**: open The Scheme
- **M**: mute
- **F11**: fullscreen
- Pet the alley cat. Kick the possum.

Progress is saved in the browser.

## Windows / Steam

This is a real Windows game, not a tab. Tauri wraps it as `Just Keep Scooping.exe` with an NSIS installer.

```bash
npm run windows
```

That needs a Windows box (or the GitHub Action `.github/workflows/just-keep-scooping-windows.yml`, which uploads the installer). Steam upload steps live in [STEAM.md](STEAM.md). You need a Steamworks App ID; you do not need paid art.

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

Store-page polish later: a music loop, a $5 SFX pack, one pixel pass. Optional.

## Tests

```bash
npm test
```
