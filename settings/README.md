# Model settings

`opencode.json` is the **single source of truth for which model the side-panel agent
uses**. Nothing is compiled in: providers, endpoints, model ids and agent behaviour are
all data.

## Where it lives

| Priority | Location | Who edits it |
|---|---|---|
| 1 | `AGENTIC_EDIT_CRAFT_SETTINGS` (absolute path) | power users / CI |
| 2 | `%APPDATA%\AgenticEditCraft\settings.json` | the installed app |
| 3 | `settings/opencode.json` in this repository (bundled fallback) | everyone, by pull request or in the GitHub web editor |

The app reads the first file that exists and falls back to the bundled copy, so editing
`settings/opencode.json` here and merging to `main` changes the default for every build.

## Configure any model

1. Change `agent.model` to a key under `models` — or use `provider/model` shorthand.
2. Add a provider under `providers` (any OpenAI-compatible endpoint works).
3. Provide the API key through the environment variable named in `apiKeyEnv`.
   Local endpoints (`ollama`, `lmstudio`) use `null` and need no key.

```json
{
  "agent": { "model": "openrouter/qwen3-coder" },
  "providers": {
    "openrouter": {
      "kind": "openaiCompatible",
      "baseUrl": "https://openrouter.ai/api/v1",
      "apiKeyEnv": "OPENROUTER_API_KEY"
    }
  }
}
```

`provider/model` shorthand means a brand-new model id works without a `models` entry; add
one only when you want to record a context window or pin a friendly name.

## Agent behaviour

| Key | Meaning |
|---|---|
| `approval` | `all` asks before every call · `edits` asks before mutating calls (default) · `none` never asks |
| `undoPerTurn` | group every edit in one user message under a single undo step (keep `true`) |
| `maxSteps` | hard cap on tool calls per turn (1–500) |
| `temperature` | 0–2 |

## Validation

`crates/agent` parses this exact file in its test suite (`bundled_settings_parse_and_resolve`),
so an invalid edit fails CI before it can ship. The schema is in `settings.schema.json`.
