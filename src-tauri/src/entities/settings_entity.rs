use nosql_orm::prelude::{Entity, EntityMeta, Validate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UserSettings {
  pub id: Option<String>,
  pub source_lang: String,
  pub target_lang: String,
}

impl Entity for UserSettings {
  fn meta() -> EntityMeta {
    EntityMeta::new("user_settings")
  }

  fn get_id(&self) -> Option<String> {
    self.id.clone()
  }

  fn set_id(&mut self, id: String) {
    self.id = Some(id);
  }
}
