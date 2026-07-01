#[cfg(mobile)]
use tauri::mobile_entry_point;
mod commands;
mod helpers;
mod models;
mod services;
use models::translation_model::LanguagesResponse;
use services::translation_service::TranslationService;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tauri::{Emitter, Manager, State, Window};
use tauri_shared::{log_error, log_info, Response};
use tauri_shared::storage::JsonDb;
use tokio::sync::RwLock;
static REQUEST_ID: AtomicUsize = AtomicUsize::new(0);
const TAURI_EVENT_TRANSLATION_RESULT: &str = "translation-result";
#[tauri::command]
fn get_supported_languages(state: State<'_, TranslationService>) -> Response<LanguagesResponse> {
  log_info!("Returning supported languages");
  state.inner().get_supported_languages()
}
#[tauri::command]
async fn translate_text(
  text: String,
  source_lang: String,
  target_lang: String,
  state: State<'_, TranslationService>,
  window: Window,
) -> Result<usize, String> {
  log_info!("Translation: {} -> {}", source_lang, target_lang);
  let request_id = REQUEST_ID.fetch_add(1, Ordering::SeqCst);
  let service = state.inner().clone();
  let text_clone = text.clone();
  let source_lang_clone = source_lang.clone();
  let target_lang_clone = target_lang.clone();
  let window_clone = window.clone();
  tauri::async_runtime::spawn(async move {
    let response = service.translate(&text_clone, &source_lang_clone, &target_lang_clone);
    if response.status == tauri_shared::Status::Error {
      log_error!("Translation failed: {}", response.message);
    }
    let response_value = serde_json::to_value(&response).unwrap_or(serde_json::json!({}));
    let payload = serde_json::json!({
      "requestId": request_id,
      "text": text_clone,
      "sourceLang": source_lang_clone,
      "targetLang": target_lang_clone,
      "response": response_value
    });
    let _ = window_clone.emit(TAURI_EVENT_TRANSLATION_RESULT, payload);
  });
  Ok(request_id)
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_mcp_bridge::init())
    .invoke_handler(tauri::generate_handler![
      get_supported_languages,
      translate_text,
      tauri_shared::commands::logger_commands::get_log_entries,
      tauri_shared::commands::logger_commands::set_log_level,
      tauri_shared::commands::logger_commands::clear_logs,
      commands::settings_commands::get_settings,
      commands::settings_commands::save_settings,
      commands::history_commands::save_translation,
      commands::history_commands::get_translation_history,
      commands::schema_commands::get_translator_schema,
      commands::schema_commands::save_translator_schema,
    ])
    .manage(TranslationService::default())
    .setup(|app| {
      let data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
      let _ = std::fs::create_dir_all(&data_dir);
      let db = JsonDb::new(data_dir.join("translator-db.json")).expect("Failed to create database");
      if db.find("schemas", "translator").is_none() {
        let _ = db.insert("schemas", "translator", commands::schema_commands::default_translator_schema());
      }
      app.manage(Arc::new(RwLock::new(db)));
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
