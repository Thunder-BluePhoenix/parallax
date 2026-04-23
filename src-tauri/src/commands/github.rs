use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn parallax_home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(|h| PathBuf::from(h).join(".parallax"))
}

fn auth_path() -> Option<PathBuf> {
    parallax_home().map(|p| p.join("github_auth.json"))
}

fn resolve_client_id(client_id: Option<String>) -> String {
    client_id
        .or_else(|| std::env::var("PARALLAX_GITHUB_CLIENT_ID").ok())
        .unwrap_or_default()
}

// ── Identity ──────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitHubIdentity {
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: String,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub token: String,
}

// ── Device flow ───────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct DeviceCodeInfo {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[tauri::command]
pub async fn github_device_auth_start(
    client_id: Option<String>,
) -> Result<DeviceCodeInfo, String> {
    let cid = resolve_client_id(client_id);
    if cid.is_empty() {
        return Err(
            "No GitHub client ID configured. Set PARALLAX_GITHUB_CLIENT_ID or configure it in settings.".to_string(),
        );
    }

    #[derive(Deserialize)]
    struct Resp {
        device_code: String,
        user_code: String,
        verification_uri: String,
        expires_in: u64,
        interval: u64,
    }

    let resp: Resp = Client::new()
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", cid.as_str()),
            ("scope", "repo read:user user:email"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    Ok(DeviceCodeInfo {
        device_code: resp.device_code,
        user_code: resp.user_code,
        verification_uri: resp.verification_uri,
        expires_in: resp.expires_in,
        interval: resp.interval,
    })
}

#[tauri::command]
pub async fn github_device_auth_poll(
    device_code: String,
    client_id: Option<String>,
) -> Result<Option<GitHubIdentity>, String> {
    let cid = resolve_client_id(client_id);

    #[derive(Deserialize)]
    struct TokenResp {
        access_token: Option<String>,
        error: Option<String>,
    }

    let resp: TokenResp = Client::new()
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", cid.as_str()),
            ("device_code", device_code.as_str()),
            (
                "grant_type",
                "urn:ietf:params:oauth:grant-type:device_code",
            ),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    match resp.error.as_deref() {
        Some("authorization_pending") | Some("slow_down") => return Ok(None),
        Some(e) => return Err(e.to_string()),
        None => {}
    }

    let token = resp.access_token.ok_or("No access token returned")?;

    #[derive(Deserialize)]
    struct UserResp {
        login: String,
        name: Option<String>,
        email: Option<String>,
        avatar_url: String,
    }

    let user: UserResp = Client::new()
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "Parallax/1.0")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let identity = GitHubIdentity {
        login: user.login,
        name: user.name,
        email: user.email,
        avatar_url: user.avatar_url,
        token: token.clone(),
    };

    persist_identity(&identity);

    Ok(Some(identity))
}

fn persist_identity(identity: &GitHubIdentity) {
    if let Some(home) = parallax_home() {
        let _ = std::fs::create_dir_all(&home);
        if let Ok(json) = serde_json::to_string_pretty(identity) {
            let _ = std::fs::write(home.join("github_auth.json"), json);
        }
    }
}

#[tauri::command]
pub async fn github_get_identity() -> Result<Option<GitHubIdentity>, String> {
    let path = match auth_path() {
        Some(p) => p,
        None => return Ok(None),
    };
    match std::fs::read_to_string(path) {
        Ok(json) => {
            let id: GitHubIdentity =
                serde_json::from_str(&json).map_err(|e| e.to_string())?;
            Ok(Some(id))
        }
        Err(_) => Ok(None),
    }
}

#[tauri::command]
pub async fn github_sign_out() -> Result<(), String> {
    if let Some(p) = auth_path() {
        let _ = std::fs::remove_file(p);
    }
    Ok(())
}

// ── Collaborators ─────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct Collaborator {
    pub login: String,
    pub avatar_url: String,
    pub role: String,
}

