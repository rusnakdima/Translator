//! Domain layer - Translation entities

use chrono::{DateTime, Utc};
use dioxus_shared::{AppError, Response};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Language {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguagesResponse {
    pub languages: Vec<Language>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationRequest {
    pub text: String,
    pub source_lang: String,
    pub target_lang: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResponse {
    pub translated_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Translation {
    pub id: String,
    pub source_text: String,
    pub target_text: String,
    pub source_lang: String,
    pub target_lang: String,
    pub created_at: DateTime<Utc>,
}

pub trait TranslationService {
    fn get_supported_languages(&self) -> Response<LanguagesResponse>;
    fn translate(
        &mut self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Response<TranslationResponse>, AppError>;
}

/// Supported languages for translation (single source of truth)
pub const SUPPORTED_LANGUAGES: &[(&str, &str)] = &[
    ("en", "English"),
    ("es", "Spanish"),
    ("fr", "French"),
    ("de", "German"),
    ("it", "Italian"),
    ("pt", "Portuguese"),
    ("ru", "Russian"),
    ("zh", "Chinese"),
    ("ja", "Japanese"),
    ("ko", "Korean"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_serde_roundtrip() {
        let lang = Language {
            code: "fr".to_string(),
            name: "French".to_string(),
        };
        let json = serde_json::to_string(&lang).expect("should serialize");
        let roundtrip: Language = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(lang.code, roundtrip.code);
        assert_eq!(lang.name, roundtrip.name);
    }

    #[test]
    fn translation_response_serde_roundtrip() {
        let resp = TranslationResponse {
            translated_text: "bonjour".to_string(),
        };
        let json = serde_json::to_string(&resp).expect("should serialize");
        let roundtrip: TranslationResponse =
            serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(resp.translated_text, roundtrip.translated_text);
    }

    #[test]
    fn translation_entity_serde_roundtrip() {
        use chrono::Utc;
        let tx = Translation {
            id: "tx-99".to_string(),
            source_text: "hello".to_string(),
            target_text: "hola".to_string(),
            source_lang: "en".to_string(),
            target_lang: "es".to_string(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&tx).expect("should serialize");
        let roundtrip: Translation = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(tx.id, roundtrip.id);
        assert_eq!(tx.source_text, roundtrip.source_text);
        assert_eq!(tx.target_lang, roundtrip.target_lang);
    }
}
