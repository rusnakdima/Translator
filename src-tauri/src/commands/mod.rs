use nosql_orm::prelude::{Entity, Repository, Validate, WithRelations};
use serde::{Deserialize, Serialize};
use tauri::State;
use tauri_shared::{log_error, log_info, Response};

/// UiSchema stored in database - id is at top level (extracted from app.id)
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UiSchema {
  pub id: String,
  #[serde(alias = "schemaVersion")]
  pub version: String,
  pub app: serde_json::Value,
  pub pages: Vec<serde_json::Value>,
  pub layouts: Vec<serde_json::Value>,
  #[serde(default)]
  pub shortcuts: Vec<serde_json::Value>,
  #[serde(default)]
  pub handlers: serde_json::Value,
  #[serde(default)]
  pub stores: serde_json::Value,
}

impl Entity for UiSchema {
  fn meta() -> nosql_orm::prelude::EntityMeta {
    nosql_orm::prelude::EntityMeta::new("schemas")
  }

  fn get_id(&self) -> Option<String> {
    Some(self.id.clone())
  }

  fn set_id(&mut self, id: String) {
    self.id = id;
  }
}

impl WithRelations for UiSchema {}

#[tauri::command]
pub async fn get_schema(
  db: State<'_, nosql_orm::providers::JsonProvider>,
  id: String,
) -> Result<Response<UiSchema>, String> {
  let repo: Repository<UiSchema, _> = Repository::new(db.inner().clone());
  match repo.find_by_id(&id).await {
    Ok(Some(schema)) => Ok(Response::success(schema, None)),
    Ok(None) => {
      log_error!("Schema not found: {}", id);
      Ok(Response::not_found("Schema"))
    }
    Err(e) => {
      log_error!("Failed to get schema: {}", e);
      Err(e.to_string())
    }
  }
}

#[tauri::command]
pub async fn save_schema(
  db: State<'_, nosql_orm::providers::JsonProvider>,
  schema: UiSchema,
) -> Result<Response<()>, String> {
  let repo: Repository<UiSchema, _> = Repository::new(db.inner().clone());
  repo.save(schema).await.map_err(|e| e.to_string())?;
  log_info!("Schema saved successfully");
  Ok(Response::success((), Some("Schema saved")))
}

#[tauri::command]
pub async fn get_all_schemas(
  db: State<'_, nosql_orm::providers::JsonProvider>,
) -> Result<Response<Vec<UiSchema>>, String> {
  let repo: Repository<UiSchema, _> = Repository::new(db.inner().clone());
  repo
    .find_all()
    .await
    .map(|schemas| Response::success(schemas, None))
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_schema(
  db: State<'_, nosql_orm::providers::JsonProvider>,
  id: String,
) -> Result<Response<()>, String> {
  let repo: Repository<UiSchema, _> = Repository::new(db.inner().clone());
  repo.delete(&id).await.map_err(|e| e.to_string())?;
  log_info!("Schema deleted: {}", id);
  Ok(Response::success((), Some("Schema deleted")))
}
