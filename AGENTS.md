# AGENTS.md - Development Guide for Translator

This document is the primary source of truth for agentic coding agents operating within the Translator repository. It outlines project structure, coding standards, and operational commands.

## 1. Project Overview

- **Framework:** Tauri v2 (Rust backend + Angular frontend)
- **Tauri:** v2.10
- **Angular:** v20.1.4
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
