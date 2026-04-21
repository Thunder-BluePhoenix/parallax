use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Environment {
    pub name: String,
    pub variables: HashMap<String, String>,
}

fn env_dir(workspace: &str) -> PathBuf {
    PathBuf::from(workspace).join(".parallax").join("environments")
}

#[tauri::command]
pub fn list_environments(workspace: String) -> Result<Vec<String>, String> {
    let dir = env_dir(&workspace);
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        return Ok(vec![]);
    }
    let names: Vec<String> = fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
        .filter_map(|e| e.path().file_stem().and_then(|s| s.to_str()).map(String::from))
        .collect();
    Ok(names)
}

#[tauri::command]
pub fn load_environment(workspace: String, name: String) -> Result<Environment, String> {
    let path = env_dir(&workspace).join(format!("{}.json", name));
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_environment(workspace: String, env: Environment) -> Result<(), String> {
    let dir = env_dir(&workspace);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.json", env.name));
    let json = serde_json::to_string_pretty(&env).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}
