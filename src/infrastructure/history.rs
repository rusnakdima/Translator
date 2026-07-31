//! Infrastructure layer - History service implementation
//!
//! State is owned by a `SignalStore` provided by `dioxus-shared`. Entries
//! persist under the `history.entries` key.

use crate::domain::{HistoryEntry, HistoryService};
use chrono::Utc;
use dioxus_shared::storage::SignalStore;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

pub struct HistoryStorage {
  store: Arc<SignalStore>,
}

impl HistoryStorage {
  pub fn new(store: Arc<SignalStore>) -> Self {
    Self { store }
  }

  fn next_id() -> String {
    Uuid::new_v4().to_string()
  }

  fn load_entries(&self) -> Vec<HistoryEntry> {
    match self.store.get("history.entries") {
      Some(v) => serde_json::from_value(v).unwrap_or_default(),
      None => Vec::new(),
    }
  }

  fn save_entries(&self, entries: &[HistoryEntry]) {
    let v = serde_json::to_value(entries).unwrap_or(json!([]));
    self.store.set("history.entries", v);
  }
}

impl HistoryService for HistoryStorage {
  fn add_entry(&mut self, translation_id: String, query: String) -> HistoryEntry {
    let entry = HistoryEntry {
      id: Self::next_id(),
      translation_id,
      query,
      timestamp: Utc::now(),
    };
    let mut entries = self.load_entries();
    entries.push(entry.clone());
    self.save_entries(&entries);
    entry
  }

  fn get_entry(&self, id: &str) -> Option<HistoryEntry> {
    self.load_entries().into_iter().find(|e| e.id == id)
  }

  fn get_all_entries(&self) -> Vec<HistoryEntry> {
    self.load_entries()
  }

  fn clear_history(&mut self) {
    self.save_entries(&[]);
  }

  fn delete_entry(&mut self, id: &str) -> bool {
    let mut entries = self.load_entries();
    let before = entries.len();
    entries.retain(|e| e.id != id);
    let removed = entries.len() != before;
    if removed {
      self.save_entries(&entries);
    }
    removed
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::cell::RefCell;

  struct TestHistoryStorage {
    inner: RefCell<HistoryStorage>,
  }

  impl TestHistoryStorage {
    fn new(store: Arc<SignalStore>) -> Self {
      Self { inner: RefCell::new(HistoryStorage::new(store)) }
    }

    fn add_entry(&self, translation_id: String, query: String) -> HistoryEntry {
      self.inner.borrow_mut().add_entry(translation_id, query)
    }

    fn list_history(&self) -> Vec<HistoryEntry> {
      self.inner.borrow().get_all_entries()
    }

    fn get_last_entry(&self) -> Option<HistoryEntry> {
      let entries = self.list_history();
      entries.into_iter().last()
    }

    fn clear_history(&self) {
      self.inner.borrow_mut().clear_history();
    }

    fn to_json(&self) -> serde_json::Value {
      serde_json::to_value(self.list_history()).unwrap()
    }
  }

  fn make_store() -> Arc<SignalStore> {
    Arc::new(SignalStore::new())
  }

  #[test]
  fn history_empty_returns_empty_list() {
    let store = make_store();
    let storage = TestHistoryStorage::new(store);
    assert!(storage.list_history().is_empty());
  }

  #[test]
  fn history_get_last_entry_empty_returns_none() {
    let store = make_store();
    let storage = TestHistoryStorage::new(store);
    assert!(storage.get_last_entry().is_none());
  }

  #[test]
  fn history_add_entry_returns_uuid() {
    let store = make_store();
    let storage = TestHistoryStorage::new(store);
    let entry = storage.add_entry("tx-1".to_string(), "hello world".to_string());
    assert!(!entry.id.is_empty());
    assert_eq!(entry.query, "hello world");
    assert_eq!(entry.translation_id, "tx-1");
  }

  #[test]
  fn history_list_includes_added() {
    let store = make_store();
    let storage = TestHistoryStorage::new(store);
    let added = storage.add_entry("tx-2".to_string(), "good morning".to_string());
    let all = storage.list_history();
    assert!(all.iter().any(|e| e.id == added.id));
  }

  #[test]
  fn history_get_last_entry_returns_most_recent() {
    let store = make_store();
    let storage = TestHistoryStorage::new(store);
    let first = storage.add_entry("tx-a".to_string(), "first".to_string());
    std::thread::sleep(std::time::Duration::from_millis(10));
    let second = storage.add_entry("tx-b".to_string(), "second".to_string());
    assert_eq!(storage.get_last_entry().unwrap().id, second.id);
    assert_ne!(storage.get_last_entry().unwrap().id, first.id);
  }

  #[test]
  fn history_multiple_entries_last_is_newest() {
    let store = make_store();
    let storage = TestHistoryStorage::new(store);
    storage.add_entry("tx-x".to_string(), "one".to_string());
    storage.add_entry("tx-y".to_string(), "two".to_string());
    storage.add_entry("tx-z".to_string(), "three".to_string());
    let last = storage.get_last_entry().expect("expected an entry");
    assert_eq!(last.translation_id, "tx-z");
    assert_eq!(last.query, "three");
  }

  #[test]
  fn history_clear_history() {
    let store = make_store();
    let storage = TestHistoryStorage::new(store);
    storage.add_entry("tx-c".to_string(), "alpha".to_string());
    storage.add_entry("tx-d".to_string(), "beta".to_string());
    storage.clear_history();
    assert!(storage.list_history().is_empty());
    assert!(storage.get_last_entry().is_none());
  }

  #[test]
  fn history_serialization_roundtrip() {
    let store = make_store();
    let storage = TestHistoryStorage::new(store);
    storage.add_entry("tx-1".to_string(), "hello".to_string());
    storage.add_entry("tx-2".to_string(), "world".to_string());

    let json = storage.to_json();
    let entries: Vec<HistoryEntry> = serde_json::from_value(json).expect("should roundtrip");
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().any(|e| e.query == "hello"));
    assert!(entries.iter().any(|e| e.query == "world"));
  }
}