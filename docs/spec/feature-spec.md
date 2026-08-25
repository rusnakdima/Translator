# Translator — Target Specification (from Tauri `master`)

Source of truth: git branch `master` (Angular frontend over `src-tauri` Rust backend using
`tauri-shared`, `nosql_orm`, the `trad` translation crate). Extracted via git plumbing; this is
a **blueprint of required behavior**, not a diff. Evidence anchors reference
`master:` paths. Translator is the **reference exemplar** for schema-driven apps in this
portfolio; §7 lists master logic not yet present in the current Dioxus/schema tree
(`Translator/src/`, schemas v3).

---

## 1. Priority summary

| Priority | Count | Features |
|---|---|---|
| **P0** | 3 | Translation engine contract (15-language catalog, validation, async request/event flow); language & settings persistence; history entity & storage |
| **P1** | 4 | Schema CRUD persistence layer (`UiSchema` envelope); i18n UI locales (en/ru); keyboard shortcut catalog; toast feedback taxonomy |
| **P2** | 3 | Input cap (5000 chars); algorithm registry commands; logger commands |

## 2. P0 features

### F-T-P0-1 Translation engine contract (`master:src-tauri/src/helpers/translator.helper.rs`, `services/translation.service.rs`, `models/translation.model.rs`)

Supported languages — exact catalog of 15 `(code, name)` pairs:

```text
en English · es Spanish · fr French · de German · it Italian · pt Portuguese
ru Russian · ja Japanese · ko Korean · zh Chinese · ar Arabic · hi Hindi
nl Dutch · pl Polish · tr Turkish
```

Engine: `trad::Translator`, lazily initialized once behind a `RwLock<Option<..>>`
singleton (`ensure_initialized`); all subsequent calls share the instance under a read guard.

**Decision rules (translate(text, source_lang, target_lang) → Result<String,String>)**, in order:

1. `text.trim().is_empty()` → error `"Empty text provided"` (no network/engine call).
2. `source_lang == target_lang` → return input unchanged (**passthrough**, no code validation).
3. source code not in catalog → error `"Unsupported source language: {code}"`.
4. target code not in catalog → error `"Unsupported target language: {code}"`.
5. ensure engine initialized; translate via `trad`; engine failure → `"Translation failed: {e}"`.
6. empty translated result → error `"Empty translation result"` (never resolve to empty string).

Service response wrapper (`Response<TranslationResponse>`): success data =
`{translatedText, sourceLang, targetLang}`; on error, data = empty-text response with the same
langs plus message `"Translation failed: {e}"`.

`get_supported_languages()` → `LanguagesResponse{languages: Vec<{code,name}>}` from the static
catalog, no engine init required.

**Async request/event flow** (`master:src-tauri/src/lib.rs`) — the signature behavior of
master's backend:

```text
translate_text(text, source_lang, target_lang) -> Ok(request_id)   // returns IMMEDIATELY
  request_id = global AtomicUsize counter (0,1,2,…)
  spawn task:
     response = service.translate_async(...)
     emit window event "translation-result" with payload:
       { requestId, text, sourceLang, targetLang,
         response: <full Response JSON> }
```

The frontend correlates results by `requestId`. Target must preserve: immediate ack, monotonic
ids, one event per request, payload field names exactly as above (camelCase JSON).

### F-T-P0-2 Language selection & user settings persistence (`master:src-tauri/src/entities/settings_entity.rs`)

Entity `UserSettings { id: Option<String>, source_lang: String, target_lang: String }`,
table `user_settings`, camelCase serde. Defaults from frontend global state: source `en`,
target `ru`. Persisted through the unified CRUD command surface (`crud_execute`, see F-T-P1-1)
rather than bespoke commands. Swap semantics live client-side: swap exchanges source/target
signals (GlobalStateService.swapLanguages), then persists.

