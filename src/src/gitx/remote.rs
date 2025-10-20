use anyhow::{Context, Result};
use git2::{Cred, FetchOptions, PushOptions, RemoteCallbacks, Repository};

pub fn fetch_prune(repo: &Repository, remote_name: &str) -> Result<()> {
    let mut remote = repo.find_remote(remote_name)?;
    let mut callback = RemoteCallbacks::new();

    callback.credentials(|_url, username_from_url, _allowed| {
        Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
    });

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callback);
    fetch_options.download_tags(git2::AutotagOption::All);

    remote
        .fetch(
            &["refs/heads/*:refs/remotes/origin/*"],
            Some(&mut fetch_options),
            None,
        )
        .context("fetch")?;
    Ok(())
}

pub fn push_ff_only(repo: &Repository, remote_name: &str, branch: &str) -> Result<()> {
    let mut remote = repo.find_remote(remote_name)?;
    let mut callback = RemoteCallbacks::new();

    callback.credentials(|_url, username_from_url, _allowed| {
        Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
    });

    let mut push_options = PushOptions::new();
    push_options.remote_callbacks(callback);

    let local = format!("refs/heads/{branch}");
    let remote_ref = format!("refs/heads/{branch}");
    remote
        .push(&[format!("{local}:{remote_ref}")], Some(&mut push_options))
        .context("push")?;
    Ok(())
}
