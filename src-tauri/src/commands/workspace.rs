use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub root: String,
    pub name: String,
    pub has_parallax: bool,
    pub has_git: bool,
    pub git_branch: Option<String>,
    pub collections_count: usize,
    pub environments_count: usize,
}

#[tauri::command]
pub async fn open_workspace(path: String) -> Result<WorkspaceInfo, String> {
    let root = PathBuf::from(&path);
    let name = root.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("workspace")
        .to_string();

    let has_parallax = root.join(".parallax").exists();
    let has_git = root.join(".git").exists();

    let git_branch = if has_git {
        let head_file = root.join(".git/HEAD");
        std::fs::read_to_string(head_file)
            .ok()
            .and_then(|s| {
                s.trim().strip_prefix("ref: refs/heads/").map(String::from)
            })
    } else {
        None
    };

    let collections_count = count_files_with_ext(&root.join(".parallax/collections"), "yaml");
    let environments_count = count_files_with_ext(&root.join(".parallax/environments"), "json");

    Ok(WorkspaceInfo {
        root: path,
        name,
        has_parallax,
        has_git,
        git_branch,
        collections_count,
        environments_count,
    })
}

#[tauri::command]
pub async fn create_workspace(path: String) -> Result<(), String> {
    let root = PathBuf::from(&path);
    let parallax_dir = root.join(".parallax");
    std::fs::create_dir_all(parallax_dir.join("collections")).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(parallax_dir.join("environments")).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(parallax_dir.join("history")).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(parallax_dir.join("scripts")).map_err(|e| e.to_string())?;

    let globals = r#"{"name":"globals","variables":{}}"#;
    std::fs::write(parallax_dir.join("environments").join("globals.json"), globals)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_workspace_info(path: String) -> Result<WorkspaceInfo, String> {
    open_workspace(path).await
}

#[tauri::command]
pub async fn get_current_branch(path: String) -> Result<Option<String>, String> {
    let head_file = PathBuf::from(&path).join(".git/HEAD");
    let branch = std::fs::read_to_string(head_file)
        .ok()
        .and_then(|s| s.trim().strip_prefix("ref: refs/heads/").map(String::from));
    Ok(branch)
}

fn count_files_with_ext(dir: &PathBuf, ext: &str) -> usize {
    std::fs::read_dir(dir)
        .map(|dir| {
            dir.filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some(ext))
                .count()
        })
        .unwrap_or(0)
}
