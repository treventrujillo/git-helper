use anyhow::{Context, Result};
use git2::{BranchType, Repository};

pub fn fast_forward_to_upstream(repo: &Repository, branch: &str) -> Result<()> {
    let local = repo.find_branch(branch, BranchType::Local)?;
    let upstream = local.upstream().context("no upstream for branch")?;
    let upstream_oid = upstream.get().target().context("upstream no target")?;
    let annotated = repo.find_annotated_commit(upstream_oid)?;

    let (analysis, _pref) = repo.merge_analysis(&[&annotated])?;
    if analysis.is_fast_forward() {
        let name = local.get().name().unwrap_or("");
        let target = upstream_oid;
        let mut reference = repo.find_reference(name)?;
        reference.set_target(target, &format!("fast-forward {name}"))?;

        if repo.head()?.name() == reference.name() {
            repo.checkout_head(None)?;
        }
    }
    Ok(())
}
