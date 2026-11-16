//! Per-conversation agent state: transcript and the undo group for each turn.
//!
//! One user message starts one turn. Every command the agent runs during that
//! turn is tagged with the same undo label, so the editor can undo the whole
//! turn as a single step.

use serde::{Deserialize, Serialize};

/// Keep transcripts bounded even in a long session.
const MAX_ENTRIES: usize = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TranscriptEntry {
    pub role: Role,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
}

/// One agent conversation attached to one project window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentSession {
    pub id: String,
    turn: u32,
    pub entries: Vec<TranscriptEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    undo_group: Option<String>,
}

impl AgentSession {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), turn: 0, entries: Vec::new(), undo_group: None }
    }

    /// Start a turn and return the undo label every edit in it must carry.
    pub fn begin_turn(&mut self) -> String {
        self.turn = self.turn.saturating_add(1);
        let group = format!("Agent turn {}", self.turn);
        self.undo_group = Some(group.clone());
        group
    }

    /// End the turn, returning the undo label to close (one undo step).
    pub fn finish_turn(&mut self) -> Option<String> {
        self.undo_group.take()
    }

    pub fn undo_group(&self) -> Option<&str> {
        self.undo_group.as_deref()
    }

    pub fn turn(&self) -> u32 {
        self.turn
    }

    /// Append a transcript entry, dropping the oldest when the cap is reached.
    pub fn record(&mut self, role: Role, text: impl Into<String>, tool: Option<&str>) {
        if self.entries.len() >= MAX_ENTRIES {
            let excess = self.entries.len() + 1 - MAX_ENTRIES;
            self.entries.drain(..excess);
        }
        self.entries.push(TranscriptEntry { role, text: text.into(), tool: tool.map(str::to_string) });
    }

    /// The transcript as plain text, for a client that wants a single prompt.
    pub fn transcript(&self) -> String {
        let mut out = String::new();
        for entry in &self.entries {
            let role = match entry.role {
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::Tool => "tool",
            };
            if let Some(tool) = &entry.tool {
                out.push_str(&format!("[{role}:{tool}] {}\n", entry.text));
            } else {
                out.push_str(&format!("[{role}] {}\n", entry.text));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::{AgentSession, Role};

    #[test]
    fn each_turn_gets_its_own_undo_group() {
        let mut session = AgentSession::new("s1");
        let first = session.begin_turn();
        assert_eq!(first, "Agent turn 1");
        assert_eq!(session.undo_group(), Some("Agent turn 1"));
        assert_eq!(session.finish_turn(), Some(first));
        assert_eq!(session.undo_group(), None);

        let second = session.begin_turn();
        assert_eq!(second, "Agent turn 2");
        assert_eq!(session.turn(), 2);
    }

    #[test]
    fn transcript_records_roles_and_tools() {
        let mut session = AgentSession::new("s1");
        session.record(Role::User, "Cut the silence", None);
        session.record(Role::Tool, "removed 12 gaps", Some("edit.remove_silence"));
        let text = session.transcript();
        assert!(text.contains("[user] Cut the silence"));
        assert!(text.contains("[tool:edit.remove_silence] removed 12 gaps"));
    }

    #[test]
    fn transcript_is_bounded() {
        let mut session = AgentSession::new("s1");
        for i in 0..super::MAX_ENTRIES + 5 {
            session.record(Role::Assistant, format!("m{i}"), None);
        }
        assert_eq!(session.entries.len(), super::MAX_ENTRIES);
        assert_eq!(session.entries.first().map(|e| e.text.as_str()), Some("m5"));
    }
}
