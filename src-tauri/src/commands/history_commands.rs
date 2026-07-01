use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tauri_shared::storage::JsonDb;
use tauri_shared::Response;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationHistoryEntry {
    pub id: String,
    pub text: String,
    pub translated_text: String,
    pub source_lang: String,
    pub target_lang: String,
    pub timestamp: String,
}

#[tauri::command]
pub async fn save_translation(db: State<'_, Arc<RwLock<JsonDb>>>, entry: TranslationHistoryEntry) -> Result<Response<()>, String> {
    let db: &Arc<RwLock<JsonDb>> = db.inner();
    let db = db.read().await;
    let value = serde_json::to_value(&entry).map_err(|e: serde_json::Error| e.to_string())?;
    db.insert("translations", &entry.id, value).map_err(|e: std::io::Error| e.to_string())?;
    Ok(Response::success((), Some("Translation saved")))
}

#[tauri::command]
pub async fn get_translation_history(db: State<'_, Arc<RwLock<JsonDb>>>) -> Result<Response<Vec<TranslationHistoryEntry>>, String> {
    let db: &Arc<RwLock<JsonDb>> = db.inner();
    let db = db.read().await;
    let entries = db.find_all("translations");
    let history: Vec<TranslationHistoryEntry> = entries.into_iter().filter_map(|v| serde_json::from_value(v).ok()).collect();
    Ok(Response::success(history, None))
}
