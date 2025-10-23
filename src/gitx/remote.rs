use anyhow::{Context, Result};
use git2::Repository;

pub fn fetch_prune(repo: &Repository, remote_name: &str) -> Result<()> {
    // Use git command directly to properly support SSH config
    let workdir = repo.workdir().context("no workdir")?;

    let output = std::process::Command::new("git")
        .arg("fetch")
        .arg("--prune")
        .arg(remote_name)
        .current_dir(workdir)
        .output()
        .context("failed to execute git fetch")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git fetch failed: {}", stderr);
    }

    Ok(())
}

pub fn push_ff_only(repo: &Repository, remote_name: &str, branch: &str) -> Result<()> {
    // Use git command directly to properly support SSH config
    let workdir = repo.workdir().context("no workdir")?;

    let refspec = format!("refs/heads/{branch}:refs/heads/{branch}");
    let output = std::process::Command::new("git")
        .arg("push")
        .arg(remote_name)
        .arg(&refspec)
        .current_dir(workdir)
        .output()
        .context("failed to execute git push")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git push failed: {}", stderr);
    }

    Ok(())
}
