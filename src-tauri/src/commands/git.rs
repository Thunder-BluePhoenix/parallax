use git2::{
    BranchType, FetchOptions, IndexAddOption, PushOptions, RemoteCallbacks, Repository,
    Signature, StatusOptions,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct GitStatus {
    pub branch: Option<String>,
    pub changes: Vec<GitChange>,
}

#[derive(Debug, Serialize)]
pub struct GitChange {
    pub path: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct GitCommit {
    pub hash: String,
    pub author: String,
    pub email: String,
    pub message: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize)]
pub struct GitBranch {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
}

#[tauri::command]
pub async fn git_init(path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        Repository::init(&path).map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn git_status(path: String) -> Result<GitStatus, String> {
    tokio::task::spawn_blocking(move || {
        let repo = Repository::open(&path).map_err(|e| e.to_string())?;

        let branch = repo
            .head()
            .ok()
            .and_then(|h| h.shorthand().map(String::from));

        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        let statuses = repo.statuses(Some(&mut opts)).map_err(|e| e.to_string())?;

        let changes = statuses
            .iter()
            .map(|e| {
                let s = e.status();
                let status_str = if s.intersects(git2::Status::INDEX_NEW | git2::Status::WT_NEW) {
                    "added"
                } else if s.intersects(
                    git2::Status::INDEX_MODIFIED | git2::Status::WT_MODIFIED,
                ) {
                    "modified"
                } else if s.intersects(
                    git2::Status::INDEX_DELETED | git2::Status::WT_DELETED,
                ) {
                    "deleted"
                } else {
                    "changed"
                };
                GitChange {
                    path: e.path().unwrap_or("?").to_string(),
                    status: status_str.to_string(),
                }
            })
            .collect();

        Ok(GitStatus { branch, changes })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn git_commit(
    path: String,
    message: String,
    author_name: String,
    author_email: String,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let repo = Repository::open(&path).map_err(|e| e.to_string())?;

        let mut index = repo.index().map_err(|e| e.to_string())?;
        index
            .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
            .map_err(|e| e.to_string())?;
        index.write().map_err(|e| e.to_string())?;

        let tree_id = index.write_tree().map_err(|e| e.to_string())?;
        let tree = repo.find_tree(tree_id).map_err(|e| e.to_string())?;

        let sig =
            Signature::now(&author_name, &author_email).map_err(|e| e.to_string())?;

        let parent_commits: Vec<git2::Commit> = match repo.head() {
            Ok(head) => vec![head.peel_to_commit().map_err(|e| e.to_string())?],
            Err(_) => vec![],
        };
        let parent_refs: Vec<&git2::Commit> = parent_commits.iter().collect();

        let oid = repo
            .commit(Some("HEAD"), &sig, &sig, &message, &tree, &parent_refs)
            .map_err(|e| e.to_string())?;

        Ok(oid.to_string()[..8].to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn git_push(
    path: String,
    remote_name: String,
    branch: String,
    token: Option<String>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let repo = Repository::open(&path).map_err(|e| e.to_string())?;
        let mut remote = repo
            .find_remote(&remote_name)
            .map_err(|e| e.to_string())?;

        let mut callbacks = RemoteCallbacks::new();
        if let Some(tok) = token {
            callbacks.credentials(move |_url, _username, _allowed| {
                git2::Cred::userpass_plaintext("x-access-token", &tok)
            });
        }

        let mut push_opts = PushOptions::new();
        push_opts.remote_callbacks(callbacks);

        let refspec = format!("refs/heads/{}:refs/heads/{}", branch, branch);
        remote
            .push(&[refspec.as_str()], Some(&mut push_opts))
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn git_pull(
    path: String,
    remote_name: String,
    branch: String,
    token: Option<String>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let repo = Repository::open(&path).map_err(|e| e.to_string())?;

        let mut callbacks = RemoteCallbacks::new();
        if let Some(tok) = token {
            callbacks.credentials(move |_url, _username, _allowed| {
                git2::Cred::userpass_plaintext("x-access-token", &tok)
            });
        }

        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);

        repo.find_remote(&remote_name)
            .map_err(|e| e.to_string())?
            .fetch(&[&branch], Some(&mut fetch_opts), None)
            .map_err(|e| e.to_string())?;

        let fetch_head = repo
            .find_reference("FETCH_HEAD")
            .map_err(|e| e.to_string())?;
        let fetch_commit = repo
            .reference_to_annotated_commit(&fetch_head)
            .map_err(|e| e.to_string())?;

        let (analysis, _) = repo
            .merge_analysis(&[&fetch_commit])
            .map_err(|e| e.to_string())?;

        if analysis.is_fast_forward() {
            let refname = format!("refs/heads/{}", branch);
            let mut reference = repo
                .find_reference(&refname)
                .map_err(|e| e.to_string())?;
            reference
                .set_target(fetch_commit.id(), "Fast-forward")
                .map_err(|e| e.to_string())?;
            repo.set_head(&refname).map_err(|e| e.to_string())?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .map_err(|e| e.to_string())?;
        } else if analysis.is_normal() {
            return Err("Merge conflicts — manual resolution required".to_string());
        }

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn git_stash(path: String, message: Option<String>) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let mut repo = Repository::open(&path).map_err(|e| e.to_string())?;
        let sig = repo.signature().map_err(|e| e.to_string())?;
        let msg = message.as_deref().unwrap_or("parallax stash");
        repo.stash_save(&sig, msg, None).map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn git_stash_pop(path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let mut repo = Repository::open(&path).map_err(|e| e.to_string())?;
        repo.stash_pop(0, None).map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn git_branches(path: String) -> Result<Vec<GitBranch>, String> {
    tokio::task::spawn_blocking(move || {
        let repo = Repository::open(&path).map_err(|e| e.to_string())?;
        let head_name = repo
            .head()
            .ok()
            .and_then(|h| h.shorthand().map(String::from));

        let branches = repo.branches(None).map_err(|e| e.to_string())?;
        let mut result = Vec::new();
        for item in branches {
            let (branch, btype) = item.map_err(|e| e.to_string())?;
            let name = branch
                .name()
                .map_err(|e| e.to_string())?
                .unwrap_or("?")
                .to_string();
            let is_remote = btype == BranchType::Remote;
            let is_current = head_name.as_deref() == Some(&name);
            result.push(GitBranch { name, is_current, is_remote });
        }
        Ok(result)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn git_create_branch(path: String, branch_name: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let repo = Repository::open(&path).map_err(|e| e.to_string())?;
        let head = repo.head().map_err(|e| e.to_string())?;
        let commit = head.peel_to_commit().map_err(|e| e.to_string())?;
        repo.branch(&branch_name, &commit, false)
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn git_checkout_branch(path: String, branch_name: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let repo = Repository::open(&path).map_err(|e| e.to_string())?;
        let refname = format!("refs/heads/{}", branch_name);
        repo.set_head(&refname).map_err(|e| e.to_string())?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn git_log(path: String, limit: usize) -> Result<Vec<GitCommit>, String> {
    tokio::task::spawn_blocking(move || {
        let repo = Repository::open(&path).map_err(|e| e.to_string())?;
        let mut revwalk = repo.revwalk().map_err(|e| e.to_string())?;
        revwalk.push_head().map_err(|e| e.to_string())?;

        let cap = if limit == 0 { 50 } else { limit };
        let mut commits = Vec::new();
        for (i, oid) in revwalk.enumerate() {
            if i >= cap {
                break;
            }
            let oid = oid.map_err(|e| e.to_string())?;
            let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;
            let hash = oid.to_string();
            commits.push(GitCommit {
                hash: hash[..8].to_string(),
                author: commit.author().name().unwrap_or("?").to_string(),
                email: commit.author().email().unwrap_or("").to_string(),
                message: commit.summary().unwrap_or("").to_string(),
                timestamp: commit.time().seconds(),
            });
        }
        Ok(commits)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn git_diff(path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let repo = Repository::open(&path).map_err(|e| e.to_string())?;
        let head_tree = repo.head().ok().and_then(|h| h.peel_to_tree().ok());

        let diff = repo
            .diff_tree_to_workdir_with_index(head_tree.as_ref(), None)
            .map_err(|e| e.to_string())?;

        let mut out = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            if let Ok(s) = std::str::from_utf8(line.content()) {
                out.push_str(s);
            }
            true
        })
        .map_err(|e| e.to_string())?;

        Ok(out)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn git_set_remote(path: String, url: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let repo = Repository::open(&path).map_err(|e| e.to_string())?;
        match repo.find_remote("origin") {
            Ok(_) => repo
                .remote_set_url("origin", &url)
                .map_err(|e| e.to_string())?,
            Err(_) => {
                repo.remote("origin", &url).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}
