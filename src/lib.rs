//! Translator Dioxus Application - Schema-Driven UI
//!
//! This app uses schema-driven UI via `dioxus-shared::DynamicPage`.
//! All UI is generated from schema JSON - NO hardcoded UI components.
//!
//! Architecture:
//! - `domain/` - Translation entities
//! - `application/` - TranslationService
//! - `infrastructure/` - TranslationBackend (trad crate)
//! - UI is generated from schema via `dioxus_shared::DynamicPage`

pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod bridge;

// Re-export for convenience
pub use application::TranslationService;
pub use domain::*;
