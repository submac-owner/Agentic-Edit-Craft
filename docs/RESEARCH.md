# AgenticEditor — Base Editor Research

**Date:** 2026-11-16
**Goal:** a Windows desktop, professional-grade NLE with a chat side panel, where an
agent edits the *same* timeline as the user through the app's MCP server — and gets
higher-leverage tools than the human UI exposes.

**Hard constraint from the brief:** the editing core must be MIT or Apache-2.0 licensed.

All repo facts below were verified against the GitHub API on 2026-11-16, not taken from
blog posts or marketing pages. Statements about maturity come from each project's own docs
and are marked as such.

---

## 1. TL;DR

**Recommended core: [storytold/filmcraft](https://github.com/storytold/filmcraft).**

It is the only candidate that is simultaneously:

- **MIT OR Apache-2.0**, with a dependency allowlist that forbids GPL/LGPL,
- a **native Windows NLE** (MSI + portable, x64/arm64/x86),
- built around **a command registry** (650+ commands) where UI, CLI, control channel and
  **MCP server** all dispatch the same edits,
- shipped with an **agent automation contract** (MCP tools, automation ids, screenshots,
  render-frame feedback), and
- architected so the **front end is swappable** (`ui-egui` sits on top of the engine).

For the brief, that is not a close call: FilmCraft already *is* the "agent can take over the
editor" architecture. We extend it with (a) an agent side panel and (b) an elevated
"director" tool tier in the MCP server.

Secondary recommendation if you want a .NET/Avalonia codebase instead of Rust:
**Sprocket** (MIT) — but it bundles a **GPL-configured FFmpeg**, which is a licensing problem
for a closed-source product (§5).

---

## 2. Evaluation criteria

| # | Criterion | Why it matters here |
|---|---|---|
| 1 | License (MIT / Apache-2.0) and transitive license profile | The brief's hard constraint; GPL contamination is expensive to undo |
| 2 | Native Windows desktop, installable | Target platform |
| 3 | Real NLE data model (tracks, trims, keyframes, effects, audio) | "Professional editing", not trim-and-merge |
| 4 | Command/API surface the agent can drive | MCP tools must map to real edits, not screenshots-only automation |
| 5 | Undo/redo integration for agent edits | Agent mistakes must be one Ctrl+Z |
| 6 | Swappable / extensible UI | We add a side panel and our own chrome |
| 7 | Codec/format reality (H.264, HEVC, ProRes, 4K) | Pro workflows |
| 8 | Maturity, activity, community | Fork risk |
| 9 | Build cost on this machine (2 cores, 8 GB RAM) | Developer velocity |
| 10 | Trademark/branding constraints when forking | Shipping a product |

---

## 3. Verified candidate matrix

| Project | License (verified) | Stack | Windows app | Agent/MCP today | Stars | Maturity |
|---|---|---|---|---|---|---|
| **storytold/filmcraft** | **MIT OR Apache-2.0** | Rust, egui + wgpu, own codecs | **MSI + portable, signed** | **Yes — MCP server + JSON TCP control + CLI + UI automation** | 18.4k | alpha; self-reported ~56% "ready for real work" |
| **SprocketVideo/Sprocket** | **MIT** (but bundles GPL FFmpeg) | C# .NET 10, Avalonia 12, SkiaSharp | Installer + portable | Yes — MCP server (~65 tools) | 8 | alpha, feature-complete slice |
| **b-editor/beutl** | **MIT** (GPL FFmpeg worker in separate process) | C# / .NET 11, Vulkan | Yes | Agent terminal + CLI; no first-class MCP | 1.2k | active, compositing-focused |
| **OpenCut-app/OpenCut** | **MIT** | TypeScript/Next.js; Rust GPUI desktop shell | Desktop shell "just a window" | No engine command API | 98.6k | web editor usable; desktop very early |
| **Augani/openreel-video** | **MIT** | React/TS, WebGPU/WebCodecs/MediaBunny | Desktop alpha | Yes — MCP clients + bundled agent tools | 5.7k | alpha, browser-bounded |
| **elahlabs/elah** | **Apache-2.0** | TypeScript, WebGL2/WebCodecs, headless CLI | Web/Electron-class | Agent-oriented JSON project spec, no NLE-grade UI | 73 | young, engine-focused |
| **Clipkit** | Editor Apache-2.0, **runtime BSL-1.1** | React + WebGPU | Web embed | Advertises MCP use | n/a | **not fully open source** — avoid |
| **OpenTimelineIO** | **Apache-2.0** | C++/Python | library | n/a | 2k | mature interchange format |

### 3.1 FilmCraft — why it wins

Verified from the repo (README, `Cargo.toml`, `docs/agents.md`, `AGENTS.md`, releases):

- **Workspace license:** `license = "MIT OR Apache-2.0"` in `Cargo.toml`.
- **No FFmpeg inside.** H.264/HEVC/ProRes/AAC/Opus/VP8/VP9/AV1 decoders are first-party Rust.
  ffmpeg is used only as an external test oracle. This removes the single biggest legal
  trap in this product category (§5).
- **Dependency allowlist** (`AGENTS.md`): ISC, MIT, BSD-2/3, Apache-2.0, OFL, CC0, CC-BY,
  BSL-1.0, MPL-2.0 unmodified. No GPL/LGPL/AGPL.
- **Everything is a command:** `id`, label, menu path, shortcut, params, `enabled()`, `run()`.
  The UI, CLI, JSON control channel and MCP server all dispatch by id. 650+ commands.
- **MCP server already ships:** `filmcraft-cli mcp` (headless) and
  `filmcraft-cli mcp --bridge 127.0.0.1:9876` (drives the running desktop app).
  Tools: `command_list`, `command_run`, `command_batch`, `doc_inspect`, `project_inspect`,
  `sequence_inspect`, `media_import`, `render_frame`, plus bridge-only `ui_*` tools
  (inspect, elements, click, drag, key, type, screenshot).
- **Agent feedback loop is designed in:** render a frame → look at it → undo if wrong.
  Destructive calls can be gated by MCP hints (`readOnlyHint`, `destructiveHint`).
- **Time model is exact:** integer ticks (254,016,000,000/s), so 23.976/29.97 DF/59.94 never drift.
- **Swappable UI:** "The interface is one crate (`ui-egui`) over the engine, so a different
  front end can replace it without touching editing logic."
- **Release reality:** v0.20.1 published 2026-11-12 with `windows-x64.msi`,
  `windows-arm64.msi`, `windows-x64-portable.zip`, signed installers, and `filmcraft-cli`.

**Risks (state them honestly):**

1. **Young.** Repo created 2026-09-30; ~6 weeks old at time of writing. 18.4k stars and
   7.4k forks in that window is unusual velocity; 2,249 open issues. Treat "86% feature
   breadth / 56% ready for real work" as the vendor's own estimate, not a measurement.
2. **Windows/Linux get less testing than macOS** (their words).
3. **Alpha-grade reliability** on real camera media in the wild.
4. **Branding:** ArtCraft name/logo are trademarks; a fork **must** remove them
   (`docs/brand/LICENSE-brand.txt`).
5. **Build cost on this laptop:** 8 GB RAM, 2-core i5-7300U → first release build is likely
   slow (LTO thin, `codegen-units = 1`). Validate with the prebuilt MSI first.

### 3.2 Sprocket — best non-Rust alternative

- MIT, C#/.NET 10, Avalonia 12, SkiaSharp GPU compositing, FFmpeg 8 via hand-rolled P/Invoke.
- Genuinely impressive feature list: three-point editing, ripple/roll/slip/slide, multicam,
  proxies, keyframed effects, color grading + LUTs, loudness metering, MCP server (~65 tools),
  undoable AI edits through the same command stack.
- **Blocker:** release script downloads **`*-gpl-shared`** FFmpeg builds (for x264/x265).
  Their own README says the MIT app is "GPL-compatible", which is true for *them*; it means a
  distributed binary is effectively a GPL combined work. If you want a closed-source
  commercial app, this needs replacing with an LGPL-only FFmpeg build (losing x264/x265) or
  the FilmCraft approach.
- Community is tiny (8 stars, 2 forks) — bus factor 1.

### 3.3 Beutl — strong compositor, weaker NLE

- MIT, .NET 11, Vulkan, node graphs, keyframes, shaders (SKSL/GLSL), C# scripting, extensions,
  Git-based project versioning, and an in-app agent terminal.
- GPL FFmpeg is isolated in a **separate executable over IPC** — legally cleaner than linking.
- Less of a Premiere-style NLE (no three-point editing/trim-tool depth); more After
  Effects-like compositing. Good if the product pivots toward motion design.

### 3.4 OpenCut — biggest community, wrong engine for this

- 98.6k stars, MIT, excellent modern web UI.
- Desktop is **GPUI and explicitly "just a window that opens"**.
- No command registry / MCP / agent API; export and heavy media work are browser-bound.
- Good source of UI/UX inspiration, not a base for an agent-drivable pro NLE.

### 3.5 OpenReel — closest "agent-native" product, browser limits

- MIT, React/TS, WebGPU + WebCodecs + MediaBunny, desktop alpha, MCP client support and a
  bundled agent tool layer (advertised 275+ tools).
- Browser memory/codec ceilings, no native hardware encode pipeline without FFmpeg, and a
  young single-maintainer project.
- **Useful as a reference implementation** for agent tool taxonomy and approval UX.

### 3.6 elah — Apache-2.0 engine, not an editor

- Apache-2.0, browser-native engine, integer-frame time model, headless render CLI, 5 packages.
- Very early (73 stars) and no professional NLE surface. Could be a rendering target for
  generated motion, not the editing core.

### 3.7 Clipkit — read the fine print

- `@clipkit/editor` is Apache-2.0, the protocol is Apache-2.0, **but `@clipkit/runtime` is
  BSL-1.1**: free production only up to 250 rendered output-minutes/month, Apache after 4
  years. That fails the brief's "MIT or Apache" requirement for the part that actually
  renders. Exclude.

### 3.8 OpenTimelineIO — keep it, as a format

- Apache-2.0, industry-standard editorial interchange (FCPXML/AAF/EDL adapters).
- Not an editor, but the right **export/import contract** for our project file and for
  "hand the cut to Resolve/Premiere". FilmCraft already round-trips OTIO.

---

## 4. What this means architecturally

The brief has four separable parts:

1. **Editing engine** — timeline model, decode, composite, mix, export.
2. **Human UI** — professional panels, timeline, monitors, keyboard-first.
3. **Agent surface** — MCP server exposing app power, plus a chat side panel.
4. **Safety layer** — approvals, undo grouping, dry-run, audit log.

FilmCraft gives us (1) and a reference (2). We build (3) and (4).

### Option A — Fork FilmCraft, add an `Agent` panel (recommended)

```
┌──────────────────────── AgenticEditor (Rust, one window) ────────────────────────┐
│  Project │ Source │ Program │ Effect Controls │        AGENT PANEL              │
│  panel   │ monitor│ monitor │ / Color / Audio │  chat · plan · tool stream      │
│          │        │         │                 │  diff · approve · undo turn     │
├──────────────────────────── Timeline (multi-track) ──────────────────────────────┤
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
        ▲ engine commands (single source of truth, one undo stack)
        │
┌───────┴─────────┐      MCP (stdio)        ┌──────────────────────────┐
│ filmcraft-engine│◄───────────────────────►│ filmcraft-cli mcp        │
│  command registry│                        │  + director tools        │
└───────┬─────────┘                         └───────────┬──────────────┘
        │                                               │
        │ JSON-lines TCP (127.0.0.1, token)             │ external clients
        ▼                                               ▼
   UI automation / screenshots                 Claude Code, Cursor, our agent
```

- Add `crates/agent` (panel + session/plan state) and `crates/agent-tools` (composite commands).
- New commands are registered like every other command → automatically available to UI, CLI,
  MCP, and control channel.
- Chat runtime: an LLM loop that speaks MCP to the in-process engine (or to the bridge).
  Language choice for the loop is open (Rust crate, or a Node/Python sidecar over the
  control channel — see §7).

**Pros:** single window, one undo stack, native performance, no IPC gap for edits, direct
access to the command registry, best safety story.
**Cons:** Rust learning/build cost on this laptop; egui visual polish is our work; upstream
house rules (no Tauri/Electron in native apps) apply only if we want to upstream.

### Option B — Separate shell driving the prebuilt editor

A Tauri/React or .NET shell that starts `filmcraft.exe --control 9876` and drives it over the
token-authenticated JSON-lines protocol; the shell owns the chat panel window and docks next
to the editor.

- **Pros:** no Rust build; fastest UI iteration; prebuilt, signed editor binary.
- **Cons:** two windows (or fragile window docking), two process lifetimes, agent edits and
  user edits still share the engine's undo stack but the shell's own UI state is separate;
  the "agent panel" feels bolted on. Acceptable for a 2-week prototype, not the product.

### Option C — Fork Sprocket (.NET/Avalonia)

- **Pros:** C# is a gentler Windows stack; Avalonia theming is more capable than egui;
  mature packaging (Velopack, signed installers path).
- **Cons:** GPL FFmpeg bundle must be replaced; ~65-tool MCP is shallower than a command
  registry; tiny community; less "everything is a command" discipline.

### Option D — Build on a web editor (OpenCut/OpenReel/elah)

- **Pros:** best-looking UI, huge component ecosystem, agent tooling already present in
  OpenReel.
- **Cons:** browser memory/codec ceilings, no native timeline engine, no professional
  trim/multicam/audio depth; MCP tools would wrap a JS state store rather than a real engine.
  Wrong foundation for "professional".

**Verdict:** A for the product. B only as a throwaway validation harness.

---

## 5. Licensing landmines (read before committing)

1. **FFmpeg.** Core is LGPL-2.1+, but typical builds enable GPL (`--enable-gpl` for x264/x265)
   and become GPL. Bundling such a build in a distributed app makes the combined work
   GPL-tainted. If you must use FFmpeg: LGPL-only build, dynamic linking, relink rights,
   notices, no x264/x265. FilmCraft avoids this entirely.
2. **Codec patents.** H.264/HEVC/AAC are patent-encumbered regardless of source license.
   Open implementations do not grant patent licenses. Get counsel before commercial shipping.
3. **BSL "open source".** Clipkit's runtime is BSL-1.1: source-available, not OSI open source,
   with a production minute cap. Excluded by the brief.
4. **MPL-2.0** (e.g. Symphonia, used by FilmCraft for some audio decoding) is file-level
   copyleft: fine to use unmodified, but changes to those files must be published. FilmCraft's
   allowlist already permits it "used unmodified".
5. **Trademarks.** Forking FilmCraft requires removing ArtCraft marks and branding. Same for
   Sprocket/OpenCut names and logos.
6. **Model weights and stock assets** pulled in by AI features carry their own licenses
   (checkpoints, fonts, music) — keep them out of the MIT/Apache claim.
7. **Our own license:** MIT or Apache-2.0 for the app, with a NOTICE/THIRD-PARTY file that
   enumerates every bundled dependency and asset. Run `cargo-deny` (or `dotnet list package
   --include-transitive` + a license audit) in CI — do not rely on memory.

---

## 6. The agent surface: user tools vs. "director" tools

The brief's differentiator: the agent has **more powerful tools than the normal user**. That
should be concrete, not marketing.

**Tier 0 — inspection (read-only, auto-approved)**

| Tool | Purpose |
|---|---|
| `project.inspect` / `sequence.inspect` | Bins, items, tracks, clips, effects, markers, playhead |
| `command.list` | Every command with params and enabled state |
| `frames.grab` | Render N frames at timecodes → vision feedback |
| `media.analyze` | Codec/HDR/loudness/silence/shot boundaries/black frames |
| `transcript.get` / `transcript.search` | Word-level timestamps |
| `beats.get` | Music beat/downbeat grid |
| `timeline.diff` | Project delta vs. a checkpoint or turn |

**Tier 1 — atomic edits (same commands the UI calls; undoable)**

`command.run`, `command.batch`, plus explicit timeline/effect/keyframe/caption/marker/audio
commands. Nothing here is agent-only — it is exactly what a power user can do.

**Tier 2 — director tools (agent-only, composite, expensive, require an approval policy)**

| Tool | What it does | Why humans don't get it as one click |
|---|---|---|
| `edit.rough_cut` | Transcript + selects + target duration → cut list → applied timeline | Needs speech, scoring, and multi-step planning |
| `edit.montage` | Clips + music + BPM/style → beat-synced sequence | Solves an assignment problem over beat grid |
| `edit.remove_silence` | Gap detection with padding/handles, ripple | Deterministic batch over the whole sequence |
| `edit.multicam_sync` | Audio/visual alignment across angles → synced multicam | Signal correlation + track construction |
| `edit.reframe` | Any aspect ratio with subject tracking + safe areas | Per-frame analysis, not a keyframe slider |
| `color.match` | Match a clip's grade to a reference frame/shot | Histogram/tonal solve, then curve synthesis |
| `audio.duck` / `audio.conform` | Music ducking under dialogue; loudness to −14/−23 LUFS | Cross-track analysis + automation writing |
| `caption.generate` | Transcribe, segment, style, optionally burn in | ASR + typography + timing |
| `fx.recipe` | Named, parameterized effect stacks applied consistently | Recipes are curated graphs |
| `delivery.variants` | One timeline → 16:9 + 9:16 + 1:1 renders with per-aspect reframes | Fan-out render + crop solving |
| `qc.check` | Offline media, flash frames, black holes, loudness, caption overflow | Rule engine over the whole timeline |
| `review.self_critique` | Render key frames, score against a brief, propose fixes | Vision loop + rubric |
| `history.checkpoint` / `history.restore` | Named restore points per agent turn | Session-level transactionality |

**Safety design (non-negotiable):**

- Every agent turn is **one undo group**; "Undo last agent turn" is a first-class button.
- Destructive/composite tools are `dryRun: true` by default → returns a **plan + diff**;
  applying requires an explicit approval (UI button or `approved: true`).
- MCP hints (`readOnlyHint` / `destructiveHint`) so external clients can auto-approve reads.
- Localhost-only MCP, per-session token, no filesystem confinement claims we can't keep.
- Full audit log: tool, params, diff, frame grabs, model, token usage, duration.
- Never let the agent silently overwrite media files; project edits only.

---

## 7. Side panel runtime options

| Option | Where the LLM loop runs | Pros | Cons |
|---|---|---|---|
| **Rust in-process** (`crates/agent`) | Inside the app | One process, direct engine access, no IPC | LLM/tooling ecosystem is thinner; slower iteration |
| **Node sidecar** | `node agent-host.js`, talks MCP to the app | Best SDKs (Anthropic/OpenAI/MCP), streaming, fast iteration | Extra process to ship; IPC boundary |
| **Python sidecar** | same | ML ecosystem (Whisper, PySceneDetect, OpenCV) | Packaging heavier |
| **External client** | Claude Code / Cursor via MCP | Zero agent code | No in-app panel, no product-owned UX |

Pragmatic recommendation: **panel in Rust (egui), LLM loop in a Node sidecar** spoken to over
the existing control channel + MCP. It keeps the editor native and single-window while letting
the agent layer iterate daily. Read-only analysis (Whisper, scene detection) can live in the
same sidecar or a second Python worker.

---

## 8. Local environment findings (this machine)

| Item | Value | Consequence |
|---|---|---|
| CPU | Intel i5-7300U (2 cores / 4 threads) | Rust release builds will be slow |
| RAM | 7.9 GB | 8 GB is tight for a big Rust link step; keep a large page file, close browsers during builds |
| GPU | Intel HD Graphics 620 | DX12-capable but weak; FilmCraft defaults to Vulkan/DX12, so validate playback before committing |
| git | 2.56 | OK |
| node / npm | 26.13 / 11.20 | Good for the agent sidecar and web prototypes |
| python | 3.15 | Good for ML/analysis workers |
| Rust | **missing** | `rustup` install needed for Option A |
| .NET | **missing** | Needed only for Option C |

**Practical path:** install the prebuilt **FilmCraft 0.20.1 Windows x64 MSI**, run
`filmcraft.exe --control 9876`, drive it with a 30-line Node script through the control
channel, and confirm four things on this hardware: (1) it launches on HD 620, (2) 1080p
H.264 playback is usable, (3) MCP `command_run` edits land, (4) `render_frame` round-trips.
Only then spend hours on a Rust toolchain + first build.

### 8.1 Smoke test — done, passed

Run on 2026-11-16 with the signed 0.20.1 portable build (SHA-256 verified). Details and
reproduction commands: [`SMOKE-TEST.md`](SMOKE-TEST.md). Headline numbers:

| Check | Result |
|---|---|
| Command registry | 718 commands |
| GUI on Intel HD 620 | 42.0 fps with the demo timeline |
| Cold start → control channel ready | 0.8 s |
| MCP bridge | 17 tools, protocol 2025-06-18 |
| Agent edit + undo | razor → `{"cuts":3}`, 7→8 clips; undo → 7 clips |
| Vision loop | `render_frame` returns `image/png` |
| H.264 export | 540p, 12.2 fps render, 9.9 s for 5 s, H.264 High + AAC 48 kHz |

**Conclusion:** the base is real and works on this hardware. Export is CPU-bound here
(no NVENC on HD 620); use proxies, and benchmark 1080p before promising delivery speed.

---

## 9. Decision to make

1. **Base:** FilmCraft (recommended) / Sprocket / Beutl / web-based.
2. **Integration:** fork and add an in-app agent panel (Option A) or companion shell (Option B).
3. **Agent runtime:** Rust in-process or Node sidecar.
4. **Scope of v1:** which Tier-2 tools ship first (suggested: `edit.rough_cut`,
   `edit.remove_silence`, `delivery.variants`, `qc.check` — highest value / lowest risk).

## 10. Next steps (once base is chosen)

1. Install the prebuilt FilmCraft MSI; run the control-channel smoke test on this laptop.
2. Fork (remove ArtCraft branding), pin the upstream commit, add `THIRD-PARTY-NOTICES`.
3. Prototype the Agent panel: chat transcript, tool-call stream, plan/diff, approve, undo turn.
4. Implement Tier-2 tools as engine commands (so they are automatically on MCP + CLI).
5. Add the safety layer: per-turn undo grouping, dry-run default, audit log.
6. Golden-path test: import 3 clips → agent rough cut → user trims → export 16:9 + 9:16.
7. Package: MSI + portable, code-signing, auto-update.

---

### Appendix: verified links

- FilmCraft — https://github.com/storytold/filmcraft (Apache-2.0 on GitHub; MIT OR Apache-2.0 in repo)
- FilmCraft agent docs — https://github.com/storytold/filmcraft/blob/main/docs/agents.md
- FilmCraft releases — https://github.com/storytold/filmcraft/releases (v0.20.1, 2026-11-12)
- Sprocket — https://github.com/SprocketVideo/Sprocket (MIT)
- Beutl — https://github.com/b-editor/beutl (MIT)
- OpenCut — https://github.com/OpenCut-app/OpenCut (MIT)
- OpenReel — https://github.com/Augani/openreel-video (MIT)
- elah — https://github.com/elahlabs/elah (Apache-2.0)
- OpenTimelineIO — https://github.com/AcademySoftwareFoundation/OpenTimelineIO (Apache-2.0)
- Clipkit licensing split — https://www.clipkit.dev/open-source-video-editor (editor Apache-2.0 / runtime BSL-1.1)