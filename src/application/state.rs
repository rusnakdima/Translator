//! Application state - Dependency injection for DDD
//!
//! This module wires domain traits to their infrastructure implementations.

use crate::domain::{GlossaryService, HistoryService, TranslationService};
use crate::infrastructure::{GlossaryStorage, HistoryStorage, TranslationBackend};
use dioxus_shared::storage::SignalStore;
use dioxus_shared::AlgorithmRegistry;
use std::sync::{Arc, Mutex};

/// Application state - holds all service implementations
pub struct AppState {
    pub translation_service: Arc<Mutex<dyn TranslationService>>,
    pub glossary_service: Arc<dyn GlossaryService>,
    pub history_service: Arc<dyn HistoryService>,
    pub signal_store: Arc<SignalStore>,
    pub algorithm_registry: Arc<AlgorithmRegistry>,
}

impl AppState {
    /// Create new application state with all services wired
    pub fn new(signal_store: Arc<SignalStore>, algorithm_registry: Arc<AlgorithmRegistry>) -> Self {
        // Translation backend uses Mutex for interior mutability (translate needs &mut self)
        let translation_backend = Arc::new(Mutex::new(TranslationBackend::new().expect("Failed to create translation backend")));

        // Glossary and History use SignalStore with interior mutability
        let glossary_service: Arc<dyn GlossaryService> =
            Arc::new(GlossaryStorage::new(signal_store.clone(), algorithm_registry.clone()));
        let history_service: Arc<dyn HistoryService> =
            Arc::new(HistoryStorage::new(signal_store.clone()));

        Self {
            translation_service: translation_backend,
            glossary_service,
            history_service,
            signal_store,
            algorithm_registry,
        }
    }
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        // Note: Each clone creates a new Arc wrapper but shares the underlying services
        Self {
            translation_service: self.translation_service.clone(),
            glossary_service: self.glossary_service.clone(),
            history_service: self.history_service.clone(),
            signal_store: self.signal_store.clone(),
            algorithm_registry: self.algorithm_registry.clone(),
        }
    }
}
