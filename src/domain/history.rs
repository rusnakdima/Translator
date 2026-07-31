//! Domain layer - History entities

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub translation_id: String,
    pub query: String,
    pub timestamp: DateTime<Utc>,
}

pub trait HistoryService {
    fn add_entry(&mut self, translation_id: String, query: String) -> HistoryEntry;
    fn get_entry(&self, id: &str) -> Option<HistoryEntry>;
    fn get_all_entries(&self) -> Vec<HistoryEntry>;
    fn clear_history(&mut self);
    fn delete_entry(&mut self, id: &str) -> bool;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn history_entry_serde_roundtrip() {
    let entry = HistoryEntry {
      id: "h-456".to_string(),
      translation_id: "tx-789".to_string(),
      query: "good morning".to_string(),
      timestamp: Utc::now(),
    };
    let json = serde_json::to_string(&entry).expect("should serialize");
    let roundtrip: HistoryEntry = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(entry.id, roundtrip.id);
    assert_eq!(entry.translation_id, roundtrip.translation_id);
    assert_eq!(entry.query, roundtrip.query);
  }
}