Current-tree divergence to reconcile: current `AppSettings{default_source_lang, default_target_lang,
auto_detect, cache_enabled, batch_size, theme}` extends the master pair — keep superset fields but
the two lang fields are the master-contract minimum that settings pages bind to.

### F-T-P0-3 Translation history (`master:src-tauri/src/entities/history_entity.rs`)

Entity `TranslationHistoryEntry { id, text, translated_text, source_lang, target_lang,
timestamp }` (camelCase JSON: `translatedText` etc.), table `translation_history`, one record
per completed translation, timestamp ISO string. Storage via nosql_orm JsonProvider rooted at
app-data dir (`NOSQL_ORM_DATA_DIR`, db path `translator_db`). Current tree has history domain +
infra already; the contract to honor is the exact field set/ordering by timestamp for "recent"
views.

## 3. P1 features

### F-T-P1-1 Schema CRUD persistence (`master:src-tauri/src/commands/mod.rs`)

The app UI itself is SDUI-rendered (`App` mounts `<lib-schema-shell appId="translator"/>`);
schemas are first-class stored documents:

```rust
UiSchema {
  id: String,                    // top-level, extracted from app.id
  version: String,               // alias accepts schemaVersion
  app: Value, pages: Vec<Value>, layouts: Vec<Value>,
  shortcuts: Vec<Value>,         // default empty
  handlers: Value,               // default null/empty
  stores: Value,                 // default null/empty
}
```

Commands: `get_schema(id)` → NotFound Response when absent; `save_schema(schema)` upsert;
`get_all_schemas()` list; `delete_schema(id)` remove. Table `schemas` in the same JsonProvider.
Plus shared-library generic command `crud_execute(entity, operation, id?, data?)` covering
settings/history records (operations get/get_all/create/save/update/patch/delete/count/exists).

### F-T-P1-2 i18n of the app chrome (`master:src/app/shared/services/i18n.service.ts`)

Shared I18nService singleton; locales limited to `en | ru`; API: setLocale, locale,
translations map, `t(key)`, getAvailableLocales. App locale is independent of translation
source/target languages. Target keeps en/ru bundle keys for menus/settings/shortcuts screens.

### F-T-P1-3 Keyboard shortcut catalog (`master:src/app/shared/utils/constants.ts`, `shortcut.service.ts`)

Normative shortcut set (action ids are stable identifiers used by schema `handlers`):

| Keys | Action id | Meaning |
|---|---|---|
| F1 | show-shortcuts | open shortcuts overlay |
| Ctrl+/ | show-shortcuts | alternate overlay trigger |
| Ctrl+Enter | translate | run translation |
| Ctrl+S / Ctrl+L | swap / swap_languages | exchange source↔target |
| Ctrl+K | clear text | clear source |
| Ctrl+Shift+C | quick-copy / copy_result | copy translation to clipboard |
| Ctrl+Shift+V | quick-paste | paste clipboard into source |
| Ctrl+, | open settings | navigate to settings page |
| Ctrl+Shift+S | focus-source | focus source input |
| Ctrl+Shift+L / Ctrl+Shift+; | focus-source-lang / focus-target-lang | focus language pickers |
| Escape | close | close overlay |

ShortcutService registers a document keydown listener (F1 handled globally dispatching an
`onShortcutsOpen` custom event) and exposes the list for the shortcuts page. Schema v3 already
carries a subset (`Ctrl+Enter translate`, `Ctrl+S swap_languages`, `Ctrl+T toggle_theme`) —
target: full catalog expressed declaratively in schema `shortcuts` + handlers.

### F-T-P1-4 Toast feedback taxonomy (`master:src/app/shared/utils/toast.helper.ts`)

Three kinds `info | success | error`, default duration 3000 ms; helper routes through shared
ToastService when present, else falls back to `window.showToast(message,type)`. Usage contract:
success on completed translation/copy, error on failed translation or unsupported language,
info for neutral notices.

## 4. P2 features

### F-T-P2-1 Source input cap

