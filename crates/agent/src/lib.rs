//! Agent layer for Agentic Edit Craft.
//!
//! This crate is deliberately independent of the editor UI: it defines
//!
//! - [`settings`] — the user-editable `settings.json` (any provider, any model),
//! - [`tools`] — the director-tier tool contract the agent is allowed to call,
//! - [`plan`] — dry-run plans and their validation before anything is applied,
//! - [`session`] — per-turn transcripts and the undo group each turn maps to.
//!
//! The editor exposes every edit as an engine command; the agent never mutates
//! project state directly. A turn is applied as commands, grouped under one undo
//! label, so "undo last agent turn" is a single user action.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod plan;
pub mod session;
pub mod settings;
pub mod tools;

pub use plan::{EditPlan, PlanError, PlanStep};
pub use session::{AgentSession, Role, TranscriptEntry};
pub use settings::{Approval, ProviderKind, ResolvedModel, Settings, SettingsError};
pub use tools::{DIRECTOR_TOOLS, Tier, ToolSpec, director_tool};
