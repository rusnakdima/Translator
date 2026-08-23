use std::{fs, path::Path};

fn main() {
    let schema_path = Path::new("schemas/translator_v3.json");
    let content = fs::read_to_string(schema_path).expect("Schema file must exist");
    let _: serde_json::Value = serde_json::from_str(&content).expect("Schema must be valid JSON");
    println!("cargo:warning=schema valid: {}", schema_path.display());
}
