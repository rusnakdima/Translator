//! Application layer tests - AppState wiring and integration
//!
//! These tests verify the AppState correctly wires all domain services.

#[cfg(test)]
mod tests {
    use crate::application::AppState;
    use crate::domain::{GlossaryItem, GlossaryService, HistoryService, TranslationService};
    use dioxus_shared::storage::SignalStore;
    use dioxus_shared::AlgorithmRegistry;
    use std::sync::Arc;

    fn create_test_app_state() -> AppState {
        let store = Arc::new(SignalStore::new());
        let registry = Arc::new(AlgorithmRegistry::new());
        AppState::new(store, registry)
    }

    // =====================================================================
    // AppState Construction Tests
    // =====================================================================

    #[test]
    fn app_state_creation_succeeds() {
        let state = create_test_app_state();
        // TranslationService is wrapped in Arc<Mutex<...>>, verify we can lock it
        let lock_result = state.translation_service.lock();
        assert!(lock_result.is_ok());
    }

    #[test]
    fn app_state_has_signal_store() {
        let state = create_test_app_state();
        // SignalStore should be shared
        assert!(Arc::ptr_eq(&state.signal_store, &state.signal_store));
    }

    #[test]
    fn app_state_clone_shares_services() {
        let state1 = create_test_app_state();
        let state2 = state1.clone();

        // Arcs should point to same data
        assert!(Arc::ptr_eq(&state1.signal_store, &state2.signal_store));
        assert!(Arc::ptr_eq(&state1.algorithm_registry, &state2.algorithm_registry));
    }

    // =====================================================================
    // Translation Service Tests (Mock)
    // =====================================================================

