use std::io::Write;
use std::process::{Command, Stdio};

pub struct Translator;

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

impl Translator {
  pub fn new() -> Self {
    Translator
  }

  pub fn get_supported_languages() -> Vec<(String, String)> {
    SUPPORTED_LANGUAGES
      .iter()
      .map(|(code, name)| (code.to_string(), name.to_string()))
      .collect()
  }

  pub fn translate(
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

    let supported_codes: Vec<&str> = SUPPORTED_LANGUAGES.iter().map(|&(code, _)| code).collect();

    if !supported_codes.contains(&source_lang) {
      return Err(format!("Unsupported source language: {}", source_lang));
    }

    if !supported_codes.contains(&target_lang) {
      return Err(format!("Unsupported target language: {}", target_lang));
    }

    let mut child = Command::new("trans")
      .args(&["-b", "-s", source_lang, "-t", target_lang])
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .map_err(|e| format!("Failed to execute translation command: {}", e))?;

    let stdin = child.stdin.as_mut().ok_or("Failed to get stdin")?;
    stdin
      .write_all(text.as_bytes())
      .map_err(|e| format!("Failed to write to stdin: {}", e))?;
    drop(stdin);

    let output = child
      .wait_with_output()
      .map_err(|e| format!("Failed to read output: {}", e))?;

    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      return Err(format!("Translation failed: {}", stderr));
    }

    let translated = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if translated.is_empty() {
      return Err("Empty translation result".to_string());
    }

    Ok(translated)
  }
}

impl Default for Translator {
  fn default() -> Self {
    Self::new()
  }
}
