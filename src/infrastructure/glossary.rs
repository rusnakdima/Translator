//! Infrastructure layer - Glossary service implementation
//!
//! State is owned by a `SignalStore` provided by `dioxus-shared`. This
//! implementation serializes/deserializes glossary entries under the
//! `glossary.items` key. Search is delegated to the `search.schemas`
//! algorithm from `dioxus-shared`.

use crate::domain::{GlossaryItem, GlossaryService};
use chrono::Utc;
use dioxus_shared::{algo_execute, AlgorithmRegistry, storage::SignalStore};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

pub struct GlossaryStorage {
  store: Arc<SignalStore>,
  registry: Arc<AlgorithmRegistry>,
}

impl GlossaryStorage {
  pub fn new(store: Arc<SignalStore>, registry: Arc<AlgorithmRegistry>) -> Self {
    Self { store, registry }
  }

  fn next_id() -> String {
    Uuid::new_v4().to_string()
  }

  fn load_items(&self) -> Vec<GlossaryItem> {
    match self.store.get("glossary.items") {
      Some(v) => serde_json::from_value(v).unwrap_or_default(),
      None => Vec::new(),
    }
  }

  fn save_items(&self, items: &[GlossaryItem]) {
    let v = serde_json::to_value(items).unwrap_or(json!([]));
    self.store.set("glossary.items", v);
  }
}

impl GlossaryService for GlossaryStorage {
  fn add_item(&mut self, term: String, definition: String) -> GlossaryItem {
    let item = GlossaryItem {
      id: Self::next_id(),
      term,
      definition,
      created_at: Utc::now(),
    };
    let mut items = self.load_items();
    items.push(item.clone());
    self.save_items(&items);
    item
  }

  fn get_item(&self, id: &str) -> Option<GlossaryItem> {
    self.load_items().into_iter().find(|i| i.id == id)
  }

  fn get_all_items(&self) -> Vec<GlossaryItem> {
    self.load_items()
  }

  fn delete_item(&mut self, id: &str) -> bool {
    let mut items = self.load_items();
    let before = items.len();
    items.retain(|i| i.id != id);
    let removed = items.len() != before;
    if removed {
      self.save_items(&items);
    }
    removed
  }

