use std::sync::Arc;
use tauri::State;
use tauri_shared::storage::JsonDb;
use tauri_shared::Response;
use tokio::sync::RwLock;

#[tauri::command]
pub async fn get_translator_schema(
    db: State<'_, Arc<RwLock<JsonDb>>>,
) -> Result<Response<serde_json::Value>, String> {
    let db: &Arc<RwLock<JsonDb>> = db.inner();
    let db = db.read().await;
    match db.find("schemas", "translator") {
        Some(schema) => Ok(Response::success(schema, None)),
        None => Ok(Response::success(default_translator_schema(), None)),
    }
}

#[tauri::command]
pub async fn save_translator_schema(
    db: State<'_, Arc<RwLock<JsonDb>>>,
    schema: serde_json::Value,
) -> Result<Response<()>, String> {
    let db: &Arc<RwLock<JsonDb>> = db.inner();
    let db = db.read().await;
    db.insert("schemas", "translator", schema)
        .map_err(|e| e.to_string())?;
    Ok(Response::success((), Some("Schema saved")))
}

pub fn default_translator_schema() -> serde_json::Value {
    serde_json::json!({
        "version": "1.0.0",
        "pages": [{
            "id": "translate",
            "route": "",
            "meta": { "title": "Translator" },
            "canvasElements": [
                {
                    "id": "page-bg",
                    "componentId": "div",
                    "classes": "relative min-h-screen p-4 md:p-8 dark:bg-linear-to-br dark:from-neutral-900 dark:via-neutral-800 dark:to-neutral-900 bg-linear-to-br from-indigo-100 via-slate-50 to-indigo-100",
                    "children": ["page-center"]
                },
                {
                    "id": "page-center",
                    "componentId": "div",
                    "classes": "mx-auto max-w-6xl",
                    "children": ["header-row", "main-card", "footer-row", "shortcuts-el"]
                },
                {
                    "id": "header-row",
                    "componentId": "div",
                    "classes": "flex items-center justify-between mb-6",
                    "children": ["header-left", "header-right"]
                },
                {
                    "id": "header-left",
                    "componentId": "div",
                    "classes": "flex flex-col",
                    "children": ["header-title", "header-subtitle"]
                },
                {
                    "id": "header-title",
                    "componentId": "h1",
                    "props": { "text": "Translator" },
                    "classes": "text-3xl font-bold dark:text-white text-slate-900"
                },
                {
                    "id": "header-subtitle",
                    "componentId": "p",
                    "props": { "text": "Translate text between 15 languages instantly" },
                    "classes": "text-sm dark:text-slate-400 text-slate-500 mt-1"
                },
                {
                    "id": "header-right",
                    "componentId": "div",
                    "classes": "flex items-center gap-2",
                    "children": ["shortcuts-btn", "theme-btn"]
                },
                {
                    "id": "shortcuts-btn",
                    "componentId": "app-button",
                    "props": { "buttonStyle": "ghost", "variant": "primary", "size": "sm", "icon": "keyboard" },
                    "events": { "click": [{ "handler": "onShortcutsOpen" }] }
                },
                {
                    "id": "theme-btn",
                    "componentId": "app-theme-toggle",
                    "events": { "toggle": [{ "handler": "onThemeToggle" }] }
                },
                {
                    "id": "main-card",
                    "componentId": "div",
                    "classes": "relative rounded-2xl p-6 md:p-8 shadow-2xl backdrop-blur-lg dark:bg-neutral-800/80 dark:border-neutral-700 bg-white/80 border border-slate-300",
                    "children": ["content-grid", "translate-row"]
                },
                {
                    "id": "content-grid",
                    "componentId": "div",
                    "classes": "grid gap-2 md:grid-cols-[1fr_auto_1fr]",
                    "children": ["source-panel", "swap-area", "target-panel"]
                },
                {
                    "id": "source-panel",
                    "componentId": "div",
                    "classes": "relative flex flex-col",
                    "children": ["source-label-row", "source-input"]
                },
                {
                    "id": "source-label-row",
                    "componentId": "div",
                    "classes": "mb-3 flex items-center justify-between text-sm font-medium dark:text-slate-300 text-slate-700",
                    "children": ["source-label", "source-lang"]
                },
                {
                    "id": "source-label",
                    "componentId": "span",
                    "props": { "text": "From" }
                },
                {
                    "id": "source-lang",
                    "componentId": "app-language-selector",
                    "props": { "labelId": "sourceLang" },
                    "events": { "change": [{ "handler": "onSourceLangChange" }] }
                },
                {
                    "id": "source-input",
                    "componentId": "app-text-input",
                    "props": { "id": "inputText", "placeholder": "Enter text to translate...", "clearable": true, "maxChars": 5000 },
                    "events": {
                        "input": [{ "handler": "onInputTextChange" }],
                        "clear": [{ "handler": "onClearInput" }]
                    }
                },
                {
                    "id": "swap-area",
                    "componentId": "div",
                    "classes": "flex items-center justify-center",
                    "children": ["swap-btn"]
                },
                {
                    "id": "swap-btn",
                    "componentId": "app-swap-button",
                    "events": { "click": [{ "handler": "onSwapLanguages" }] }
                },
                {
                    "id": "target-panel",
                    "componentId": "div",
                    "classes": "relative flex flex-col",
                    "children": ["target-label-row", "target-output"]
                },
                {
                    "id": "target-label-row",
                    "componentId": "div",
                    "classes": "mb-3 flex items-center justify-between text-sm font-medium dark:text-slate-300 text-slate-700",
                    "children": ["target-label", "target-lang-wrap"]
                },
                {
                    "id": "target-label",
                    "componentId": "span",
                    "props": { "text": "To" }
                },
                {
                    "id": "target-lang-wrap",
                    "componentId": "div",
                    "classes": "flex items-center gap-2",
                    "children": ["target-lang"]
                },
                {
                    "id": "target-lang",
                    "componentId": "app-language-selector",
                    "props": { "labelId": "targetLang" },
                    "events": { "change": [{ "handler": "onTargetLangChange" }] }
                },
                {
                    "id": "target-output",
                    "componentId": "app-translation-output",
                    "props": { "id": "outputText", "placeholder": "Translation will appear here as you type..." },
                    "events": { "copy": [{ "handler": "onCopyTranslation" }] }
                },
                {
                    "id": "translate-row",
                    "componentId": "div",
                    "classes": "mt-4 flex justify-center",
                    "children": ["translate-btn"]
                },
                {
                    "id": "translate-btn",
                    "componentId": "app-button",
                    "props": { "buttonStyle": "solid", "variant": "primary", "size": "md" },
                    "events": { "click": [{ "handler": "onTranslate" }] }
                },
                {
                    "id": "footer-row",
                    "componentId": "footer",
                    "classes": "mt-8 text-center text-sm dark:text-slate-400 text-slate-500",
                    "children": ["footer-text"]
                },
                {
                    "id": "footer-text",
                    "componentId": "p",
                    "props": { "text": "Translator - Translate text between 15 languages instantly" }
                },
                {
                    "id": "shortcuts-el",
                    "componentId": "app-shortcuts-overlay"
                }
            ]
        }],
        "layouts": []
    })
}
