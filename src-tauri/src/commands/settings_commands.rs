use std::sync::Arc;
use tauri::State;
use tauri_shared::storage::JsonDb;
use tauri_shared::Response;
use tokio::sync::RwLock;

#[tauri::command]
pub async fn get_settings(db: State<'_, Arc<RwLock<JsonDb>>>) -> Result<Response<serde_json::Value>, String> {
    let db: &Arc<RwLock<JsonDb>> = db.inner();
    let db = db.read().await;
    let result = db.find("settings", "user_preferences");
    match result {
        Some(data) => Ok(Response::success(data, None)),
        None => Ok(Response::success(serde_json::json!({"source_lang": "en", "target_lang": "es"}), None)),
    }
}

#[tauri::command]
pub async fn save_settings(db: State<'_, Arc<RwLock<JsonDb>>>, settings: serde_json::Value) -> Result<Response<()>, String> {
    let db: &Arc<RwLock<JsonDb>> = db.inner();
    let db = db.read().await;
    db.insert("settings", "user_preferences", settings).map_err(|e: std::io::Error| e.to_string())?;
    Ok(Response::success((), Some("Settings saved")))
}
