#[cfg(mobile)]
use tauri::mobile_entry_point;
mod commands;
mod entities;
mod helpers;
mod models;
mod services;
use entities::{TranslationHistoryEntry, UserSettings};
use models::translation_model::LanguagesResponse;
use nosql_orm::prelude::Repository;
use services::translation_service::TranslationService;
use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::{Emitter, Manager, State, Window};
use tauri_shared::{log_error, log_info, AppError, Response};
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
    let response = service.translate_async(&text_clone, &source_lang_clone, &target_lang_clone).await;
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
#[tauri::command]
async fn get_settings(
  db: State<'_, nosql_orm::providers::JsonProvider>,
) -> tauri_shared::Result<Response<UserSettings>> {
  let repo: Repository<UserSettings, _> = Repository::new(db.inner().clone());
  match repo.find_by_id("user_preferences").await {
    Ok(Some(settings)) => Ok(Response::success(settings, None)),
    Ok(None) => Ok(Response::success(
      UserSettings {
        id: Some("user_preferences".to_string()),
        source_lang: "en".to_string(),
        target_lang: "es".to_string(),
      },
      None,
    )),
    Err(e) => Err(AppError::from(e)),
  }
}
#[tauri::command]
async fn save_settings(
  db: State<'_, nosql_orm::providers::JsonProvider>,
  settings: UserSettings,
) -> tauri_shared::Result<Response<()>> {
  let repo: Repository<UserSettings, _> = Repository::new(db.inner().clone());
  repo.save(settings).await.map_err(AppError::from)?;
  Ok(Response::success((), Some("Settings saved")))
}
#[tauri::command]
async fn save_translation(
  db: State<'_, nosql_orm::providers::JsonProvider>,
  entry: TranslationHistoryEntry,
) -> tauri_shared::Result<Response<()>> {
  let repo: Repository<TranslationHistoryEntry, _> = Repository::new(db.inner().clone());
  repo.save(entry).await.map_err(AppError::from)?;
  Ok(Response::success((), Some("Translation saved")))
}
#[tauri::command]
async fn get_translation_history(
  db: State<'_, nosql_orm::providers::JsonProvider>,
) -> tauri_shared::Result<Response<Vec<TranslationHistoryEntry>>> {
  let repo: Repository<TranslationHistoryEntry, _> = Repository::new(db.inner().clone());
  repo
    .find_all()
    .await
    .map(|entries| Response::success(entries, None))
    .map_err(AppError::from)
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_mcp_bridge::init())
    .invoke_handler(tauri::generate_handler![
      get_supported_languages,
      translate_text,
      get_settings,
      save_settings,
      save_translation,
      get_translation_history,
      tauri_shared::commands::logger_commands::get_log_entries,
      tauri_shared::commands::logger_commands::set_log_level,
      tauri_shared::commands::logger_commands::clear_logs,
      commands::get_schema,
      commands::save_schema,
      commands::get_all_schemas,
      commands::delete_schema,
      // Algorithm commands from tauri-shared
      tauri_shared::commands::algorithm_commands::quick_sort,
      tauri_shared::commands::algorithm_commands::merge_sort,
      tauri_shared::commands::algorithm_commands::bubble_sort,
      tauri_shared::commands::algorithm_commands::insertion_sort,
      tauri_shared::commands::algorithm_commands::dijkstra,
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
      let provider = tauri::async_runtime::block_on(nosql_orm::providers::JsonProvider::new(db_path))
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
