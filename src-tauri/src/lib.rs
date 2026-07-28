#[cfg(mobile)]
use tauri::mobile_entry_point;
mod commands;
mod entities;
mod helpers;
mod models;
mod services;
use models::translation_model::LanguagesResponse;
use services::translation_service::TranslationService;
use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::{Emitter, Manager, State, Window};
use tauri_shared::{log_error, log_info, Response};
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
    let response = service
      .translate_async(&text_clone, &source_lang_clone, &target_lang_clone)
      .await;
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
    .plugin(
      tauri_plugin_log::Builder::new()
        .format(|out, message, record| {
          out.finish(format_args!(
            "[{}] [{}] [{}] [{}] {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            "translator",
            record.target(),
            message
          ))
        })
        .build(),
    )
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_mcp_bridge::init())
    .invoke_handler(tauri::generate_handler![
      get_supported_languages,
      translate_text,
      tauri_shared::commands::logger_commands::get_log_entries,
      tauri_shared::commands::logger_commands::set_log_level,
      tauri_shared::commands::logger_commands::clear_logs,
      commands::get_schema,
      commands::save_schema,
      commands::get_all_schemas,
      commands::delete_schema,
      // Algorithm commands from tauri-shared (registry pattern)
      tauri_shared::commands::algorithm_commands::algo_execute,
      tauri_shared::commands::algorithm_commands::list_algorithms,
      // Unified CRUD commands (replaces get_settings, save_settings, get_translation_history, save_translation)
      tauri_shared::commands::crud_commands::crud_execute,
    ])
    .manage(TranslationService::default())
    .setup(|app| {
      let data_dir = app
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");
      // Set NOSQL_ORM_DATA_DIR so nosql_orm uses this as base path
      // Then we just pass "translator_db" as the relative path
      std::env::set_var("NOSQL_ORM_DATA_DIR", data_dir.to_string_lossy().as_ref());
      let db_path = "translator_db";
      let provider =
        tauri::async_runtime::block_on(nosql_orm::providers::JsonProvider::new(db_path))
          .expect("Failed to create JSON provider");

      app.manage(provider);
      log_info!(
        "Initialized nosql_orm JsonProvider at: {}/{}",
        data_dir.display(),
        db_path
      );
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
