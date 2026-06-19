# Translator v0.5.2

Release version 0.5.2 with code formatting, refactoring, and dependency updates.

---

## Changes

- **Code Formatting** — Applied prettier to frontend and rustfmt to Rust backend for consistent code style
- **Component Restructuring** — Restructured components and removed external logger dependency
- **Internal Logger** — Created internal LoggerService replacing @tauri-apps/logger
- **Shared Structure** — Unified structure with core/services, models/response modules
- **Dependency Update** — Replaced log crate with paste crate

---

## New Features

- **Environment Configuration** — Added support for environment-based configuration settings
- **Tauri API Service** — Integrated Tauri API for native desktop functionality
- **Shared Utilities and Services** — New shared module with common utilities and services
- **Translation Feature Module** — Core translation functionality module

## Bug Fixes

- **Toast notifications** — Improved toast notification system with consolidated types
- **UI component fixes** — Fixed component behavior and improved directive implementation
