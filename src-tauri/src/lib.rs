#[cfg(mobile)]
use tauri::mobile_entry_point;

mod helpers;
mod models;
pub mod routes;
mod services;

use helpers::translator_helper::Translator;
use models::response_model::Response;
use models::translation_model::{LanguagesResponse, TranslationResponse};
use routes::translation_route::TranslationRoute;
use services::translation_service::TranslationService;
use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::{Emitter, State, Window};

static REQUEST_ID: AtomicUsize = AtomicUsize::new(0);

#[tauri::command]
fn get_supported_languages(state: State<'_, TranslationService>) -> Response<LanguagesResponse> {
  TranslationRoute::get_supported_languages(state)
}

#[tauri::command]
async fn translate_text(
  text: String,
  source_lang: String,
  target_lang: String,
  _state: State<'_, TranslationService>,
  window: Window,
) -> Result<usize, String> {
  let request_id = REQUEST_ID.fetch_add(1, Ordering::SeqCst);
  println!(
    "[Rust] Translation request {}: {} -> {}",
    request_id, source_lang, target_lang
  );

  let translator = Translator::new();
  let text_clone = text.clone();
  let source_lang_clone = source_lang.clone();
  let target_lang_clone = target_lang.clone();
  let window_clone = window.clone();

  tauri::async_runtime::spawn(async move {
    let result = translator.translate(&text_clone, &source_lang_clone, &target_lang_clone);

    let response = match result {
      Ok(translated_text) => {
        let data = TranslationResponse {
          translated_text,
          source_lang: source_lang_clone.clone(),
          target_lang: target_lang_clone.clone(),
        };
        Response::success("Translation completed".to_string(), data)
      }
      Err(e) => {
        let data = TranslationResponse {
          translated_text: String::new(),
          source_lang: source_lang_clone.clone(),
          target_lang: target_lang_clone.clone(),
        };
        Response::error_with_data(format!("Translation failed: {}", e), data)
      }
    };

    let response_value = serde_json::to_value(&response).unwrap_or(serde_json::json!({}));

    let payload = serde_json::json!({
      "requestId": request_id,
      "text": text_clone,
      "sourceLang": source_lang_clone,
      "targetLang": target_lang_clone,
      "response": response_value
    });

    let _ = window_clone.emit("translation-result", payload);
  });

  Ok(request_id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![
      get_supported_languages,
      translate_text
    ])
    .manage(TranslationService::new())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
