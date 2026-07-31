//! Domain layer - Glossary entities

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryItem {
    pub id: String,
    pub term: String,
    pub definition: String,
    pub created_at: DateTime<Utc>,
}

pub trait GlossaryService {
    fn add_item(&mut self, term: String, definition: String) -> GlossaryItem;
    fn get_item(&self, id: &str) -> Option<GlossaryItem>;
    fn get_all_items(&self) -> Vec<GlossaryItem>;
    fn delete_item(&mut self, id: &str) -> bool;
    fn search(&self, query: &str) -> Vec<GlossaryItem>;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn glossary_item_serde_roundtrip() {
    let item = GlossaryItem {
      id: "g-123".to_string(),
      term: "keyboard".to_string(),
      definition: "teclado".to_string(),
      created_at: Utc::now(),
    };
    let json = serde_json::to_string(&item).expect("should serialize");
    let roundtrip: GlossaryItem = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(item.id, roundtrip.id);
    assert_eq!(item.term, roundtrip.term);
    assert_eq!(item.definition, roundtrip.definition);
  }
}
