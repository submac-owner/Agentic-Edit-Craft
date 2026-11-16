# FilmCraft smoke test — 2026-11-16

Purpose: prove the chosen base is real and usable **on this machine** before forking it.

Machine: Intel i5-7300U (2C/4T), 7.9 GB RAM, Intel HD Graphics 620,
Windows, Node 26.13, no Rust toolchain installed.

Build under test: **FilmCraft 0.20.1**, `filmcraft-0.20.1-windows-x64-portable.zip`
(108.2 MB, SHA-256 `bfb00e2089df5f13e6f76aab117a4572a8d005d009c5818989688b140a502710`,
matches the published `SHA256SUMS.txt`).

Artifacts are in `vendor/` (git-ignored) and `vendor/out/`.

---

## Results

| Check | Result |
|---|---|
| Portable package unpacks and runs | ✅ |
| `filmcraft-cli --version` | ✅ `0.20.1` |
| Headless demo project inspect | ✅ 0.24 s, 1,576 lines of JSON |
| Command registry | ✅ **718 commands** |
| Headless 1080p frame render | ✅ 1920×1080 in **241.6 ms** (1.08 s wall incl. process start) |
| GUI launches on HD 620 | ✅ window up, demo project loaded |
| UI refresh rate with demo timeline | ✅ **42.0 fps** (`ui.inspect`) |
| Cold start → control channel accepting connections | ✅ **0.8 s** |
| MCP bridge | ✅ 17 tools, protocol `2025-06-18`, server `filmcraft 0.20.1` |
| Agent edits timeline | ✅ `timeline.razor` → `{"cuts": 3}`, clips 7 → 8 |
| Undo restores the edit | ✅ `{"undone": "Razor"}`, clips back to 7 |
| Vision feedback | ✅ `render_frame` returns `image/png` (125 KB) inside MCP |
| H.264 export from the live session | ✅ 120 frames @ 960×540, **12.2 render fps**, 9.9 s, 2.73 MB, H.264 High + AAC 48 kHz, BT.709 |

### MCP tools present today (17)

`command_batch`, `command_list`, `command_run`, `doc_inspect`, `media_import`,
`project_inspect`, `render_frame`, `render_preview`, `sequence_inspect`,
`ui_click`, `ui_control`, `ui_drag`, `ui_elements`, `ui_inspect`, `ui_key`,
`ui_screenshot`, `ui_type`.

Read-only vs destructive is already hinted per tool, and `command_run` edits are undoable.

### Data model notes (from `sequence.inspect`)

- Top level: `audio`, `video`, `duration`, `durationFrames`, `id`, `in`, `out`, `markers`,
  `name`, `playhead`, `selection`, `settings`, `split`, `gapSelection`, `transitionSelection`.
- Tracks have `items` (not `clips`), each item carrying `clip`, `item`, `start`, `end`,
  `sourceIn`, `sourceOut`, `duration`, `durationFrames`, `speed`, `reverse`, `gainDb`, `link`.
- Time is integer ticks (254,016,000,000/s). `playhead.set {seconds: 3}` returned
  `752215464000` ticks ≈ 2.961 s, i.e. snapped to the sequence frame grid.

---

## Gaps this exposes (input for our fork)

1. **Only 17 generic MCP tools.** The brief wants "more powerful tools than the user".
   That is our Tier-2 director layer (`edit.rough_cut`, `edit.montage`, `edit.remove_silence`,
   `color.match`, `delivery.variants`, `qc.check`, …) — nothing to remove, only to add.
2. **`render_frame` in bridge mode returns the Program monitor with UI chrome** (scaled
   screenshot), while headless mode renders a clean frame. For agent vision we want a clean,
   uncropped frame at sequence resolution by default.
3. **No agent panel in the UI.** We add it (egui panel + agent runtime).
4. **Export is CPU-bound on this GPU.** HD 620 has no NVENC and FilmCraft's hardware H.264
   path on Windows is NVENC-only. 540p ≈ 12 fps; expect 1080p ≈ 3–4 fps. Proxies and
   background jobs matter; a faster GPU box is recommended for real work.
5. **No log file was written** under `%APPDATA%\FilmCraft\Logs` during a clean run
   (the README says it exists). Not a blocker, but worth checking on a crashed/GPU-fallback run.
6. **Schema doc:** `video`/`audio` + `items` naming is undocumented for consumers; our MCP
   tool descriptions should state it.

---

## Reproduction

```powershell
# 1. Verify and unpack
Invoke-WebRequest https://github.com/storytold/filmcraft/releases/download/v0.20.1/SHA256SUMS.txt -OutFile vendor\SHA256SUMS.txt
Invoke-WebRequest https://github.com/storytold/filmcraft/releases/download/v0.20.1/filmcraft-0.20.1-windows-x64-portable.zip -OutFile vendor\filmcraft.zip
(Get-FileHash vendor\filmcraft.zip -Algorithm SHA256).Hash   # bfb00e20...
tar -xf vendor\filmcraft.zip -C vendor\filmcraft

# 2. Headless checks
$fc = "vendor\filmcraft\filmcraft-0.20.1-windows-x64-portable\filmcraft-cli.exe"
& $fc --version
& $fc --demo inspect sequence
& $fc --demo render --seconds 2 --out vendor\out\demo-frame.png

# 3. Live app + control channel (keep the same token for the shell and the client)
$env:FILMCRAFT_CONTROL_TOKEN = -join ((1..64) | % { '{0:x}' -f (Get-Random -Max 16) })
Start-Process "vendor\filmcraft\filmcraft-0.20.1-windows-x64-portable\filmcraft.exe" -ArgumentList '--control','9876'
node scripts\filmcraft-control.mjs smoke
node scripts\mcp-smoke.mjs

# 4. Agent edit + undo + export through the live app
& $fc --bridge 127.0.0.1:9876 exec timeline.razor seconds=4
& $fc --bridge 127.0.0.1:9876 exec edit.undo
& $fc --bridge 127.0.0.1:9876 export vendor\out\smoke-export.mp4 --range custom --start 0 --end 5 --scale 0.5
```

Note: the control channel requires `FILMCRAFT_CONTROL_TOKEN` to be identical in the app
process and every client (CLI bridge, MCP bridge, raw socket).
