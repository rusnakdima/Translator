//! Domain layer - Settings entity

use dioxus_shared::AppError;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    pub default_source_lang: String,
    pub default_target_lang: String,
    pub auto_detect: bool,
    pub cache_enabled: bool,
    pub batch_size: u32,
    pub theme: String,
    pub theme_variant: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_source_lang: "en".to_string(),
            default_target_lang: "es".to_string(),
            auto_detect: true,
            cache_enabled: true,
            batch_size: 10,
            theme: "light".to_string(),
            theme_variant: "material-design-v3".to_string(),
        }
    }
}

pub trait SettingsService {
    fn load_settings(&self) -> AppSettings;
    fn save_settings(&self, settings: &AppSettings) -> Result<(), AppError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_settings_serde_roundtrip() {
        let settings = AppSettings {
            default_source_lang: "de".to_string(),
            default_target_lang: "fr".to_string(),
            auto_detect: false,
            cache_enabled: true,
            batch_size: 25,
            theme: "dark".to_string(),
            theme_variant: "neo-brutalism".to_string(),
        };
        let json = serde_json::to_string(&settings).expect("should serialize");
        let roundtrip: AppSettings = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(settings, roundtrip);
    }
}
