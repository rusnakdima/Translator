//! Application layer - Translation service

use crate::domain::{LanguagesResponse, TranslationResponse};
use crate::infrastructure::get_translation_backend;
use dioxus_shared::{AppError, Response};

/// Application service for translation operations.
pub struct TranslationService;

impl TranslationService {
    /// Get all supported languages.
    pub fn get_supported_languages() -> Response<LanguagesResponse> {
        let backend = get_translation_backend();
        backend.read().unwrap().get_supported_languages()
    }

    /// Translate text from source to target language.
    pub fn translate(
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Response<TranslationResponse>, AppError> {
        let backend = get_translation_backend();
        backend
            .write()
            .unwrap()
            .translate(text, source_lang, target_lang)
    }
}
