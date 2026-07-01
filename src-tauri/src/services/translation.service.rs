use crate::helpers::translator_helper::Translator;
use crate::models::translation_model::{Language, LanguagesResponse, TranslationResponse};
use tauri_shared::Response;
#[derive(Clone, Default)]
pub struct TranslationService {
  translator: Translator,
}
impl TranslationService {
  pub fn get_supported_languages(&self) -> Response<LanguagesResponse> {
    let languages: Vec<Language> = Translator::get_supported_languages()
      .into_iter()
      .map(|(code, name)| Language { code, name })
      .collect();
    let data = LanguagesResponse { languages };
    Response::success(data, None)
  }
  pub fn translate(
    &self,
    text: &str,
    source_lang: &str,
    target_lang: &str,
  ) -> Response<TranslationResponse> {
    let translated_text = match self.translator.translate(text, source_lang, target_lang) {
      Ok(text) => text,
      Err(e) => {
        let data = TranslationResponse {
          translated_text: String::new(),
          source_lang: source_lang.to_string(),
          target_lang: target_lang.to_string(),
        };
        return Response {
          status: tauri_shared::Status::Error,
          message: format!("Translation failed: {}", e),
          data: Some(data),
        };
      }
    };
    let data = TranslationResponse {
      translated_text,
      source_lang: source_lang.to_string(),
      target_lang: target_lang.to_string(),
    };
    Response::success(data, None)
  }
}
