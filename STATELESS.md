# Translator: Stateless Application Documentation

## Overview

The Translator project is a **stateless translation utility**. It does not persist any data between sessions and does not require any storage layer.

## Why This Project is Stateless

1. **No Data Persistence**: The application performs in-memory translation only. No database, file storage, or cache is used or needed.

2. **External Translation Command**: All translation is delegated to the external `trans` command (translate-shell). The application:
   - Spawns `trans -b -s <source> -t <target>` as a subprocess
   - Pipes input text to stdin
   - Reads translated text from stdout
   - Returns the result immediately without storage

3. **Hardcoded Language List**: Supported languages are defined as a static constant array in `src-tauri/src/helpers/translator.helper.rs:11-27`. No language configuration is loaded from storage.

4. **No Session State**: Each translation request is independent. There is no concept of user sessions, history, or preferences that would require persistence.

## No Refactoring Required

This project does **not** fit the unified storage + nosql_orm pattern because:

- There is no data model requiring persistence
- No CRUD operations on stored entities
- No query or retrieval of previously stored data
- Translation is a pure function: input text + language pair → output text

Adding a storage layer would:
- Introduce unnecessary complexity
- Add dependencies without benefit
- Slow down translation operations
- Require migration of non-existent data

## Architecture Summary

```
Frontend (Angular)
    ↓ Tauri invoke("translate_text")
Backend (Rust/Tauri)
    ↓ translate()
services/translation.service.rs
    ↓ translate()
helpers/translator.helper.rs
    ↓ spawns subprocess
External: trans command
    → STDOUT → translated text
```

## Verification

- No TODO/FIXME/XXX/HACK comments found in source code
- No placeholder code suggesting storage was intended but not implemented
- No database, cache, or persistence imports in any source file
- No `storage`, `persistence`, `db`, or `database` keywords in TypeScript or Rust source files