Client-side max length **5000 chars** (`TranslationService.maxChars`); enforced at the input
component level before invoke. Not validated server-side on master — keep as presentation rule.

### F-T-P2-2 Algorithm registry commands

From tauri-shared: `list_algorithms()` and `execute_algorithm(...)` registered in the invoke
handler. Master exposes them to every app uniformly; target keeps the registry pattern if the
shared library retains it, else document as dropped capability.

### F-T-P2-3 Logger commands

tauri-shared logger plugin commands `get_log_entries`, `set_log_level`, `clear_logs`; log
format `[{timestamp}] [{level}] [translator] [{target}] {message}`. MCP bridge additionally
offers `logs_read`.

## 5. Persistence contracts summary

| Table | Entity | Access path |
|---|---|---|
| `schemas` | UiSchema | dedicated get/save/get_all/delete commands |
| `user_settings` | UserSettings | `crud_execute` |
| `translation_history` | TranslationHistoryEntry | `crud_execute` |

Storage: nosql_orm JsonProvider at `{app_data_dir}/translator_db`; env var
`NOSQL_ORM_DATA_DIR` set at setup.

## 6. Cross-cutting contracts

- Response envelope everywhere: `{status: success|error, message?, data?}` (tauri-shared
  Response); frontend unwraps `.data.languages ?? []` style defensively.
- Events: single custom event name `translation-result` (constants.TAURI_EVENTS).
- Error surfacing: backend logs errors via `log_error!`; user sees toast + inline message.
- Strangler verification: `src-tauri/tests/strangler/sdui_verify.rs` asserts schema-shell usage
  and absence of hardcoded component selectors — retain as the compliance gate for the
  schema-driven rewrite.

## 7. Master logic NOT yet in the current schema-driven app (gap list)

Verified against `Translator/src/` (dioxus-migration) and `schemas/translator_v3.json`:

| # | Master behavior | Status in current tree | Spec ref |
|---|---|---|---|
| 1 | 15-language catalog with names | current `Language` entity exists; catalog contents not pinned to the 15-pair table | F-T-P0-1 |
| 2 | passthrough on equal langs happens BEFORE code validation | bridge tests cover same-lang passthrough ✓, but ordering vs validation must be preserved in domain impl | F-T-P0-1 |
| 3 | async request-id + `translation-result` event correlation | absent — current flow is synchronous command-style | F-T-P0-1 |
| 4 | lazy-init singleton engine behind RwLock | current infra uses its own translator; init semantics unspecified | F-T-P0-1 |
| 5 | empty-text / unsupported-lang / empty-result error strings | partially present (bridge errors); exact strings unpinned | F-T-P0-1 |
| 6 | `UiSchema` envelope persistence incl. `handlers`, `stores` | schema files exist as assets; no runtime schema CRUD storage | F-T-P1-1 |
| 7 | unified `crud_execute` surface for settings/history | current uses per-service storages | F-T-P1-1 |
| 8 | en/ru i18n of chrome | absent | F-T-P1-2 |
| 9 | full 11-entry shortcut catalog (schema v3 carries only 3) | partial | F-T-P1-3 |
| 10 | toast taxonomy w/ 3000 ms default | absent | F-T-P1-4 |
| 11 | 5000-char input cap | absent | F-T-P2-1 |
| 12 | logger/algorithm registry commands | MCP bridge has `logs_read`/`commands_list` only | F-T-P2-2/3 |
| 13 | history field set incl. `timestamp` | current domain matches shape (created_at) — rename/pin to contract | F-T-P0-3 |

Items 3, 6, 8, 10 are the highest-value exemplar behaviors for other apps' schema-driven ports.

## 8. Diagram index (`docs/diagrams/`)

| File | Covers |
|---|---|
| `spec-architecture.puml` | component map: schema shell ↔ Rust services ↔ storage |
| `spec-translation-pipeline.puml` | activity: validate → engine → event emission → UI correlation |
