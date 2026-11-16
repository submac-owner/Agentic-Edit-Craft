# Agentic Edit Craft

A Windows desktop NLE where a human and an agent edit the **same timeline**.

The editor core is a fork of [FilmCraft](https://github.com/storytold/filmcraft) (MIT OR
Apache-2.0, pure Rust, own codecs). This fork adds:

- an **agent side panel** (in progress) that talks to the engine through the app's MCP server,
- a **director tool tier** — composite edits (rough cut, montage, silence removal, reframe,
  color match, loudness conform, delivery variants, QC) that are not single-click UI actions,
- a **user-editable model configuration** (`settings/opencode.json`) for any provider/model,
- **one undo step per agent turn**.

Status: **scaffold**. The editor builds and runs; the agent panel is the next milestone.
All builds happen in GitHub Actions (`.github/workflows/build.yml`); the artifact is
`agentic-edit-craft-windows-x64`.

## Architecture

| Layer | Crate | Notes |
|---|---|---|
| Codecs, containers, frame, color, time | `crates/h264`, `hevc`, `vp9`, `aac`, `isobmff`, … | first-party Rust, no FFmpeg |
| Timeline, project, render, export | `crates/edit`, `project`, `render`, `export` | integer ticks, non-destructive |
| Engine + command registry | `crates/engine` | 700+ commands; UI, CLI, control channel and MCP dispatch the same ids |
| UI | `crates/ui-egui` | swappable front end over the engine |
| Automation | `crates/automation` | MCP server |
| **Agent** | **`crates/agent`** | settings, director tool contract, dry-run plans, per-turn undo |

Agent edits are commands, never direct state mutation: an agent turn is applied as
`command.batch` under one undo label (`Agent turn N`), so the user can undo the whole turn.

## Model settings

Edit [ `settings/opencode.json` ](settings/opencode.json) — providers, endpoints, model ids,
approval policy, step cap. At runtime the app reads, in order:
`AGENTIC_EDIT_CRAFT_SETTINGS` → `%APPDATA%\AgenticEditCraft\settings.json` → the bundled file.
See [settings/README.md](settings/README.md).

## Build and run

GitHub Actions builds the Windows binaries on every push to `main`:

```
Actions → build → Windows app + CLI → artifact: agentic-edit-craft-windows-x64
```

Locally (contributors):

```powershell
cargo build --release -p filmcraft -p filmcraft-cli
./target/release/filmcraft.exe --control 9876
```

The engine is agent-drivable out of the box:

```powershell
./target/release/filmcraft-cli.exe mcp --bridge 127.0.0.1:9876   # MCP over stdio
./target/release/filmcraft-cli.exe commands                       # 700+ command ids
```

## Licensing

- This fork: **MIT OR Apache-2.0**, at your option. Keep `LICENSE-MIT`, `LICENSE-APACHE`
  and `NOTICE` with every distribution.
- Upstream copyright and attribution: see [FORK-NOTICE.md](FORK-NOTICE.md).
- No GPL/LGPL/AGPL code is linked or bundled. Codec patents (H.264/HEVC/AAC) are a separate
  legal matter for anyone shipping commercially.

## Roadmap

1. Agent panel inside the editor (chat, plan/diff, approve, undo last turn).
2. Director tools implemented as engine commands, exposed through MCP.
3. Settings UI that writes the same `settings.json`.
4. Windows installer + code signing.
