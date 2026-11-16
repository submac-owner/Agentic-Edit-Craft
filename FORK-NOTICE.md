# Fork notice

**Agentic Edit Craft** is a fork of **FilmCraft**.

- Upstream: https://github.com/storytold/filmcraft
- Base release: `v0.20.1` (tag), commit `cedf0c9bd060eb06a977d532402be2097c9ebe42`
- Upstream license: MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`)
- Upstream copyright: Copyright (c) 2026 ArtCraft Team and the FilmCraft contributors
- This fork: Copyright (c) 2026 the Agentic Edit Craft contributors

## What changed in this fork

1. **Branding.** The ArtCraft name, wordmark and logo assets are trademarks of the ArtCraft
   Team and are **not** part of this fork. `docs/brand/` was removed, and
   `crates/ui-egui/src/brand.rs` no longer embeds or draws those files. No ArtCraft artwork
   is redistributed here.
2. **Agent layer.** Added `crates/agent`: model settings, the director-tool contract,
   dry-run edit plans, and per-turn undo grouping.
3. **Model settings.** Added `settings/opencode.json` plus a schema and documentation, so the
   model/provider is data rather than code.
4. **CI.** Replaced the upstream shared-workflow CI with a self-contained GitHub Actions
   workflow that builds the Windows app and tests the agent crate.

## What was not changed

Codec, container, render, export and engine code is upstream code, unmodified except where
noted above. Upstream's clean-room and asset rules still apply to it; see `AGENTS.md`.

## Attribution obligations when you redistribute

- Keep `LICENSE-MIT`, `LICENSE-APACHE` and `NOTICE`.
- Keep this file.
- Do not reintroduce ArtCraft trademarks or their logo files.
