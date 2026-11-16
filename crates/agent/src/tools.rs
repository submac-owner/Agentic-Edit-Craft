//! The tool contract between the agent and the editor.
//!
//! Atomic edits are engine commands (`command.run` / `command.batch`), exactly
//! what a power user can do. The director tier below is intentionally *not*
//! reachable as a single click in the UI: each entry replaces a multi-step
//! analysis-and-edit workflow that a human would otherwise perform by hand.
//! Every director tool is mutating by default and must be approved.

use serde::{Deserialize, Serialize};

/// Which layer a tool belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Tier {
    /// Reads project, media, or frames. Never mutates.
    Inspect,
    /// One engine command; the same operation the user can invoke.
    Edit,
    /// A composite workflow with analysis and planning baked in.
    Director,
}

/// A tool the agent may call, with the safety flags an MCP client needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolSpec {
    /// Stable MCP tool name (`edit.rough_cut`).
    pub name: &'static str,
    pub tier: Tier,
    /// True when the call changes the project (undoable, but still a change).
    pub mutates: bool,
    /// True when the UI must ask before the call runs under `Approval::Edits`.
    pub needs_approval: bool,
    pub summary: &'static str,
}

const fn director(name: &'static str, summary: &'static str) -> ToolSpec {
    ToolSpec {
        name,
        tier: Tier::Director,
        mutates: true,
        needs_approval: true,
        summary,
    }
}

/// The elevated tier: composite editing workflows offered to the agent only.
pub const DIRECTOR_TOOLS: &[ToolSpec] = &[
    director(
        "edit.rough_cut",
        "Transcribe and score a bin, propose selects, and lay a target-length rough cut on the timeline.",
    ),
    director(
        "edit.montage",
        "Cut a set of shots to a music bed on its beat/downbeat grid in a chosen style.",
    ),
    director(
        "edit.remove_silence",
        "Detect dead air across dialogue tracks and ripple it out with handles and padding.",
    ),
    director(
        "edit.multicam_sync",
        "Sync multiple angles by audio/visual correlation and build a multicam sequence.",
    ),
    director(
        "edit.reframe",
        "Reframe a sequence to another aspect ratio with subject tracking and title-safe margins.",
    ),
    director(
        "color.match",
        "Match a clip's grade to a reference frame or shot and write the resulting grade.",
    ),
    director(
        "audio.duck",
        "Write music ducking automation under dialogue with configurable depth and release.",
    ),
    director(
        "audio.conform",
        "Conform a mix to a loudness target (for example -14 or -23 LUFS) with true-peak limits.",
    ),
    director(
        "caption.generate",
        "Transcribe speech, segment it, and create a styled caption track, optionally burned in.",
    ),
    director(
        "fx.recipe",
        "Apply a named, parameterized effect stack consistently across selected clips.",
    ),
    director(
        "delivery.variants",
        "Render several aspect-ratio or length variants from one timeline with per-variant reframes.",
    ),
    director("qc.check", "Run delivery QC: offline media, black holes, flash frames, loudness, caption overflow."),
    director(
        "review.self_critique",
        "Render key frames, score the cut against the brief, and propose concrete fixes.",
    ),
    director(
        "history.checkpoint",
        "Name a restore point for this turn so the user can return to it in one action.",
    ),
];

/// Look up a director tool by its MCP name.
pub fn director_tool(name: &str) -> Option<&'static ToolSpec> {
    DIRECTOR_TOOLS.iter().find(|tool| tool.name == name)
}

/// Names in registration order, for documentation and tests.
pub fn director_tool_names() -> Vec<&'static str> {
    DIRECTOR_TOOLS.iter().map(|tool| tool.name).collect()
}

#[cfg(test)]
mod tests {
    use super::{DIRECTOR_TOOLS, Tier, director_tool, director_tool_names};
    use std::collections::BTreeSet;

    #[test]
    fn names_are_unique_and_namespaced() {
        let names = director_tool_names();
        let unique: BTreeSet<&str> = names.iter().copied().collect();
        assert_eq!(unique.len(), names.len());
        for name in names {
            assert!(name.contains('.'), "{name} is not namespaced");
            assert!(!name.split('.').any(str::is_empty), "{name} has an empty segment");
        }
    }

    #[test]
    fn every_director_tool_is_elevated_and_gated() {
        for tool in DIRECTOR_TOOLS {
            assert_eq!(tool.tier, Tier::Director);
            assert!(tool.mutates, "{} should mutate", tool.name);
            assert!(tool.needs_approval, "{} must require approval", tool.name);
            assert!(!tool.summary.trim().is_empty(), "{} needs a summary", tool.name);
        }
    }

    #[test]
    fn lookup_finds_and_misses() {
        assert!(director_tool("edit.rough_cut").is_some());
        assert!(director_tool("edit.nope").is_none());
    }
}