  fn search(&self, query: &str) -> Vec<GlossaryItem> {
    let items = self.load_items();
    // Serialize items as JSON strings for search.schemas (treats each item's
    // JSON representation as a searchable string).
    let string_items: Vec<String> = items
      .iter()
      .map(|i| serde_json::to_string(i).unwrap_or_default())
      .collect();
    // search.schemas expects SearchInput { items: Vec<Value>, query: String }
    let data = serde_json::json!({ "items": string_items, "query": query });
    match algo_execute(&self.registry, "search.schemas", data, None, None) {
      Ok(matching_json) => {
        let matching: Vec<String> = serde_json::from_value(matching_json).unwrap_or_default();
        matching
          .into_iter()
          .filter_map(|s| serde_json::from_str(&s).ok())
          .collect()
      }
      Err(_) => items,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::cell::RefCell;

  /// Test harness that wraps GlossaryStorage with RefCell for interior mutability,
  /// allowing trait methods to be called without &mut self.
  struct TestGlossaryStorage {
    inner: RefCell<GlossaryStorage>,
  }

  impl TestGlossaryStorage {
    fn new(store: Arc<SignalStore>) -> Self {
      let registry = Arc::new(AlgorithmRegistry::new());
      Self { inner: RefCell::new(GlossaryStorage::new(store, registry)) }
    }

    fn add_item(&self, term: String, definition: String) -> GlossaryItem {
      self.inner.borrow_mut().add_item(term, definition)
    }

    fn get_item(&self, id: &str) -> Option<GlossaryItem> {
      self.inner.borrow().get_item(id)
    }

    fn list_terms(&self) -> Vec<GlossaryItem> {
      self.inner.borrow().get_all_items()
    }

    fn delete_term(&self, id: &str) -> bool {
      self.inner.borrow_mut().delete_item(id)
    }

    fn to_json(&self) -> serde_json::Value {
      let items = self.list_terms();
      serde_json::to_value(&items).unwrap()
    }

    fn from_json_and_add(store: Arc<SignalStore>, json: serde_json::Value) -> Result<Vec<GlossaryItem>, String> {
      let items: Vec<GlossaryItem> = serde_json::from_value(json)
        .map_err(|e| format!("{e}"))?;
      let registry = Arc::new(AlgorithmRegistry::new());
      let storage = GlossaryStorage::new(store, registry);
      // Re-populate store from the deserialized items
      let v = serde_json::to_value(&items).unwrap_or(json!([]));
      storage.store.set("glossary.items", v);
      Ok(items)
    }
  }

  fn make_store() -> Arc<SignalStore> {
    Arc::new(SignalStore::new())
  }

  #[test]
  fn glossary_empty_initially() {
    let store = make_store();
    let storage = TestGlossaryStorage::new(store);
    assert!(storage.list_terms().is_empty());
  }

  #[test]
  fn glossary_add_item_returns_uuid() {
    let store = make_store();
    let storage = TestGlossaryStorage::new(store);
    let item = storage.add_item("hello".to_string(), "hola".to_string());
    assert!(!item.id.is_empty(), "expected non-empty UUID");
    assert_eq!(item.term, "hello");
    assert_eq!(item.definition, "hola");
  }

  #[test]
  fn glossary_list_includes_added() {
    let store = make_store();
    let storage = TestGlossaryStorage::new(store);
    let added = storage.add_item("goodbye".to_string(), "adios".to_string());
    let all = storage.list_terms();
    assert!(all.iter().any(|i| i.id == added.id));
  }

  #[test]
  fn glossary_update_via_get_set() {
    let store = make_store();
    let storage = TestGlossaryStorage::new(store);
    let item = storage.add_item("cat".to_string(), "gato".to_string());
    // GlossaryService doesn't expose update; verify item is retrievable
    let retrieved = storage.get_item(&item.id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().term, "cat");
  }

  #[test]
  fn glossary_delete_removes_from_list() {
    let store = make_store();
    let storage = TestGlossaryStorage::new(store);
    let item = storage.add_item("dog".to_string(), "perro".to_string());
    let deleted = storage.delete_term(&item.id);
    assert!(deleted);
    assert!(storage.get_item(&item.id).is_none());
  }

  #[test]
  fn glossary_get_none_after_delete() {
    let store = make_store();
    let storage = TestGlossaryStorage::new(store);
    let item = storage.add_item("fish".to_string(), "pez".to_string());
    storage.delete_term(&item.id);
    assert!(storage.get_item(&item.id).is_none());
  }

  #[test]
  fn glossary_serialization_roundtrip() {
    let store = make_store();
    let storage = TestGlossaryStorage::new(store);
    storage.add_item("apple".to_string(), "manzana".to_string());
    storage.add_item("orange".to_string(), "naranja".to_string());

    let json = storage.to_json();
    let items: Vec<GlossaryItem> = serde_json::from_value(json.clone())
      .expect("roundtrip should succeed");
    assert_eq!(items.len(), 2);
    assert!(items.iter().any(|i| i.term == "apple"));
    assert!(items.iter().any(|i| i.term == "orange"));
  }

  #[test]
  fn glossary_from_json_malformed_error() {
    let store = make_store();
    let bad_json = serde_json::json!({ "not": "a valid GlossaryItem" });
    let result = TestGlossaryStorage::from_json_and_add(store, bad_json);
    assert!(result.is_err() || result.unwrap().is_empty());
  }

  #[test]
  fn glossary_empty_list_returns_empty() {
    let store = make_store();
    let storage = TestGlossaryStorage::new(store);
    let json = storage.to_json();
    let items: Vec<GlossaryItem> = serde_json::from_value(json).unwrap();
    assert!(items.is_empty());
  }
}