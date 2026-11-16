//! Dry-run edit plans.
//!
//! The agent proposes a plan first. Destructive work only happens after the
//! plan validates and the user (or an explicit `approved` flag) accepts it.

use serde::{Deserialize, Serialize};

use crate::tools::director_tool;

/// A proposal: what the agent intends to do, and why.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EditPlan {
    pub summary: String,
    pub steps: Vec<PlanStep>,
    /// Defaults to true: a plan is inert unless someone applies it.
    #[serde(default = "plan_is_dry_by_default")]
    pub dry_run: bool,
}

const fn plan_is_dry_by_default() -> bool {
    true
}

impl Default for EditPlan {
    fn default() -> Self {
        Self {
            summary: String::new(),
            steps: Vec::new(),
            dry_run: true,
        }
    }
}

/// One tool call inside a plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanStep {
    pub tool: String,
    #[serde(default = "empty_object")]
    pub args: serde_json::Value,
    #[serde(default)]
    pub why: String,
}

fn empty_object() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PlanError {
    #[error("the plan has no summary")]
    EmptySummary,
    #[error("the plan has no steps")]
    EmptyPlan,
    #[error("the plan has {count} steps; the maximum is {max}")]
    TooManySteps { count: usize, max: usize },
    #[error("step {index} names unknown tool `{tool}`")]
    UnknownTool { index: usize, tool: String },
    #[error("step {index} (`{tool}`) has non-object args")]
    InvalidArgs { index: usize, tool: String },
}

impl EditPlan {
    /// Upper bound on steps in one plan; keeps a runaway model from
    /// generating an unbounded amount of work per approval.
    pub const MAX_STEPS: usize = 64;

    /// Check the plan is applyable without touching project state.
    pub fn validate(&self) -> Result<(), PlanError> {
        if self.summary.trim().is_empty() {
            return Err(PlanError::EmptySummary);
        }
        if self.steps.is_empty() {
            return Err(PlanError::EmptyPlan);
        }
        if self.steps.len() > Self::MAX_STEPS {
            return Err(PlanError::TooManySteps {
                count: self.steps.len(),
                max: Self::MAX_STEPS,
            });
        }
        for (index, step) in self.steps.iter().enumerate() {
            let known = director_tool(&step.tool).is_some() || step.tool.starts_with("command.");
            if !known {
                return Err(PlanError::UnknownTool {
                    index,
                    tool: step.tool.clone(),
                });
            }
            if !step.args.is_object() {
                return Err(PlanError::InvalidArgs {
                    index,
                    tool: step.tool.clone(),
                });
            }
        }
        Ok(())
    }

    /// Steps that would change the project.
    pub fn mutating_steps(&self) -> Vec<&PlanStep> {
        self.steps
            .iter()
            .filter(|step| {
                director_tool(&step.tool).is_some_and(|tool| tool.mutates)
                    || step.tool.starts_with("command.")
            })
            .collect()
    }

    /// True when applying this plan needs an explicit approval.
    pub fn requires_approval(&self) -> bool {
        !self.mutating_steps().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{EditPlan, PlanError, PlanStep};

    fn step(tool: &str) -> PlanStep {
        PlanStep {
            tool: tool.to_string(),
            args: serde_json::json!({}),
            why: String::new(),
        }
    }

    #[test]
    fn a_valid_plan_validates() {
        let plan = EditPlan {
            summary: "Rough cut the interview".to_string(),
            steps: vec![step("edit.rough_cut"), step("command.run")],
            dry_run: true,
        };
        assert_eq!(plan.validate(), Ok(()));
        assert_eq!(plan.mutating_steps().len(), 2);
        assert!(plan.requires_approval());
    }

    #[test]
    fn rejects_empty_plans() {
        let plan = EditPlan {
            summary: String::new(),
            steps: Vec::new(),
            dry_run: true,
        };
        assert_eq!(plan.validate(), Err(PlanError::EmptySummary));

        let plan = EditPlan {
            summary: "Nothing".to_string(),
            steps: Vec::new(),
            dry_run: true,
        };
        assert_eq!(plan.validate(), Err(PlanError::EmptyPlan));
    }

    #[test]
    fn rejects_unknown_tools_and_bad_args() {
        let plan = EditPlan {
            summary: "Nope".to_string(),
            steps: vec![step("edit.nope")],
            dry_run: true,
        };
        assert!(matches!(plan.validate(), Err(PlanError::UnknownTool { index: 0, .. })));

        let plan = EditPlan {
            summary: "Bad args".to_string(),
            steps: vec![PlanStep {
                tool: "edit.rough_cut".to_string(),
                args: serde_json::json!([1, 2, 3]),
                why: String::new(),
            }],
            dry_run: true,
        };
        assert!(matches!(plan.validate(), Err(PlanError::InvalidArgs { index: 0, .. })));
    }

    #[test]
    fn rejects_runaway_plans() {
        let plan = EditPlan {
            summary: "Too much".to_string(),
            steps: (0..=EditPlan::MAX_STEPS).map(|_| step("edit.rough_cut")).collect(),
            dry_run: true,
        };
        assert!(matches!(plan.validate(), Err(PlanError::TooManySteps { .. })));
    }

    #[test]
    fn plans_are_dry_by_default() {
        assert!(EditPlan::default().dry_run);
        let from_json: EditPlan =
            serde_json::from_str(r#"{"summary":"s","steps":[]}"#).unwrap();
        assert!(from_json.dry_run);
    }
}
