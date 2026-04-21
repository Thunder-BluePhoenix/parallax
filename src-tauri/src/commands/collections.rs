use std::path::PathBuf;
use std::fs;
use serde::{Deserialize, Serialize};
use crate::http_engine::ParallaxRequest;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collection {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub requests: Vec<ParallaxRequest>,
    pub folders: Vec<CollectionFolder>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CollectionFolder {
    pub name: String,
    pub requests: Vec<ParallaxRequest>,
}

fn parallax_dir(workspace: &str) -> PathBuf {
    PathBuf::from(workspace).join(".parallax")
}

#[tauri::command]
pub fn list_collections(workspace: String) -> Result<Vec<String>, String> {
    let collections_dir = parallax_dir(&workspace).join("collections");
    if !collections_dir.exists() {
        fs::create_dir_all(&collections_dir).map_err(|e| e.to_string())?;
        return Ok(vec![]);
    }

    let names: Vec<String> = fs::read_dir(&collections_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().and_then(|ext| ext.to_str())
                == Some("yaml")
        })
        .filter_map(|e| {
            e.path().file_stem()
                .and_then(|s| s.to_str())
                .map(String::from)
        })
        .collect();

    Ok(names)
}

#[tauri::command]
pub fn load_collection(workspace: String, name: String) -> Result<Collection, String> {
    let path = parallax_dir(&workspace).join("collections").join(format!("{}.yaml", name));
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_yaml::from_str(&content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_collection(workspace: String, collection: Collection) -> Result<(), String> {
    let dir = parallax_dir(&workspace).join("collections");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let path = dir.join(format!("{}.yaml", collection.name.to_lowercase().replace(' ', "-")));
    let yaml = serde_yaml::to_string(&collection).map_err(|e| e.to_string())?;
    fs::write(&path, yaml).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_collection(workspace: String, name: String) -> Result<(), String> {
    let path = parallax_dir(&workspace).join("collections").join(format!("{}.yaml", name));
    fs::remove_file(&path).map_err(|e| e.to_string())
}
