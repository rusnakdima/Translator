//! Translator Dioxus Application - Schema-Driven UI
//!
//! This app uses schema-driven UI via `dioxus-shared::DynamicPage`.
//! All UI is generated from schema JSON - NO hardcoded UI components.
//!
//! Architecture:
//! - `domain/` - Translation entities and traits
//! - `application/` - AppState wiring
//! - `infrastructure/` - TranslationBackend (trad crate)
//! - UI is generated from schema via `dioxus_shared::DynamicPage`

pub mod application;
pub mod bridge;
pub mod domain;
pub mod infrastructure;

// Re-exports for convenience
pub use application::AppState;
pub use domain::*;
