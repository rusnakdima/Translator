# AGENTS.md - Development Guide for Translator

This document is the primary source of truth for agentic coding agents operating within the Translator repository. It outlines project structure, coding standards, and operational commands.

## 1. Project Overview

- **Framework:** Tauri v2 (Rust backend + Angular frontend)
- **Tauri:** v2.10
- **Angular:** v22.0.6
- **Package Manager:** npm/bun
- **Rust Edition:** 2021
- **Key Dependencies:** `tauri-plugin-opener`, `serde`, `serde_json`, `chrono`, `dirs`, `paste`

## 2. Operational Commands

### Development

- **Start Dev Environment:** `bun run tauri:dev`
- **Start Angular Only:** `bun run start` (available at http://localhost:1420)

### Build Commands (CORRECT)

- **Rust Check:** `cargo check --manifest-path src-tauri/Cargo.toml` (NEVER cargo build for verification)
- **Rust Build:** `cargo build --manifest-path src-tauri/Cargo.toml` (only for actual builds)
- **Rust Build Release:** `cargo build --release --manifest-path src-tauri/Cargo.toml`
- **Build Application:** `bun run tauri:build`
- **Build Angular Only:** `bun run build`

### Verification

- **Rust Check:** `cargo check --manifest-path src-tauri/Cargo.toml`
- **Rust Lint:** `cargo clippy --manifest-path src-tauri/Cargo.toml`
- **Rust Test:** `cargo test --manifest-path src-tauri/Cargo.toml`

---

## 3. Backend (Rust) Standards

Located in `src-tauri/src/`.

### Directory Structure

```
src-tauri/src/
  helpers/         # Utility helpers (e.g., translator engine)
  models/          # Data structures
    response/      # Response and Status types
  services/        # Business logic
  utils/           # Re-exports for response utilities
  lib.rs           # Command registration and app entry
  main.rs          # Binary entry point
```

### Naming Conventions

- **Files:** `<kebab-case>.<singular-folder-derivative>.rs` (e.g., `translation.model.rs`)
- **Structs/Traits:** PascalCase (e.g., `TranslationService`, `LanguagesResponse`)
- **Struct Fields:** **camelCase** (required for Angular frontend compatibility)
- **Functions/Variables:** **snake_case**

### Response System — CRITICAL

All Tauri commands MUST return `Result<Response<T>, String>`. The `String` in the `Err` variant is a legacy artifact — prefer returning `Response::error(...)` wrapped in `Ok`.

#### Response Struct

```rust
// src-tauri/src/models/response/response.rs
pub struct Response<T = serde_json::Value> {
    pub status: Status,
    pub message: String,
    pub data: T,
}
```

#### Status Enum Variants

```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Success,
    Created,
    Updated,
    Deleted,
    Error,
    ValidationError,
    NotFound,
    Unauthorized,
    Forbidden,
}
```

#### Response::success Signature

```rust
impl<T> Response<T> {
    pub fn success(message: impl Into<String>, data: T) -> Self
    // Usage:
    Response::success("Operation completed", my_data)
}
```

#### Response::error Signature

```rust
impl Response<serde_json::Value> {
    pub fn error(status: Status, message: impl Into<String>) -> Self
    pub fn error_with_data(message: impl Into<String>, data: T) -> Self
    pub fn validation_error(message: impl Into<String>) -> Self
    pub fn not_found(entity: &str) -> Self
    pub fn unauthorized() -> Self
    pub fn forbidden() -> Self
    // Usage:
    Response::error(Status::NotFound, "Language not found")
    Response::error_with_data("Translation failed", error_data)
}
```

#### Command Return Type Pattern

```rust
#[tauri::command]
fn my_command(...) -> Result<Response<MyData>, String> {
    Ok(Response::success("Message", my_data))
    // OR for errors (preferred over Err(String)):
    Ok(Response::error(Status::NotFound, "Not found"))
}
```

---

## 4. Frontend (Angular) Standards

Located in `src/app/`.

### Directory Structure

- `components/`: Reusable UI elements
- `views/`: Page-level structures
- `models/`: TypeScript interfaces mirroring Rust structs
- `services/`: API interaction and state management

### Key Angular Patterns

- **Signals** for reactive state (`signal`, `computed`, `effect`)
- **Standalone components** only (no `NgModule`)
- **Control Flow:** `@if`, `@for`, `@switch`
- **camelCase** for TypeScript fields matching Rust structs

### Translation-Specific Notes

- The `translate_text` command is **asynchronous** — results arrive via Tauri events.
- Listen for the `translation-result` event on the window:

```typescript
import { listen } from '@tauri-apps/api/event';
// Payload shape:
interface TranslationResultPayload {
  requestId: number;
  text: string;
  sourceLang: string;
  targetLang: string;
  response: Response<TranslationResponse>;
}
```

---

## 5. tauri-mcp Tool Usage (CRITICAL)

**CONNECTING TO RUNNING APPS ONLY**:
- `tauri-mcp_driver_session`: Connect to ALREADY RUNNING Tauri app
- **NEVER** kill a Tauri process with this tool
- **NEVER** try to start/run the app - frontend won't be running
- The app must be started separately (e.g., by user or another process)
- After connecting, you can use `tauri-mcp_webview_*` and `tauri-mcp_ipc_*` tools

**Workflow**:
1. User starts the Tauri app separately (`bun run tauri:dev` or similar)
2. Agent connects via `tauri-mcp_driver_session` with action: "start"
3. Agent uses webview/ipc tools to interact
4. **NEVER** use driver_session to stop/kill the app process

**Available Tools**:
- `tauri-mcp_driver_session` - Connect to running app (action: "start")
- `tauri-mcp_webview_dom_snapshot` - Get UI structure
- `tauri-mcp_webview_find_element` - Find elements
- `tauri-mcp_webview_interact` - Click, type, scroll
- `tauri-mcp_ipc_execute_command` - Call Rust backend commands
- `tauri-mcp_ipc_monitor` - Monitor IPC calls
- `tauri-mcp_manage_window` - Window management

### Connection

```bash
tauri-mcp_driver_session(action: "start", appIdentifier: "<port or bundle ID>")
```

### Available Tools

| Tool | Purpose |
|------|---------|
| `tauri-mcp_webview_dom_snapshot` | Get full DOM/accessibility tree |
| `tauri-mcp_webview_screenshot` | Capture viewport screenshot |
| `tauri-mcp_webview_interact` | Click, scroll, swipe, type |
| `tauri-mcp_webview_keyboard` | Send key events |
| `tauri-mcp_webview_find_element` | Locate elements by selector |
| `tauri-mcp_ipc_execute_command` | Invoke Tauri IPC commands |
| `tauri-mcp_ipc_emit_event` | Emit Tauri events to frontend |
| `tauri-mcp_ipc_monitor` | Capture IPC traffic |
| `tauri-mcp_read_logs` | Read console/android/system logs |

### Key Notes

- `tauri-mcp_driver_session` MUST be active before using any `webview_*` or `ipc_*` tools
- `tauri-plugin-mcp-bridge` is **not** included in Translator's Tauri build (no `devtools` feature)
- For this project, prefer `tauri-mcp_ipc_execute_command` for testing commands

---

## 6. Project-Specific Patterns

### Only 2 Commands Exist

| Command | Signature | Behavior |
|---------|-----------|----------|
| `get_supported_languages` | `State<'_, TranslationService>` → `Response<LanguagesResponse>` | Synchronous |
| `translate_text` | `State + Window` → `Result<usize, String>` | Returns `request_id` immediately; result emitted via event |

### Event Emission Pattern

```rust
// translate_text spawns an async task and emits via window:
tauri::async_runtime::spawn(async move {
    let response = service.translate(&text, &source, &target);
    let payload = serde_json::json!({ ... });
    let _ = window.emit(TAURI_EVENT_TRANSLATION_RESULT, payload);
});
// Returns request_id so frontend can correlate the event
```

### State Management

`TranslationService` is managed as a **singleton** via `tauri::State`:

```rust
.manage(TranslationService::default())
```

### Settings/CRUD

No CRUD commands exist. Settings are not persisted in this project.

---

## 7. General Rules

- **No `ng generate`**: Create files manually
- **No test files** unless explicitly requested
- **camelCase** for all serialized struct fields (Rust `#[serde(rename_all = "camelCase")]` and TypeScript interfaces)
- **2-space indent** for HTML/CSS
- Use path aliases (`@components/*`, `@services/*`) as configured in `tsconfig.json`

---

## 8. Style System

### Schema NEVER Contains CSS Classes

Schema uses **semantic props only** — never CSS classes.

```json
{
  "id": "content-grid",
  "componentId": "div",
  "props": { "layout": "grid", "gap": "md", "columns": "1fr auto 1fr" }
}
```

**Why?**
- Schema is shared between Designer and runtime — CSS classes couple schema to implementation
- Props are semantic; themes transform them to CSS at render time
- Dark mode: Props stay the same, only the theme changes

### Schema Lives in JSON DB — NOT in Project Repo

The schema is stored in the JSON database only:
`~/.local/share/com.tcs.translator/translator_db/schemas.json`

**Never commit schema files in project repositories.** Each app manages its own schema via the DB.

### Schema Props Reference

| Prop | Values | Purpose |
|------|--------|---------|
| `styleName` | Named style (e.g. `"solid"`, `"ghost-sm"`) | Global named style lookup in theme registry |
| `layout` | `flex`, `grid`, `stack`, `flow` | Layout mode |
| `direction` | `row`, `col`, `row-reverse`, `col-reverse` | Flex direction |
| `gap` | `xs`, `sm`, `md`, `lg`, `xl` | Spacing scale |
| `columns` | CSS grid column string | Grid columns |
| `align` | `start`, `center`, `end`, `stretch` | Alignment |
| `justify` | `start`, `center`, `end`, `between`, `around` | Justification |
| `padding` | `none`, `xs`, `sm`, `md`, `lg`, `xl` | Padding scale |
| `marginTop`, `marginBottom` | `xs`, `sm`, `md`, `lg`, `xl` | Margin spacing |
| `maxWidth` | `sm`, `md`, `lg`, `xl`, `2xl`, `6xl` | Max width |
| `mx` | `auto` | Horizontal margin auto |
| `fullHeight` | `true` | Full viewport height |
| `rounded` | `true` | Rounded corners |
| `visible` | `true`, `false` | Visibility |
| `i18nKey` | dot-notation string | i18n translation key |
| `placeholder_i18n` | dot-notation string | Placeholder translation key |

### TailwindCSS v4 with `@theme {}`

All styling comes from `@tauri-front/shared` themes via TailwindCSS v4 `@theme {}` directive.

```css
/* CORRECT — theme-based styling */
.my-wrapper { @apply flex flex-col gap-4 p-6 bg-base-100 rounded-lg; }

/* WRONG — raw CSS properties */
.my-wrapper { display: flex; flex-direction: column; gap: 1rem; }
```

### Dark Mode

- **Trigger**: `html.dark` class on the `<html>` element
- **Toggle**: `ThemeService.toggleDarkMode()`

```typescript
import { ThemeService } from '@tauri-front/shared';
ThemeService.toggleDarkMode();
```

### Loading Themes

Apps import themes via `StyleThemeService.loadTheme(variant)`:

```typescript
import { StyleThemeService } from '@tauri-front/shared';

StyleThemeService.loadTheme('default');  // Light
StyleThemeService.loadTheme('dark');     // Dark
```

### Global Style Variants

Schema supports global `variant` and `size` keys in the `app` section that automatically apply to all UI components:

```json
{
  "app": {
    "variant": "ghost",
    "size": "sm"
  },
  "pages": [...]
}
```

**How it works:**
- `variant` sets the base style (e.g., `"ghost"`, `"solid"`, `"text"`)
- `size` sets the size modifier (e.g., `"sm"`, `"md"`, `"lg"`)
- All components inherit these global defaults automatically
- Per-element `variant` and/or `size` props override the global defaults

**App config example:**
```json
{
  "app": {
    "id": "translator_schema",
    "name": "translator",
    "style": "material-design-v3",
    "variant": "ghost",
    "size": "sm"
  },
  "pages": [
    {
      "id": "main",
      "elements": [
        {
          "componentId": "app-button",
          "props": {
            "variant": "solid",
            "icon": "keyboard"
          }
        }
      ]
    }
  ]
}
```

---

## 9. Migration Guide: Old Style System → New Style System

### Overview

The new style system moves from **CSS classes in schema** to **semantic props in schema + TailwindCSS v4 themes**.

### Step 1: Update Schema

**Before (OLD):**
```json
{
  "componentId": "div",
  "classes": "grid gap-4 md:grid-cols-[1fr_auto_1fr] items-stretch p-6 bg-base-100 rounded-lg"
}
```

**After (NEW):**
```json
{
  "componentId": "div",
  "props": {
    "layout": "grid",
    "gap": "md",
    "columns": "1fr auto 1fr",
    "align": "stretch",
    "padding": "md",
    "rounded": true
  }
}
```

### Step 1b: Use styleName for Component Styling

**Before (OLD):** Individual variant/size props
```json
{
  "componentId": "app-button",
  "props": {
    "buttonStyle": "ghost",
    "variant": "primary",
    "size": "sm",
    "icon": "keyboard",
    "i18nKey": "header.shortcuts"
  }
}
```

**After (NEW):** Named style from global registry
```json
{
  "componentId": "app-button",
  "props": {
    "styleName": "ghost-sm",
    "icon": "keyboard",
    "i18nKey": "header.shortcuts"
  }
}
```

Available `styleName` values per theme are defined in `@tauri-front/shared` `StyleVariantConfig.componentStyles`. Common names: `"solid"`, `"ghost"`, `"text"`, `"icon"`, `"solid-sm"`, `"ghost-sm"`, `"text-sm"`, `"icon-sm"`.

### Step 2: Remove Inline Classes

**Before (OLD):**
```html
<div class="flex flex-col gap-4 p-6 bg-base-100 rounded-lg">
  <app-button class="w-full"></app-button>
</div>
```

**After (NEW):**
```html
<div class="schema-props-target">
  <app-button></app-button>
</div>
```

```css
.schema-props-target { @apply flex flex-col gap-4 p-6 bg-base-100 rounded-lg; }
```

### Step 3: Use ThemeService for Dark Mode

**Before (OLD):**
```typescript
document.documentElement.classList.toggle('dark');
```

**After (NEW):**
```typescript
import { ThemeService } from '@tauri-front/shared';
ThemeService.toggleDarkMode();
```

### Step 4: Load Themes at Startup

```typescript
import { StyleThemeService } from '@tauri-front/shared';
StyleThemeService.loadTheme('default');
```

### Step 5: Replace Raw CSS Properties

**Before (OLD):**
```css
.my-component {
  --my-bg: #f5f5f5;
  background: var(--my-bg);
  display: flex;
  flex-direction: column;
}
```

**After (NEW):**
```css
.my-component {
  @apply bg-base-100 flex flex-col;
}
```

### Verification Checklist

- [ ] Schema contains no `classes` field — only `props`
- [ ] Schema lives in JSON DB (`~/.local/share/<bundle_id>/schemas.json`) — not committed in project repo
- [ ] All styling uses `@apply` with theme tokens
- [ ] Dark mode uses `ThemeService.toggleDarkMode()`
- [ ] Theme loaded via `StyleThemeService.loadTheme()`
- [ ] No raw CSS properties (display, flex-direction, padding, etc.)
- [ ] Component styles use `styleName` prop (named style lookup) — no `variant`/`size`/`buttonStyle` props
