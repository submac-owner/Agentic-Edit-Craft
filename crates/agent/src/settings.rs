//! The user-editable model configuration.
//!
//! The file is plain JSON so it can be edited in the repository
//! (`settings/opencode.json`), in the installed app's data directory
//! (`%APPDATA%\AgenticEditCraft\settings.json`), or pointed at with
//! `AGENTIC_EDIT_CRAFT_SETTINGS`. No provider is hardcoded in the application:
//! every entry in `providers` + `models` is data.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Environment variable naming an explicit settings file.
pub const SETTINGS_ENV: &str = "AGENTIC_EDIT_CRAFT_SETTINGS";

/// The settings checked into the repository; used when no user file exists.
pub const BUNDLED_SETTINGS: &str = include_str!("../../../settings/opencode.json");

const MAX_STEPS_LIMIT: u32 = 500;
const MAX_TEMPERATURE: f32 = 2.0;

/// Top-level `settings.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Settings {
    /// Optional `$schema` pointer; ignored at runtime.
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub agent: AgentSettings,
    #[serde(default)]
    pub providers: BTreeMap<String, Provider>,
    #[serde(default)]
    pub models: BTreeMap<String, ModelEntry>,
}

const fn default_version() -> u32 {
    1
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema: None,
            version: default_version(),
            agent: AgentSettings::default(),
            providers: BTreeMap::new(),
            models: BTreeMap::new(),
        }
    }
}

/// How the agent behaves; the model id is a key in [`Settings::models`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct AgentSettings {
    pub model: String,
    pub temperature: f32,
    pub max_steps: u32,
    pub approval: Approval,
    pub undo_per_turn: bool,
}

impl Default for AgentSettings {
    fn default() -> Self {
        Self {
            model: "anthropic/claude-sonnet-4-5".to_string(),
            temperature: 0.2,
            max_steps: 60,
            approval: Approval::Edits,
            undo_per_turn: true,
        }
    }
}

/// When the UI must ask the user before a tool call runs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Approval {
    /// Ask before every tool call, including reads.
    All,
    /// Ask before mutating calls; reads run unattended.
    #[default]
    Edits,
    /// Never ask. Only for trusted, sandboxed sessions.
    None,
}

/// A provider endpoint. Adding a model never requires a code change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Provider {
    pub kind: ProviderKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// Environment variable holding the key; `null` for local endpoints.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_env: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderKind {
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "openai")]
    OpenAi,
    #[serde(rename = "openaiCompatible")]
    OpenAiCompatible,
    #[serde(rename = "google")]
    Google,
}

/// One selectable model: `provider` is a key in [`Settings::providers`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelEntry {
    pub provider: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<u64>,
}

/// A model id with its provider fully resolved, ready for a client.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedModel {
    pub id: String,
    pub provider_name: String,
    pub kind: ProviderKind,
    pub base_url: Option<String>,
    pub api_key_env: Option<String>,
    pub model: String,
    pub temperature: f32,
    pub max_steps: u32,
    pub approval: Approval,
    pub undo_per_turn: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("could not read settings at {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid settings JSON in {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("model `{model}` is not declared in `models` and cannot be split into provider/model")]
    UnknownModel { model: String },
    #[error("model `{model}` names provider `{provider}`, which is not declared in `providers`")]
    UnknownProvider { model: String, provider: String },
    #[error("agent.maxSteps is {0}; it must be between 1 and 500")]
    InvalidMaxSteps(u32),
    #[error("agent.temperature is {0}; it must be between 0 and 2")]
    InvalidTemperature(f32),
}

impl Settings {
    /// Parse settings from a string; `path` is only used in error messages.
    pub fn parse(text: &str, path: &Path) -> Result<Self, SettingsError> {
        let settings: Self =
            serde_json::from_str(text).map_err(|source| SettingsError::Parse {
                path: path.to_path_buf(),
                source,
            })?;
        settings.validate_agent()?;
        Ok(settings)
    }

