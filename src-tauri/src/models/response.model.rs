use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
  Success,
  Info,
  Warning,
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

  pub fn info(message: String, data: T) -> Self {
    Response::new(Status::Info, message, data)
  }

  pub fn warning(message: String, data: T) -> Self {
    Response::new(Status::Warning, message, data)
  }

  pub fn error_with_data(message: String, data: T) -> Self {
    Response::new(Status::Error, message, data)
  }
}

impl Response<Value> {
  pub fn error(message: String) -> Self {
    Response::new(Status::Error, message, Value::Null)
  }
}
