//! Infrastructure layer - Translation service implementation
//!
//! Uses the `trad` crate for translation engine.

use crate::domain::{Language, LanguagesResponse, TranslationResponse, SUPPORTED_LANGUAGES};
use dioxus_shared::{AppError, Response};
use std::sync::RwLock;

/// One shared runtime for the async `trad` API; created once, reused for every
/// call so the per-call `tokio::runtime::Builder` allocations are eliminated.
/// Multi-thread so `block_on` works from any thread.
fn translate_runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to build shared translation runtime")
    })
}

/// Translation backend using the `trad` crate
pub struct TranslationBackend {
    translator: Option<trad::Translator>,
}

impl TranslationBackend {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self { translator: None })
    }

    fn ensure_translator(&mut self) -> Result<(), AppError> {
        if self.translator.is_none() {
            let translator = translate_runtime()
                .block_on(trad::Translator::setup(None))
                .map_err(|e| AppError::Internal(format!("Failed to setup translator: {e}")))?;
            self.translator = Some(translator);
        }
        Ok(())
    }

    pub fn get_supported_languages(&self) -> Response<LanguagesResponse> {
        let languages = SUPPORTED_LANGUAGES
            .iter()
            .map(|(code, name)| Language {
                code: (*code).to_string(),
                name: (*name).to_string(),
            })
            .collect::<Vec<_>>();
        Response::success(LanguagesResponse { languages }, Some("Languages retrieved"))
    }

    pub fn translate(
        &mut self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Response<TranslationResponse>, AppError> {
        if text.trim().is_empty() {
            return Err(AppError::ValidationError("Text is empty".to_string()));
        }

        if source_lang == target_lang {
            return Ok(Response::success(
                TranslationResponse {
                    translated_text: text.to_string(),
                },
                Some("Same language"),
            ));
        }

        let source = self.code_to_lang(source_lang)?.to_string();
        let target = self.code_to_lang(target_lang)?.to_string();

        self.ensure_translator()?;

        let translator = self
            .translator
            .as_mut()
            .ok_or_else(|| AppError::Internal("Translator not initialized".to_string()))?;

        let translated = translate_runtime()
            .block_on(translator.translate(text, &source, &target))
            .map_err(|e| AppError::RequestFailed(format!("Translation failed: {e}")))?;

        Ok(Response::success(
            TranslationResponse {
                translated_text: translated,
            },
            Some("Translation completed"),
        ))
    }

    fn code_to_lang(&self, code: &str) -> Result<&'static str, AppError> {
        match code.to_lowercase().as_str() {
            "en" => Ok(trad::languages::ENGLISH),
            "es" => Ok(trad::languages::SPANISH),
            "fr" => Ok(trad::languages::FRENCH),
            "de" => Ok(trad::languages::GERMAN),
            "it" => Ok(trad::languages::ITALIAN),
            "pt" => Ok(trad::languages::PORTUGUESE),
            "ru" => Ok(trad::languages::RUSSIAN),
            "zh" => Ok(trad::languages::CHINESE_SIMPLIFIED),
            "ja" => Ok(trad::languages::JAPANESE),
            "ko" => Ok(trad::languages::KOREAN),
            "nl" => Ok(trad::languages::DUTCH),
            "pl" => Ok(trad::languages::POLISH),
            "tr" => Ok(trad::languages::TURKISH),
            "vi" => Ok(trad::languages::VIETNAMESE),
            "th" => Ok(trad::languages::THAI),
            "id" => Ok(trad::languages::INDONESIAN),
            "cs" => Ok(trad::languages::CZECH),
            "sv" => Ok(trad::languages::SWEDISH),
            "da" => Ok(trad::languages::DANISH),
            "fi" => Ok(trad::languages::FINNISH),
            "uk" => Ok(trad::languages::UKRAINIAN),
            "el" => Ok(trad::languages::GREEK),
            "ro" => Ok(trad::languages::ROMANIAN),
            "hu" => Ok(trad::languages::HUNGARIAN),
            _ => Err(AppError::ValidationError(format!(
                "Unsupported language: {}",
                code
            ))),
        }
    }
}

impl Default for TranslationBackend {
    fn default() -> Self {
        Self::new().expect("Failed to create default translation backend")
    }
}

/// Global translation backend with RwLock for interior mutability
static TRANSLATION_BACKEND: std::sync::OnceLock<RwLock<TranslationBackend>> =
    std::sync::OnceLock::new();

pub fn get_translation_backend() -> &'static RwLock<TranslationBackend> {
    TRANSLATION_BACKEND
        .get_or_init(|| RwLock::new(TranslationBackend::new().expect("Failed to init translator")))
}

/// Detects the language of the given text using character set heuristics.
/// Returns ISO 639-1 language code, falling back to "ru".
pub fn detect_language(text: &str) -> String {
    if text.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c)) {
        return "zh".to_string();
    }
    if text.chars().any(|c| ('\u{3040}'..='\u{30ff}').contains(&c)) {
        return "ja".to_string();
    }
    if text.chars().any(|c| ('\u{AC00}'..='\u{D7AF}').contains(&c)) {
        return "ko".to_string();
    }
    if text.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c)) {
        return "ru".to_string();
    }
    "ru".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_new_does_not_panic_without_model() {
        // new() creates backend with translator: None — no model loaded yet.
        let backend = TranslationBackend::new();
        assert!(backend.is_ok());
    }

    #[test]
    fn translate_same_lang_returns_unchanged() {
        let mut backend = TranslationBackend::new().unwrap();
        let result = backend.translate("hello world", "en", "en");
        assert!(result.is_ok());
        let resp = result.unwrap();
        let data = resp.data.expect("expected data");
        assert_eq!(data.translated_text, "hello world");
    }

    #[test]
    fn get_supported_languages_returns_known_langs() {
        let backend = TranslationBackend::new().unwrap();
        let langs = backend.get_supported_languages();
        assert!(langs.data.is_some());
        let response = langs.data.unwrap();
        assert!(response.languages.iter().any(|l| l.code == "en"));
        assert!(response.languages.iter().any(|l| l.code == "es"));
        assert!(response.languages.iter().any(|l| l.code == "zh"));
    }
}