    #[test]
    fn translation_service_same_language_returns_unchanged() {
        let state = create_test_app_state();
        let mut svc = state.translation_service.lock().unwrap();

        let result = svc.translate("Hello World", "en", "en");
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.data.is_some());
        assert_eq!(response.data.unwrap().translated_text, "Hello World");
    }

    #[test]
    fn translation_service_empty_text_returns_error() {
        let state = create_test_app_state();
        let mut svc = state.translation_service.lock().unwrap();

        let result = svc.translate("", "en", "es");
        assert!(result.is_err());
    }

    #[test]
    fn translation_service_get_supported_languages() {
        let state = create_test_app_state();
        let mut svc = state.translation_service.lock().unwrap();

        let result = svc.get_supported_languages();
        assert!(result.data.is_some());

        let langs = result.data.unwrap().languages;
        assert!(langs.iter().any(|l| l.code == "en"));
        assert!(langs.iter().any(|l| l.code == "es"));
        assert!(langs.iter().any(|l| l.code == "fr"));
        assert!(langs.iter().any(|l| l.code == "de"));
        assert!(langs.iter().any(|l| l.code == "zh"));
        assert!(langs.iter().any(|l| l.code == "ja"));
    }

    #[test]
    fn translation_service_unsupported_language_returns_error() {
        let state = create_test_app_state();
        let mut svc = state.translation_service.lock().unwrap();

        // "xyz" is not a supported language code
        let result = svc.translate("Hello", "xyz", "en");
        assert!(result.is_err());
    }

    // =====================================================================
    // Glossary Service Tests (Mock/SignalStore)
    // =====================================================================

    #[test]
    fn glossary_service_empty_initially() {
        let state = create_test_app_state();
        let items = state.glossary_service.get_all_items();
        assert!(items.is_empty());
    }

    #[test]
    fn glossary_service_add_item() {
        let state = create_test_app_state();

        let item = state.glossary_service.add_item(
            "keyboard".to_string(),
            "teclado".to_string(),
        );

        assert!(!item.id.is_empty());
        assert_eq!(item.term, "keyboard");
        assert_eq!(item.definition, "teclado");
    }

    #[test]
    fn glossary_service_get_item() {
        let state = create_test_app_state();

        let added = state.glossary_service.add_item(
            "mouse".to_string(),
            "ratón".to_string(),
        );

        let found = state.glossary_service.get_item(&added.id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().term, "mouse");
    }

    #[test]
    fn glossary_service_get_nonexistent_item() {
        let state = create_test_app_state();

        let found = state.glossary_service.get_item("nonexistent-id");
        assert!(found.is_none());
    }

    #[test]
    fn glossary_service_delete_item() {
        let state = create_test_app_state();

        let added = state.glossary_service.add_item(
            "monitor".to_string(),
            "pantalla".to_string(),
        );

        let deleted = state.glossary_service.delete_item(&added.id);
        assert!(deleted);

        let found = state.glossary_service.get_item(&added.id);
        assert!(found.is_none());
    }

    #[test]
    fn glossary_service_delete_nonexistent_returns_false() {
        let state = create_test_app_state();

        let deleted = state.glossary_service.delete_item("nonexistent-id");
        assert!(!deleted);
    }

    #[test]
    fn glossary_service_multiple_items() {
        let state = create_test_app_state();

        state.glossary_service.add_item("hello".to_string(), "hola".to_string());
        state.glossary_service.add_item("goodbye".to_string(), "adiós".to_string());
        state.glossary_service.add_item("please".to_string(), "por favor".to_string());

        let items = state.glossary_service.get_all_items();
        assert_eq!(items.len(), 3);
    }

    // =====================================================================
    // History Service Tests (Mock/SignalStore)
    // =====================================================================

    #[test]
    fn history_service_empty_initially() {
        let state = create_test_app_state();
        let entries = state.history_service.get_all_entries();
        assert!(entries.is_empty());
    }

    #[test]
    fn history_service_add_entry() {
        let state = create_test_app_state();

        let entry = state.history_service.add_entry(
            "tx-123".to_string(),
            "Hello world".to_string(),
        );

        assert!(!entry.id.is_empty());
        assert_eq!(entry.translation_id, "tx-123");
        assert_eq!(entry.query, "Hello world");
    }

    #[test]
    fn history_service_get_entry() {
        let state = create_test_app_state();

        let added = state.history_service.add_entry(
            "tx-456".to_string(),
            "Good morning".to_string(),
        );

        let found = state.history_service.get_entry(&added.id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().query, "Good morning");
    }

    #[test]
    fn history_service_clear_history() {
        let state = create_test_app_state();

        state.history_service.add_entry("tx-1".to_string(), "one".to_string());
        state.history_service.add_entry("tx-2".to_string(), "two".to_string());

        state.history_service.clear_history();

        let entries = state.history_service.get_all_entries();
        assert!(entries.is_empty());
    }

    #[test]
    fn history_service_delete_entry() {
        let state = create_test_app_state();

        let added = state.history_service.add_entry(
            "tx-789".to_string(),
            "Thank you".to_string(),
        );

        let deleted = state.history_service.delete_entry(&added.id);
        assert!(deleted);

        let found = state.history_service.get_entry(&added.id);
        assert!(found.is_none());
    }

    #[test]
    fn history_service_multiple_entries_order() {
        let state = create_test_app_state();

        state.history_service.add_entry("tx-a".to_string(), "first".to_string());
        std::thread::sleep(std::time::Duration::from_millis(1));
        state.history_service.add_entry("tx-b".to_string(), "second".to_string());
        std::thread::sleep(std::time::Duration::from_millis(1));
        state.history_service.add_entry("tx-c".to_string(), "third".to_string());

        let entries = state.history_service.get_all_entries();
        assert_eq!(entries.len(), 3);
        // Most recent should be last
        assert_eq!(entries.last().unwrap().translation_id, "tx-c");
    }

    // =====================================================================
    // SignalStore Integration Tests
    // =====================================================================

    #[test]
    fn signal_store_persists_glossary_items() {
        let store = Arc::new(SignalStore::new());
        let registry = Arc::new(AlgorithmRegistry::new());

        // Create first state and add item
        let state1 = AppState::new(store.clone(), registry.clone());
        state1.glossary_service.add_item("test".to_string(), "prueba".to_string());

        // Create new state with same store
        let state2 = AppState::new(store, registry);

        // Item should persist
        let items = state2.glossary_service.get_all_items();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].term, "test");
    }

    #[test]
    fn signal_store_persists_history_entries() {
        let store = Arc::new(SignalStore::new());
        let registry = Arc::new(AlgorithmRegistry::new());

        // Create first state and add entry
        let state1 = AppState::new(store.clone(), registry.clone());
        state1.history_service.add_entry("tx-persist".to_string(), "persistent".to_string());

        // Create new state with same store
        let state2 = AppState::new(store, registry);

        // Entry should persist
        let entries = state2.history_service.get_all_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].query, "persistent");
    }

    // =====================================================================
    // Send + Sync Safety Tests
    // =====================================================================

    #[test]
    fn app_state_is_send_and_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        let state = create_test_app_state();
        assert_send::<AppState>();
        assert_sync::<AppState>();

        // Verify services are also Send + Sync
        assert_send::<Arc<dyn TranslationService>>();
        assert_sync::<Arc<dyn TranslationService>>();
        assert_send::<Arc<dyn GlossaryService>>();
        assert_sync::<Arc<dyn GlossaryService>>();
        assert_send::<Arc<dyn HistoryService>>();
        assert_sync::<Arc<dyn HistoryService>>();
    }
}
