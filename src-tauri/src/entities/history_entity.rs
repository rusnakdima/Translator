use nosql_orm::prelude::{Entity, EntityMeta, Validate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TranslationHistoryEntry {
  pub id: String,
  pub text: String,
  pub translated_text: String,
  pub source_lang: String,
  pub target_lang: String,
  pub timestamp: String,
}

impl Entity for TranslationHistoryEntry {
  fn meta() -> EntityMeta {
    EntityMeta::new("translation_history")
  }

  fn get_id(&self) -> Option<String> {
    Some(self.id.clone())
  }

  fn set_id(&mut self, id: String) {
    self.id = id;
  }
}
