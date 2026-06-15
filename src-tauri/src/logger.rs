use log::{LevelFilter, Log, Metadata, Record};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static LOG_FILE: OnceLock<Mutex<Option<File>>> = OnceLock::new();
static LOG_LEVEL: OnceLock<LevelFilter> = OnceLock::new();

pub struct AppLogger;

impl AppLogger {
  pub fn new() -> Self {
    let _ = LOG_LEVEL.set(LevelFilter::Info);
    Self
  }
}

pub fn init_logger() {
  let log_dir = dirs::data_local_dir()
    .unwrap_or_else(|| PathBuf::from("."))
    .join("translator")
    .join("logs");

  std::fs::create_dir_all(&log_dir).ok();

  let log_file = log_dir.join(format!(
    "translator_{}.log",
    chrono::Local::now().format("%Y%m%d")
  ));

  let file = OpenOptions::new()
    .create(true)
    .append(true)
    .open(&log_file)
    .ok();

  let _ = LOG_FILE.set(Mutex::new(file));

  let logger = Box::leak(Box::new(AppLogger::new()));
  let _ = log::set_logger(logger);
  let _ = log::set_max_level(LevelFilter::Info);
}

impl Log for AppLogger {
  fn log(&self, record: &Record) {
    let level = LOG_LEVEL.get().copied().unwrap_or(LevelFilter::Info);
    if record.level() > level {
      return;
    }
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let msg = format!(
      "[{}] {} - {}: {}\n",
      timestamp,
      record.level(),
      record.target(),
      record.args()
    );
    eprint!("{}", msg);
    if let Some(mutex) = LOG_FILE.get() {
      if let Ok(mut guard) = mutex.lock() {
        if let Some(ref mut f) = *guard {
          let _ = f.write_all(msg.as_bytes());
          let _ = f.flush();
        }
      }
    }
  }

  fn enabled(&self, metadata: &Metadata) -> bool {
    let level = LOG_LEVEL.get().copied().unwrap_or(LevelFilter::Info);
    metadata.level() <= level
  }

  fn flush(&self) {}
}