#[tauri::command]
pub async fn github_list_collaborators(
    owner: String,
    repo: String,
    token: String,
) -> Result<Vec<Collaborator>, String> {
    #[derive(Deserialize)]
    struct Resp {
        login: String,
        avatar_url: String,
        role_name: Option<String>,
    }

    let data: Vec<Resp> = Client::new()
        .get(format!(
            "https://api.github.com/repos/{}/{}/collaborators",
            owner, repo
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "Parallax/1.0")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    Ok(data
        .into_iter()
        .map(|c| Collaborator {
            login: c.login,
            avatar_url: c.avatar_url,
            role: c.role_name.unwrap_or_else(|| "write".to_string()),
        })
        .collect())
}

#[tauri::command]
pub async fn github_invite_collaborator(
    owner: String,
    repo: String,
    username: String,
    token: String,
) -> Result<(), String> {
    let resp = Client::new()
        .put(format!(
            "https://api.github.com/repos/{}/{}/collaborators/{}",
            owner, repo, username
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "Parallax/1.0")
        .header("Content-Type", "application/json")
        .body(r#"{"permission":"push"}"#)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = resp.status().as_u16();
    if status != 201 && status != 204 && !resp.status().is_success() {
        return Err(format!("GitHub API error: {}", resp.status()));
    }
    Ok(())
}

#[tauri::command]
pub async fn github_remove_collaborator(
    owner: String,
    repo: String,
    username: String,
    token: String,
) -> Result<(), String> {
    let resp = Client::new()
        .delete(format!(
            "https://api.github.com/repos/{}/{}/collaborators/{}",
            owner, repo, username
        ))
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "Parallax/1.0")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() && resp.status().as_u16() != 204 {
        return Err(format!("GitHub API error: {}", resp.status()));
    }
    Ok(())
}

// ── API Docs Publisher ────────────────────────────────────────

#[tauri::command]
pub async fn github_publish_docs(
    workspace_path: String,
    repo_owner: String,
    repo_name: String,
    token: String,
    html_content: String,
) -> Result<String, String> {
    use base64::{engine::general_purpose::STANDARD, Engine};

    let client = Client::new();
    let encoded = STANDARD.encode(html_content.as_bytes());

    // Check if gh-pages branch exists
    let branch_url = format!(
        "https://api.github.com/repos/{}/{}/git/refs/heads/gh-pages",
        repo_owner, repo_name
    );
    let branch_exists = client
        .get(&branch_url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "Parallax/1.0")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .status()
        .is_success();

    // Get or create file SHA for upsert
    let file_url = format!(
        "https://api.github.com/repos/{}/{}/contents/index.html",
        repo_owner, repo_name
    );

    #[derive(Deserialize)]
    struct FileResp {
        sha: Option<String>,
    }
    let existing_sha: Option<String> = client
        .get(&file_url)
        .query(&[("ref", "gh-pages")])
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "Parallax/1.0")
        .send()
        .await
        .ok()
        .and_then(|r| r.json::<FileResp>().ok().and_then(|f| f.sha));

    let mut body = serde_json::json!({
        "message": format!("Update API docs — {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")),
        "content": encoded,
        "branch": "gh-pages",
    });

    if !branch_exists {
        // Need to create branch from default first — get default branch SHA
        #[derive(Deserialize)]
        struct RepoResp { default_branch: String }
        #[derive(Deserialize)]
        struct BranchResp { commit: CommitObj }
        #[derive(Deserialize)]
        struct CommitObj { sha: String }

        let repo_info: RepoResp = client
            .get(format!("https://api.github.com/repos/{}/{}", repo_owner, repo_name))
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "Parallax/1.0")
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        let branch_info: BranchResp = client
            .get(format!(
                "https://api.github.com/repos/{}/{}/branches/{}",
                repo_owner, repo_name, repo_info.default_branch
            ))
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "Parallax/1.0")
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        client
            .post(format!(
                "https://api.github.com/repos/{}/{}/git/refs",
                repo_owner, repo_name
            ))
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "Parallax/1.0")
            .json(&serde_json::json!({
                "ref": "refs/heads/gh-pages",
                "sha": branch_info.commit.sha,
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;
    }

    if let Some(sha) = existing_sha {
        body["sha"] = serde_json::Value::String(sha);
    }

    client
        .put(&file_url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "Parallax/1.0")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let _ = workspace_path; // used for future local git ops
    Ok(format!(
        "https://{}.github.io/{}/",
        repo_owner, repo_name
    ))
}
