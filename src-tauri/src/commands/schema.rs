use crate::schema_explorer::{SchemaExplorer, ExplorerResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FrameworkInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub detection_hints: Vec<String>,
}

#[tauri::command]
pub fn list_frameworks() -> Vec<FrameworkInfo> {
    vec![
        FrameworkInfo {
            id: "frappe".to_string(),
            name: "Frappe / ERPNext".to_string(),
            description: "Crawls DocType JSON files to build entity map and REST endpoints".to_string(),
            detection_hints: vec!["hooks.py".to_string(), "doctype/ folder".to_string(), "sites/ folder".to_string()],
        },
        FrameworkInfo {
            id: "django".to_string(),
            name: "Django".to_string(),
            description: "Parses models.py files for model classes and infers DRF endpoints".to_string(),
            detection_hints: vec!["manage.py".to_string(), "models.py".to_string()],
        },
        FrameworkInfo {
            id: "laravel".to_string(),
            name: "Laravel".to_string(),
            description: "Reads app/Models/*.php $fillable arrays for field discovery".to_string(),
            detection_hints: vec!["artisan".to_string(), "app/Models/".to_string()],
        },
        FrameworkInfo {
            id: "rails".to_string(),
            name: "Ruby on Rails".to_string(),
            description: "Parses db/schema.rb for table definitions".to_string(),
            detection_hints: vec!["Gemfile".to_string(), "db/schema.rb".to_string()],
        },
        FrameworkInfo {
            id: "openapi".to_string(),
            name: "OpenAPI / Swagger".to_string(),
            description: "Parses openapi.yaml/swagger.json for full schema + endpoint map".to_string(),
            detection_hints: vec!["openapi.yaml".to_string(), "swagger.json".to_string()],
        },
        FrameworkInfo {
            id: "wordpress".to_string(),
            name: "WordPress".to_string(),
            description: "Uses WP REST API introspection endpoints".to_string(),
            detection_hints: vec!["wp-content/".to_string(), "wp-json/".to_string()],
        },
    ]
}

#[tauri::command]
pub fn explore_schema(root_path: String) -> Result<ExplorerResult, String> {
    SchemaExplorer::explore(&root_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_design_spec(name: String, yaml: String, _app_handle: tauri::AppHandle) -> Result<(), String> {
    use std::fs;
    use std::path::PathBuf;
    
    // In a real app we'd get the current workspace path, but for now we'll use a local .parallax folder
    // Or we can just use the app's current directory + .parallax/design
    let mut path = std::env::current_dir().map_err(|e| e.to_string())?;
    path.push(".parallax");
    path.push("design");
    
    if !path.exists() {
        fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    }
    
    let safe_name = name.to_lowercase().replace(" ", "-").replace("/", "-");
    let file_name = format!("{}.openapi.yaml", safe_name);
    path.push(file_name);
    
    fs::write(path, yaml).map_err(|e| e.to_string())?;
    Ok(())
}
