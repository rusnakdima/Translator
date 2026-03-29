use crate::models::response_model::Response;
use crate::models::translation_model::LanguagesResponse;
use crate::services::translation_service::TranslationService;
use tauri::State;

pub struct TranslationRoute;

impl TranslationRoute {
  pub fn get_supported_languages(
    state: State<'_, TranslationService>,
  ) -> Response<LanguagesResponse> {
    state.get_supported_languages()
  }
}
