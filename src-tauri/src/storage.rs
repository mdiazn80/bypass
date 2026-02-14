use std::fs;
use std::path::PathBuf;

use crate::models::{AppConfig, Context};

fn data_dir() -> PathBuf {
    let dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.mdiazn80.bypass");
    fs::create_dir_all(&dir).ok();
    dir
}

fn contexts_path() -> PathBuf {
    data_dir().join("contexts.json")
}

fn config_path() -> PathBuf {
    data_dir().join("config.json")
}

pub fn load_contexts() -> Vec<Context> {
    let path = contexts_path();
    if path.exists() {
        let data = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    }
}

pub fn save_contexts(contexts: &[Context]) -> Result<(), String> {
    let path = contexts_path();
    let data = serde_json::to_string_pretty(contexts).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| e.to_string())
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    if path.exists() {
        let data = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_else(|_| AppConfig::default())
    } else {
        AppConfig::default()
    }
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = config_path();
    let data = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| e.to_string())
}
