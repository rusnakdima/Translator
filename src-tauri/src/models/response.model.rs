use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Status {
  Success,
  Error,
}

#[derive(Serialize, Clone)]
pub struct Response<T = Value> {
  pub status: Status,
  pub message: String,
  pub data: T,
}

impl<T> Response<T> {
  pub fn new(status: Status, message: String, data: T) -> Self {
    Response {
      status,
      message,
      data,
    }
  }

  pub fn success(message: String, data: T) -> Self {
    Response::new(Status::Success, message, data)
  }

  pub fn error_with_data(message: String, data: T) -> Self {
    Response::new(Status::Error, message, data)
  }
}
