# Shipping Just Keep Scooping on Steam (Windows)

This folder is the Steam-facing wrapper around the web game. The Windows build is a **Tauri** app: a real `.exe`, an NSIS installer, WebView2, and a `steam_appid.txt` next to the binary.

You do **not** need paid art to ship. You **do** need a Steamworks partner account and an App ID (Valve's fee, not an asset store).

## 1. Get an App ID

1. Join Steamworks and create the app **Just Keep Scooping**.
2. Copy the numeric App ID.
3. Put it alone in `src-tauri/steam_appid.txt` (replace the `0`).
4. Rebuild. Steam overlay and identity only work with a real ID. `0` means "skip Steam" so you can still play the installer.

Spacewar (`480`) is Valve's test ID if you want overlay before your own app exists.

## 2. Build the Windows installer

On a Windows machine, or via GitHub Actions (`just-keep-scooping-windows.yml`):

```bash
cd just-keep-scooping
npm ci
npx tauri build --bundles nsis
```

The installer lands at:

`src-tauri/target/release/bundle/nsis/Just Keep Scooping_*_x64-setup.exe`

Steam can also take a **loose folder** instead of the installer. After a build, the payload is:

`src-tauri/target/release/` plus `Just Keep Scooping.exe`, WebView2 bits the bundle already handles, and `steam_appid.txt`.

The GitHub workflow uploads the NSIS installer as an artifact named `just-keep-scooping-windows`.

## 3. Steamworks depot

Typical layout:

| Depot | Contents |
| --- | --- |
| Windows | Extracted install dir: `Just Keep Scooping.exe`, resources, `steam_appid.txt` |

Launch option:

```
Just Keep Scooping.exe
```

Install script: none required if you upload the folder. If you prefer the NSIS installer as redistributable for friends, keep it off the Steam depot — Steam should launch the exe directly.

## 4. What this build already has

- Native window (1280×800, resizable, F11 fullscreen in-game)
- Local saves (browser storage inside WebView2, per Windows user)
- Achievements in-game (codex trophies). Mapping them to Steam stats is a follow-up once the App ID exists.
- No extra paid runtime. Players on Windows 10/11 already have WebView2; the NSIS installer bootstraps it if someone is missing it.

## 5. Follow-ups after the App ID

- Steam Cloud for `localStorage` / a JSON save in `%APPDATA%`
- Steam achievements hooked to the in-game trophy ids
- Linux / Steam Deck build (WebView2 + Proton is shaky; a Linux Tauri bundle is the right Deck path)

None of those block a first Windows depot.
