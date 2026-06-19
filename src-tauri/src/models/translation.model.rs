use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationResponse {
  pub translated_text: String,
  pub source_lang: String,
  pub target_lang: String,
}
impl TranslationResponse {
  pub fn new(translated_text: String, source_lang: String, target_lang: String) -> Self {
    Self {
      translated_text,
      source_lang,
      target_lang,
    }
  }
  pub fn empty(source_lang: String, target_lang: String) -> Self {
    Self {
      translated_text: String::new(),
      source_lang,
      target_lang,
    }
  }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Language {
  pub code: String,
  pub name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguagesResponse {
  pub languages: Vec<Language>,
}
