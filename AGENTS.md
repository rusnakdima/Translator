# AGENTS.md - Development Guide for Translator

This document is the primary source of truth for agentic coding agents operating within the Translator repository. It outlines project structure, coding standards, and operational commands.

## 1. Project Overview

- **Framework:** Dioxus 0.8 (Desktop) + `dioxus-desktop`
- **UI Architecture:** 100% Schema-Driven UI (SDUI) — ZERO hardcoded UI pages
- **Rust Edition:** 2021
- **Key Dependencies:** `dioxus`, `dioxus-desktop`, `dioxus-shared`, `trad` (translation engine), `serde`, `serde_json`, `chrono`, `tokio`
- **MCP Bridge:** `dioxus-plugin-mcp-bridge` for external tool integration (ws://127.0.0.1:9223)

### Core Principle

All UI is generated from schema JSON at runtime. There are no hardcoded page components. Edit the schema JSON to change UI.

---

## 2. Directory Structure

```
Translator/
├── src/
│   ├── main.rs                 # App entry, ActionBus setup, schema loading
│   ├── lib.rs                  # Crate root, re-exports domain/infrastructure
│   ├── domain/                 # Domain entities
│   │   ├── mod.rs
│   │   ├── translation.rs      # TranslationRequest, TranslationResponse, Language, trait TranslationService
│   │   ├── glossary.rs         # GlossaryItem
│   │   ├── history.rs          # HistoryEntry
│   │   └── settings.rs         # AppSettings
│   ├── application/            # Application services
│   │   ├── mod.rs
│   │   └── translation_service.rs  # Orchestrates domain + infrastructure
│   ├── infrastructure/         # External integrations
│   │   ├── mod.rs
│   │   ├── translation.rs      # TranslationBackend (uses `trad` crate)
│   │   ├── glossary.rs         # In-memory glossary storage
│   │   ├── history.rs          # In-memory history storage
│   │   └── settings.rs         # JSON file settings persistence
├── schemas/
│   └── translator.json       # SDUI schema (3 pages, modals, shortcuts; symlink → /mnt/Other/Projects/schemas/translatorschemas.json)
├── Cargo.toml
└── AGENTS.md                   # This file

dioxus-shared/                  # Shared SDUI library
├── src/
│   ├── schema/                  # Schema types (Schema, Page, CanvasElement, Modal, etc.)
│   │   ├── mod.rs
│   │   ├── page.rs             # Schema, Page, CanvasElement, Shortcut, Modal
│   │   ├── entity.rs           # Entity, EntityField, FieldType
│   │   └── ...
│   ├── ui/
│   │   ├── mod.rs
│   │   └── components/
│   │       ├── mod.rs
│   │       ├── dynamic_page.rs     # DynamicPage component (renders pages from schema)
│   │       ├── dynamic_renderer.rs # DynamicRenderer component (renders elements)
│   │       ├── action_bus.rs      # ActionBus context (dispatch, bindings, nav, modals, theme)
│   │       ├── modal.rs
│   │       └── ...
│   └── ...
```

---

## 3. Schema-Driven UI (SDUI) Architecture

### Flow

```
Schema JSON (schemas/translator.json → /mnt/Other/Projects/schemas/translatorschemas.json)
    ↓ seeded into JSON DB (~/.local/share/com.tcs.translator/schemas.json) via nosql_orm
DynamicPage component (src/main.rs)
    ↓ finds page by route
DynamicRenderer component (dioxus-shared)
    ↓ matches component string to arm
Concrete Dioxus element (div, button, textarea, etc.)
```

### Schema Structure

```json
{
  "app_id": "translator",
  "version": "1.0.0",
  "id": "translator",
  "shortcuts": [...],
  "modals": [...],
  "pages": [
    {
      "id": "translate-page",
      "title": "Translate",
      "route": "/",
      "layout": "stacked",
      "elements": [
        {
          "id": "header",
          "component": "div",
          "classes": "flex items-center justify-between p-4 divider",
          "props": {},
          "children": [...]
        }
      ]
    }
  ]
}
```

### Key Schema Types

- **Schema**: Top-level container with `app_id`, `version`, `pages`, `shortcuts`, `modals`, `id`
- **Page**: Single page with `route` path, `elements` array
- **CanvasElement**: Any UI element with `component` (type string), `props` (key-value), `classes` (semantic layout classes + style-free semantic tokens resolved by `ClassMapper`), `children` (nested elements), `visible` toggle
- **Shortcut**: Keyboard shortcut binding (`keys` → `action`)
- **Modal**: Dialog overlay with `title` and `elements`

---

## 4. ActionBus Mechanism

`ActionBus` is a Dioxus context provided at app root that manages all UI state.

### Provided Context

```rust
pub struct ActionBus {
    pub dispatch: Signal<VecDeque<AppAction>>,    // Action queue
    pub bindings: Signal<HashMap<String, String>>, // Form field bindings
    pub navigate: Signal<Option<NavigateAction>>,  // Navigation queue
    pub current_route: Signal<String>,               // Current page route
    pub current_modal: Signal<Option<String>>,       // Active modal ID
    pub theme_mode: Signal<ThemeMode>,               // Light/Dark
}
```

### Core API

| Method | Purpose |
|--------|---------|
| `dispatch(action)` | Queue an action for processing |
| `set_binding(key, value)` | Update form field value |
| `get_binding(key)` | Read form field value |
| `navigate(route, params)` | Request page navigation |
| `open_modal(modal_id)` | Show modal dialog |
| `close_modal()` | Hide active modal |
| `toggle_theme()` | Switch light/dark mode |
| `pop_action()` | Dequeue next action (returns `Option<AppAction>`) |

### Action Processing

`ActionProcessor` component (main.rs:66-84) watches the dispatch queue and processes actions:

```rust
fn handle_action(bus: &mut ActionBus, action: AppAction) {
    match action.name.as_str() {
        "translate" => { /* read bindings, call TranslationService */ }
        "add_term" => { /* glossary logic */ }
        "toggle_theme" => { bus.toggle_theme(); }
        "swap_languages" => { /* swap source/target bindings */ }
        // ...
    }
}
```

### Bound Elements

Schema elements with `binding` prop automatically sync to ActionBus:
- `action-input` — text input bound to `binding` key
- `action-textarea` — textarea bound to `binding` key
- `action-select` — select bound to `binding` key

---

## 5. Component Types Reference

`DynamicRenderer` matches `component` string to render arms:

| Schema `component` | Renderer Arm | Purpose |
|-------------------|--------------|---------|
| `"div"` | `Div` arm | Container with children |
| `"text"` | `Text` arm | Static text display |
| `"button"` | `Button` arm | Navigation or action button |
| `"select"` | `Select` arm | Dropdown (reads/writes binding) |
| `"textarea"` | `Textarea` arm | Multi-line input (reads/writes binding) |
| `"input"` | `Input` arm | Text input |
| `"card"` | `Card` arm | Card container |
| `"badge"` | `Badge` arm | Badge label |
| `"action-button"` | `action-button` arm | **Interactive** — dispatches named action on click |
| `"action-select"` | `action-select` arm | **Interactive** — updates binding on change |
| `"action-textarea"` | `action-textarea` arm | **Interactive** — updates binding on input |
| `"action-input"` | `action-input` arm | **Interactive** — updates binding on input |
| Unknown | Default arm | Renders error box with component name |

### Props for Interactive Components

**action-button:**
- `label`: Button text
- `action`: Name of action to dispatch (e.g., `"translate"`, `"swap_languages"`)

**action-select / action-textarea / action-input:**
- `binding`: Key in ActionBus bindings map
- `options` (select only): Array of option strings

### Dark Mode

All components detect `dark_mode: bool` prop and resolve classes via `ClassMapper`
(theme-aware semantic tokens → concrete Tailwind classes). Interactive helper
classes (`get_input_classes`, `get_btn_classes`, `get_surface_classes`, etc.) map
schema semantic tokens (`btn-filled`, `btn-tonal`, `input-base`, `surface-container`,
`divider`, …) through `ClassMapper::map_all(extra)`.

---

## 6. Development Commands

### Build & Run

```bash
# Check code (DO NOT use cargo build for verification)
cd Translator && cargo check

# Run development
cd Translator && cargo run

# Check shared library
cd dioxus-shared && cargo check
```

### MCP Bridge

The app starts an MCP Bridge on `ws://127.0.0.1:9223` automatically via `dioxus_plugin_mcp_bridge`.

### Verification

```bash
cargo check --manifest-path Translator/Cargo.toml
cargo check --manifest-path dioxus-shared/Cargo.toml
cargo clippy --manifest-path Translator/Cargo.toml
```

---

## 7. Domain Layer

Located in `src/domain/`.

### Entities

**Translation** (`translation.rs`):
```rust
pub struct Language { pub code: String, pub name: String }
pub struct LanguagesResponse { pub languages: Vec<Language> }
pub struct TranslationRequest { pub text, pub source_lang, pub target_lang }
pub struct TranslationResponse { pub translated_text: String }
pub struct Translation { pub id, pub source_text, pub target_text, pub source_lang, pub target_lang, pub created_at }

pub trait TranslationService {
    fn get_supported_languages(&self) -> dioxus_shared::Response<LanguagesResponse>;
    fn translate(&mut self, text: &str, source_lang: &str, target_lang: &str)
        -> Result<dioxus_shared::Response<TranslationResponse>, dioxus_shared::AppError>;
}
```

**GlossaryItem** (`glossary.rs`): Term/translation pair with metadata.

**HistoryEntry** (`history.rs`): Past translation record with timestamp.

**AppSettings** (`settings.rs`): User preferences (default languages, theme).

### Application Service

`TranslationService` (`src/application/translation_service.rs`) delegates to infrastructure:
```rust
impl TranslationService {
    pub fn translate(text: &str, source_lang: &str, target_lang: &str)
        -> Result<dioxus_shared::Response<TranslationResponse>, dioxus_shared::AppError> {
        let backend = get_translation_backend();
        backend.write().unwrap().translate(text, source_lang, target_lang)
    }
}
```

---

## 8. Infrastructure Layer

Located in `src/infrastructure/`.

### Translation Backend (`translation.rs`)

Uses the `trad` crate for translation:
- Lazy-initializes translator on first use
- Supports 15+ languages via `trad::languages::*`
- Thread-local Tokio runtime for async translation
- Returns `Response<TranslationResponse>` / `AppError` (data-first envelope)

### Storage

- **Glossary**: In-memory `Vec<GlossaryItem>` (placeholder)
- **History**: In-memory `Vec<HistoryEntry>` (placeholder)
- **Settings**: JSON file persistence via `dirs` crate

---

## 9. Adding a New Page

**No code changes required.** Edit `schemas/translator.json` (canonical source:
`/mnt/Other/Projects/schemas/translatorschemas.json`):

```json
{
  "pages": [
    {
      "id": "my-new-page",
      "title": "My New Page",
      "route": "/my-new-page",
      "layout": "stacked",
      "elements": [
        {
          "id": "header",
          "component": "div",
          "classes": "p-4 surface-container",
          "children": [
            { "id": "title", "component": "text", "classes": "text-xl", "props": { "text": "My New Page" } }
          ]
        }
      ]
    }
  ]
}
```

Then add navigation buttons pointing to `/my-new-page`. The updated schema is
picked up on next launch (schema is read from the JSON DB via `nosql_orm`).

---

## 10. Adding a New Component Type

Edit `dioxus-shared/src/ui/components/dynamic_renderer.rs`:

Add a new arm to the `match component.as_str()` block:

```rust
"my-component" => {
    let custom_prop = element.props.get("custom")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let my_class = if dark_mode { "bg-gray-800" } else { "bg-gray-100" };

    rsx! {
        div { class: "{my_class}",
            for child in element.children.iter() {
                DynamicRenderer { element: child.clone(), dark_mode: dark_mode }
            }
        }
    }
}
```

Then use in schema:
```json
{ "component": "my-component", "props": { "custom": "value" } }
```

---

## 11. Key Differences from Tauri/Angular Version

| Aspect | Old (Tauri + Angular) | Current (Dioxus Desktop) |
|--------|----------------------|-------------------------|
| Framework | Tauri v2 + Angular 22 | Dioxus 0.8 Desktop |
| UI Rendering | Angular components + TailwindCSS v4 | SDUI via DynamicPage/DynamicRenderer |
| Schema | Stored in JSON DB at `~/.local/share/...` | JSON DB via `nosql_orm`; seeded from `schemas/translator.json` |
| State Management | Angular signals + Tauri events | ActionBus context (Signals) |
| Styling | Semantic props → CSS classes | Schema `classes` (semantic tokens) → `ClassMapper` → Tailwind |
| Navigation | Angular Router | ActionBus.navigate() → route signal |
| Dark Mode | html.dark class + ThemeService | ActionBus.toggle_theme() + dark_mode prop |
| Translation | Tauri command → async event | Direct Rust function call |
| Build | `bun run tauri:build` | `cargo run` in Translator dir |
| MCP Tools | `tauri-mcp_*` tools | Not applicable (no running Tauri) |

### ActionBus vs Tauri Events

Old: Tauri commands + `window.emit()` + `listen()`
New: `ActionBus.dispatch()` → `ActionProcessor` watches queue → `handle_action()`

### Schema Classes

Old Angular version had semantic props (layout, gap, etc.) that transformed to CSS at runtime. Current version uses semantic layout classes plus style-free semantic tokens (e.g. `btn-filled`, `btn-tonal`, `input-base`, `surface-container`, `divider`) that `ClassMapper` resolves to theme-aware Tailwind classes. Both support dark mode via theme switching. Raw Tailwind/hex color classes in schema are forbidden (Rule 5).

---

## General Rules

- **No test files** unless explicitly requested
- **snake_case** for Rust functions/variables
- **PascalCase** for Rust structs/traits
- Schema `classes` uses semantic names (layout classes + `ClassMapper` tokens), never raw Tailwind/hex colors
- All schema/DB access goes through `nosql_orm` via `dioxus-shared` (Rule 6)
- `Response<T>` is data-first; errors use `dioxus_shared::AppError`
- Always `cargo check` after changes — never `cargo build` for verification
