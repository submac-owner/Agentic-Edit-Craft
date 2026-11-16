# AGENTS.md — rules for everyone working in this repository

Agentic Edit Craft is a Windows desktop NLE where a person and an agent edit the same timeline.
It is a fork of [FilmCraft](https://github.com/storytold/filmcraft); see `FORK-NOTICE.md`.

These rules apply to human and AI contributors equally. When this file and any other document
disagree, this file wins. Keep it short enough that every contributor can read it.

## 1. Builds happen in GitHub Actions

- `.github/workflows/build.yml` is the only build path that counts. A change is not done until that
  workflow is green on `main`.
- Do not report a local build as evidence. If you could not run a build, say so and rely on CI.
- The Windows artifact is `agentic-edit-craft-windows-x64` (contains `filmcraft.exe` and
  `filmcraft-cli.exe`).
- Do not add CI that calls another repository's workflows. This fork is self-contained.

## 2. Never crash

A crash loses hours of editing work, so this rule outranks every feature.

- Return `Result<T, E>` and propagate with `?`; add context instead of discarding it.
- No `unwrap()`, `expect()`, `panic!`, `unreachable!`, `todo!` or `unimplemented!` outside tests. An
  unfinished path returns an "unsupported" error.
- No `unsafe` (`unsafe_code = "forbid"`), except `crates/platform`, which needs it for OS media APIs
  and documents every block (`docs/adr/0001-platform-ffi.md`).
- Treat every number that comes from input as hostile: media, projects, interchange files, presets,
  fonts, CLI / MCP / control-channel parameters and UI state. Use `get()`, `checked_*`,
  `saturating_*`, and never divide by a value that can be zero.
- Bound recursion and loops: projects can be cyclic or deeply nested.
- Every thread and job runs under `catch_unwind` and reports failure as an error.
- Lock poisoning is recovered, not fatal: `lock().unwrap_or_else(PoisonError::into_inner)`.
- Every crash fix ships with a small synthetic regression test that failed before the fix.

## 3. Licensing and assets

- The project is **MIT OR Apache-2.0**. Keep `LICENSE-MIT`, `LICENSE-APACHE` and `NOTICE`.
- Dependencies must be permissively licensed: MIT, Apache-2.0, BSD, ISC, Zlib, Unicode, CC0,
  BSL-1.0, or MPL-2.0 used unmodified. No GPL, LGPL or AGPL code — never FFmpeg, x264, x265, LAME,
  MLT or anything derived from them.
- No Adobe assets or iconography, in any form, including redrawn or "inspired" copies. No third-party
  brand marks at all: draw a generic icon instead.
- Every asset needs an `X.attribution` sidecar and a row in `ATTRIBUTION.md`. `cargo xtask assets`
  enforces this.
- **Do not reintroduce the ArtCraft name, wordmark or logos.** They are trademarks that this fork
  removed. `crates/ui-egui/src/brand.rs` intentionally draws nothing.
- Fonts live in [storytold/craft-fonts](https://github.com/storytold/craft-fonts) and arrive only
  through the optional build input `CRAFT_FONTS_DIR`. Never add craft-fonts to a `Cargo.toml`, and
  code using `CRAFT_FONTS` must work when it is empty.
- Codecs and formats are implemented from public specifications only. Do not read third-party decoder
  source while implementing a format. ffmpeg/ffprobe are test oracles only, never linked or shipped.

## 4. The engine is the only way to edit

- Every edit is an engine command (`crates/engine`) with an id, label, menu path, shortcut, params,
  `enabled()` and `run()`. UI, CLI, control channel, MCP and the agent all dispatch by id.
- The agent never mutates project state directly. It calls commands.
- **One user message is one undo step.** Every command the agent runs during a turn is tagged with
  its turn label (`Agent turn N`), so the user can undo the whole turn in one action.
- Destructive or director-tier work is a dry-run plan first: validate, show it, then apply on
  approval. `settings.json` decides whether approval is required.

## 5. Model configuration is data

- Which provider and model the agent uses lives in `settings/opencode.json` (bundled default) or the
  user's `%APPDATA%\AgenticEditCraft\settings.json`; `AGENTIC_EDIT_CRAFT_SETTINGS` overrides both.
- Nothing about a provider, endpoint or model id is compiled in. Adding a model must not require a
  code change, and `provider/model` shorthand must keep working.
- `crates/agent` parses the bundled file in its test suite, so an invalid edit fails CI before it can
  ship. Keep `settings/settings.schema.json` in sync with the structs.
- Never log an API key or write one into a project, a transcript or a settings file.

## 6. Engineering

- Layers are enforced by `cargo xtask layers`: nothing below L5 depends on egui/eframe/winit/rfd/cpal.
- All time is `filmcraft_time::Tick` (254 016 000 000/s). Never use f64 seconds for edit math.
- New workspace crates add `[lints] workspace = true`. Production crate roots declare the crate-level
  lint attributes before their modules; crate-level `//!` documentation may come first.
- Every interactive widget registers an automation id, and UI state is serde so the control channel
  can read and set it.
- Quality gates before a commit: `cargo xtask ci` (fmt, clippy `-D warnings`, tests, layers, assets,
  wasm). One task id per commit, green states only.
- Do not commit media or large binary fixtures. Generate them into `target/fixtures/`.
- `main` changes through a pull request. Never push to it.

## 7. Checking before claiming

- Read the issue, search pull requests for `#N`, and check recent commits on `main` before writing
  code.
- Confirm the cause in the code, even when someone names a location.
- Review the diff before opening a pull request: identifiers exist as used, the test fails without the
  fix, nothing unrelated changed, and the description is true.
- Report only what ran.
- No agent attribution in commits, pull requests or comments.

## See also

- [`docs/architecture.md`](docs/architecture.md): layers, data model, commands, pipeline
- [`docs/testing.md`](docs/testing.md): oracle tests, criteria, benchmarks
- [`docs/agents.md`](docs/agents.md): driving the editor over MCP and the control channel
- [`docs/control-protocol.md`](docs/control-protocol.md): control-channel method reference
- [`settings/README.md`](settings/README.md): how to configure any model
- [`ATTRIBUTION.md`](ATTRIBUTION.md): asset index
