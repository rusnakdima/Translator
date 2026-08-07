//! Infrastructure layer - Settings service implementation
//!
//! Settings live in the shared `SignalStore` (key: `settings.value`) and are
//! mirrored to a JSON file on disk so they survive restarts. The `SignalStore`
//! remains the source of truth for in-process reads; `save_settings` flushes
//! to disk on every write.

use crate::domain::{AppSettings, SettingsService};
use dioxus_shared::storage::SignalStore;
use dioxus_shared::AppError;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

pub struct SettingsStorage {
    store: Arc<SignalStore>,
    config_path: PathBuf,
}

impl SettingsStorage {
    pub fn new(store: Arc<SignalStore>) -> Self {
        let config_path = get_settings_path();
        let storage = Self { store, config_path };
        // Hydrate from disk on startup.
        if let Ok(content) = fs::read_to_string(&storage.config_path) {
            if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                if let Ok(v) = serde_json::to_value(&settings) {
                    storage.store.set("settings.value", v);
                }
            }
        }
        storage
    }

    /// Create a SettingsStorage with an explicit file path (for test isolation).
    #[cfg(test)]
    pub(crate) fn with_path(store: Arc<SignalStore>, path: PathBuf) -> Self {
        let storage = Self {
            store,
            config_path: path,
        };
        // Hydrate from disk on startup (same as new()).
        if let Ok(content) = fs::read_to_string(&storage.config_path) {
            if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                if let Ok(v) = serde_json::to_value(&settings) {
                    storage.store.set("settings.value", v);
                }
            }
        }
        storage
    }

    fn ensure_dir(&self) -> Result<(), AppError> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::Io(format!("Failed to create config directory: {e}")))?;
        }
        Ok(())
    }
}

pub fn get_settings_path() -> PathBuf {
    dirs::home_dir()
        .map(|h| h.join(".local/share/com.tcs.translator/settings.json"))
        .unwrap_or_else(|| PathBuf::from("settings.json"))
}

impl SettingsService for SettingsStorage {
    fn load_settings(&self) -> AppSettings {
        match self.store.get("settings.value") {
            Some(v) => serde_json::from_value(v).unwrap_or_default(),
            None => AppSettings::default(),
        }
    }

    fn save_settings(&self, settings: &AppSettings) -> Result<(), AppError> {
        self.ensure_dir()?;
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| AppError::Io(format!("Failed to serialize settings: {e}")))?;
        fs::write(&self.config_path, content)
            .map_err(|e| AppError::Io(format!("Failed to write settings file: {e}")))?;
        self.store.set("settings.value", json!(settings));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::AppSettings;
    use tempfile::TempDir;

    fn make_store() -> Arc<SignalStore> {
        Arc::new(SignalStore::new())
    }

    #[test]
    fn settings_with_path_empty_initially() {
        let store = make_store();
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("settings.json");
        let storage = SettingsStorage::with_path(store, path);
        assert_eq!(storage.load_settings(), AppSettings::default());
    }

    #[test]
    fn settings_save_persists_to_disk() {
        let store = make_store();
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("settings.json");
        let storage = SettingsStorage::with_path(store.clone(), path.clone());

        let settings = AppSettings {
            default_source_lang: "fr".to_string(),
            default_target_lang: "de".to_string(),
            auto_detect: false,
            cache_enabled: false,
            batch_size: 5,
            theme: "dark".to_string(),
            theme_variant: "material-design-v3".to_string(),
        };
        storage
            .save_settings(&settings)
            .expect("save should succeed");

        // Verify file contents
        let content = std::fs::read_to_string(&path).expect("file should exist");
        let loaded: AppSettings = serde_json::from_str(&content).expect("should parse");
        assert_eq!(loaded.default_source_lang, "fr");
        assert_eq!(loaded.default_target_lang, "de");
        assert_eq!(loaded.theme, "dark");
    }

    #[test]
    fn settings_reload_after_drop() {
        let store = make_store();
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("settings.json");

        {
            let storage = SettingsStorage::with_path(store.clone(), path.clone());
            let settings = AppSettings {
                default_source_lang: "ja".to_string(),
                default_target_lang: "ko".to_string(),
                auto_detect: true,
                cache_enabled: true,
                batch_size: 20,
                theme: "light".to_string(),
                theme_variant: "glassmorphism".to_string(),
            };
            storage
                .save_settings(&settings)
                .expect("save should succeed");
        }

        // Create new storage pointing to same file
        let store2 = make_store();
        let storage2 = SettingsStorage::with_path(store2, path);
        let loaded = storage2.load_settings();
        assert_eq!(loaded.default_source_lang, "ja");
        assert_eq!(loaded.default_target_lang, "ko");
    }

    #[test]
    fn app_settings_default_stability() {
        let a = AppSettings::default();
        let b = AppSettings::default();
        assert_eq!(a, b);
    }

    #[test]
    fn settings_malformed_json_returns_error_or_default() {
        let store = make_store();
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("settings.json");

        // Write malformed JSON
        std::fs::write(&path, "not valid json at all {{{").unwrap();

        let storage = SettingsStorage::with_path(store, path);
        // load_settings falls back to default on parse error
        let loaded = storage.load_settings();
        assert_eq!(loaded, AppSettings::default());
    }
}
