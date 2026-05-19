use tauri::command;
use std::time::Duration;
use tokio::time::timeout;

#[command]
pub async fn eval_shell_template(cmd: String) -> Result<String, String> {
    let fut = async {
        #[cfg(target_os = "windows")]
        let mut child = tokio::process::Command::new("cmd")
            .args(["/C", &cmd])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;

        #[cfg(not(target_os = "windows"))]
        let mut child = tokio::process::Command::new("sh")
            .args(["-c", &cmd])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;

        let output = child.wait_with_output().await.map_err(|e| e.to_string())?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Shell command failed: {}", stderr.trim()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Strip trailing newline(s) — standard shell behaviour
        Ok(stdout.trim_end_matches('\n').trim_end_matches('\r').to_string())
    };

    timeout(Duration::from_secs(10), fut)
        .await
        .map_err(|_| format!("Shell command timed out after 10s: {}", cmd))?
}
