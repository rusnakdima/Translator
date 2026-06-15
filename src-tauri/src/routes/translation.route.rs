use crate::services::translation_service::TranslationService;
use tauri::State;

pub fn get_supported_languages(
  state: State<'_, TranslationService>,
) -> crate::models::response_model::Response<crate::models::translation_model::LanguagesResponse> {
  state.get_supported_languages()
}
