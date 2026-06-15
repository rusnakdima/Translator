use crate::helpers::translator_helper::Translator;
use crate::models::response_model::Response;
use crate::models::translation_model::{Language, LanguagesResponse, TranslationResponse};

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
    Response::success("Languages retrieved successfully".to_string(), data)
  }

  pub fn translate(
    &self,
    text: &str,
    source_lang: &str,
    target_lang: &str,
  ) -> Response<TranslationResponse> {
    let source_lang = source_lang.to_string();
    let target_lang = target_lang.to_string();

    let translated_text = match self.translator.translate(text, &source_lang, &target_lang) {
      Ok(text) => text,
      Err(e) => {
        return Response::error_with_data(
          format!("Translation failed: {}", e),
          TranslationResponse::empty(source_lang.clone(), target_lang.clone()),
        );
      }
    };

    Response::success(
      "Translation completed successfully".to_string(),
      TranslationResponse::new(translated_text, source_lang, target_lang),
    )
  }
}
