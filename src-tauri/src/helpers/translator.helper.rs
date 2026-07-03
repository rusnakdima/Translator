use once_cell::sync::Lazy;
use tokio::sync::RwLock;
use trad::Translator as TradTranslator;

static TRANSLATOR: Lazy<RwLock<Option<TradTranslator>>> = Lazy::new(|| RwLock::new(None));

const SUPPORTED_LANGUAGES: &[(&str, &str)] = &[
  ("en", "English"),
  ("es", "Spanish"),
  ("fr", "French"),
  ("de", "German"),
  ("it", "Italian"),
  ("pt", "Portuguese"),
  ("ru", "Russian"),
  ("ja", "Japanese"),
  ("ko", "Korean"),
  ("zh", "Chinese"),
  ("ar", "Arabic"),
  ("hi", "Hindi"),
  ("nl", "Dutch"),
  ("pl", "Polish"),
  ("tr", "Turkish"),
];

fn validate_language_code(code: &str, label: &str) -> Result<(), String> {
  let supported_codes: Vec<&str> = SUPPORTED_LANGUAGES.iter().map(|&(c, _)| c).collect();
  if !supported_codes.contains(&code) {
    return Err(format!("Unsupported {} language: {}", label, code));
  }
  Ok(())
}

#[derive(Clone, Default)]
pub struct Translator;

impl Translator {
  pub fn get_supported_languages() -> Vec<(String, String)> {
    SUPPORTED_LANGUAGES
      .iter()
      .map(|(code, name)| (code.to_string(), name.to_string()))
      .collect()
  }

  pub async fn ensure_initialized() -> Result<(), String> {
    let mut guard = TRANSLATOR.write().await;
    if guard.is_none() {
      let translator = TradTranslator::setup(None)
        .await
        .map_err(|e| format!("Failed to initialize translator: {}", e))?;
      *guard = Some(translator);
    }
    Ok(())
  }

  pub async fn translate(
    &self,
    text: &str,
    source_lang: &str,
    target_lang: &str,
  ) -> Result<String, String> {
    if text.trim().is_empty() {
      return Err("Empty text provided".to_string());
    }
    if source_lang == target_lang {
      return Ok(text.to_string());
    }
    validate_language_code(source_lang, "source")?;
    validate_language_code(target_lang, "target")?;

    // Ensure translator is initialized
    Self::ensure_initialized().await?;

    let guard = TRANSLATOR.read().await;
    let translator = guard.as_ref().ok_or("Translator not initialized")?;

    let translated = translator
      .translate(text, source_lang, target_lang)
      .await
      .map_err(|e| format!("Translation failed: {}", e))?;

    if translated.is_empty() {
      return Err("Empty translation result".to_string());
    }
    Ok(translated)
  }
}
