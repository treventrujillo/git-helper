use anyhow::{Context, Result};
use git2::{BranchType, Repository};

pub fn rebase_onto(
    repo: &Repository,
    src_branch: &str,
    onto_branch: &str,
    _non_interactive: bool,
) -> Result<()> {
    let src = repo.find_branch(src_branch, BranchType::Local)?;
    let src_refname = src.get().name().context("src refname")?;
    repo.set_head(src_refname)?;
    repo.checkout_head(None)?;

    let onto = repo.revparse_single(&format!("refs/heads/{onto_branch}"))?;
    let upstream = repo.revparse_single(&format!("refs/heads/{src_branch}"))

    let mut rebase = repo
        .rebase(None, None, None, None)
        .context("start base (skeleton)")?;
    while let Some(_) = rebase.next() {
        // TODO: apply, handle conflicts; just auto-commit for now
        rebase.commit(None, &repo.signature()?, None)?;
    }
    let _ = rebase.finish(None);
    Ok(())
}