    /// Read and parse a settings file.
    pub fn load_from(path: &Path) -> Result<Self, SettingsError> {
        let text = std::fs::read_to_string(path).map_err(|source| SettingsError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        Self::parse(&text, path)
    }

    /// The settings bundled with this build.
    pub fn bundled() -> Result<Self, SettingsError> {
        Self::parse(BUNDLED_SETTINGS, Path::new("<bundled>"))
    }

    /// Load the user's file when it exists, otherwise the bundled defaults.
    pub fn load() -> Result<Self, SettingsError> {
        let path = settings_path();
        if path.is_file() {
            Self::load_from(&path)
        } else {
            Self::bundled()
        }
    }

    /// Resolve the configured model, or an explicit `id`, into a client-ready value.
    pub fn resolve(&self, id: Option<&str>) -> Result<ResolvedModel, SettingsError> {
        let requested = id.unwrap_or(self.agent.model.as_str()).to_string();
        let entry = self.model_entry(&requested)?;
        let provider = self
            .providers
            .get(&entry.provider)
            .ok_or_else(|| SettingsError::UnknownProvider {
                model: requested.clone(),
                provider: entry.provider.clone(),
            })?;
        Ok(ResolvedModel {
            id: requested,
            provider_name: entry.provider.clone(),
            kind: provider.kind,
            base_url: provider.base_url.clone(),
            api_key_env: provider.api_key_env.clone(),
            model: entry.model.clone(),
            temperature: self.agent.temperature,
            max_steps: self.agent.max_steps,
            approval: self.agent.approval,
            undo_per_turn: self.agent.undo_per_turn,
        })
    }

    fn model_entry(&self, id: &str) -> Result<ModelEntry, SettingsError> {
        if let Some(entry) = self.models.get(id) {
            return Ok(entry.clone());
        }
        // `provider/model` shorthand, so a new model needs no `models` entry.
        match id.split_once('/') {
            Some((provider, model)) if !provider.is_empty() && !model.is_empty() => {
                Ok(ModelEntry {
                    provider: provider.to_string(),
                    model: model.to_string(),
                    context: None,
                })
            }
            _ => Err(SettingsError::UnknownModel {
                model: id.to_string(),
            }),
        }
    }

    fn validate_agent(&self) -> Result<(), SettingsError> {
        if self.agent.max_steps == 0 || self.agent.max_steps > MAX_STEPS_LIMIT {
            return Err(SettingsError::InvalidMaxSteps(self.agent.max_steps));
        }
        if !(0.0..=MAX_TEMPERATURE).contains(&self.agent.temperature) {
            return Err(SettingsError::InvalidTemperature(self.agent.temperature));
        }
        Ok(())
    }
}

/// Where the app looks for the user's settings file.
pub fn settings_path() -> PathBuf {
    let explicit = std::env::var_os(SETTINGS_ENV).map(PathBuf::from);
    let appdata = std::env::var_os("APPDATA").map(PathBuf::from);
    let home = std::env::var_os("HOME").map(PathBuf::from);
    resolve_settings_path(explicit, appdata, home)
}

/// Pure path resolution, so it can be tested without mutating the process environment.
fn resolve_settings_path(
    explicit: Option<PathBuf>,
    appdata: Option<PathBuf>,
    home: Option<PathBuf>,
) -> PathBuf {
    if let Some(path) = explicit {
        return path;
    }
    if let Some(dir) = appdata {
        return dir.join("AgenticEditCraft").join("settings.json");
    }
    if let Some(dir) = home {
        return dir
            .join(".config")
            .join("agentic-edit-craft")
            .join("settings.json");
    }
    PathBuf::from(".agentic-edit-craft").join("settings.json")
}

#[cfg(test)]
mod tests {
    use super::{AgentSettings, Approval, ProviderKind, Settings, SettingsError};
    use std::path::Path;

    #[test]
    fn bundled_settings_parse_and_resolve() {
        let settings = Settings::bundled().unwrap();
        let resolved = settings.resolve(None).unwrap();
        assert_eq!(resolved.id, settings.agent.model);
        assert!(!resolved.model.is_empty());
    }

    #[test]
    fn shorthand_model_needs_no_models_entry() {
        let settings = Settings::bundled().unwrap();
        let resolved = settings.resolve(Some("openai/some-new-model")).unwrap();
        assert_eq!(resolved.provider_name, "openai");
        assert_eq!(resolved.model, "some-new-model");
    }

    #[test]
    fn unknown_provider_is_reported() {
        let settings = Settings::bundled().unwrap();
        let error = settings.resolve(Some("nope/model")).unwrap_err();
        assert!(matches!(error, SettingsError::UnknownProvider { .. }));
    }

    #[test]
    fn bare_unknown_model_is_reported() {
        let settings = Settings::bundled().unwrap();
        let error = settings.resolve(Some("not-a-model")).unwrap_err();
        assert!(matches!(error, SettingsError::UnknownModel { .. }));
    }

    #[test]
    fn rejects_out_of_range_agent_settings() {
        let text = r#"{
            "version": 1,
            "agent": { "maxSteps": 0 },
            "providers": {},
            "models": {}
        }"#;
        let error = Settings::parse(text, Path::new("test.json")).unwrap_err();
        assert!(matches!(error, SettingsError::InvalidMaxSteps(0)));

        let text = r#"{
            "version": 1,
            "agent": { "temperature": 9.0 },
            "providers": {},
            "models": {}
        }"#;
        let error = Settings::parse(text, Path::new("test.json")).unwrap_err();
        assert!(matches!(error, SettingsError::InvalidTemperature(_)));
    }

    #[test]
    fn reads_a_settings_file() {
        let path = std::env::temp_dir().join(format!(
            "agentic-edit-craft-settings-{}.json",
            std::process::id()
        ));
        let text = r#"{
            "version": 1,
            "agent": { "model": "local/test" },
            "providers": {
                "local": { "kind": "openaiCompatible", "baseUrl": "http://127.0.0.1:1234/v1" }
            },
            "models": {}
        }"#;
        std::fs::write(&path, text).unwrap();
        let settings = Settings::load_from(&path).unwrap();
        let resolved = settings.resolve(None).unwrap();
        assert_eq!(resolved.kind, ProviderKind::OpenAiCompatible);
        assert_eq!(resolved.base_url.as_deref(), Some("http://127.0.0.1:1234/v1"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn defaults_are_approval_edits_and_single_undo_per_turn() {
        let agent = AgentSettings::default();
        assert_eq!(agent.approval, Approval::Edits);
        assert!(agent.undo_per_turn);
    }

    #[test]
    fn settings_path_honours_the_explicit_location() {
        let path = super::resolve_settings_path(
            Some(std::path::PathBuf::from("C:/tmp/example.json")),
            Some(std::path::PathBuf::from("C:/Users/example/AppData/Roaming")),
            None,
        );
        assert_eq!(path, std::path::PathBuf::from("C:/tmp/example.json"));
    }

    #[test]
    fn settings_path_falls_back_to_appdata_then_home() {
        let appdata = super::resolve_settings_path(
            None,
            Some(std::path::PathBuf::from("C:/Users/example/AppData/Roaming")),
            Some(std::path::PathBuf::from("/home/example")),
        );
        assert!(appdata.ends_with("AgenticEditCraft/settings.json"));

        let home = super::resolve_settings_path(
            None,
            None,
            Some(std::path::PathBuf::from("/home/example")),
        );
        assert!(home.ends_with(".config/agentic-edit-craft/settings.json"));
    }
}
