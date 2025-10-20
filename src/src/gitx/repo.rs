use anyhow::{Context, Result};
use git2::{BranchType, Oid, Repository};
use std::path::PathBuf;

pub struct OpenRepoOpts {
    pub workdir: PathBuf,
}

pub struct GitRepo {
    pub inner: Repository,
}

impl GitRepo {
    pub fn discover(opts: OpenRepoOpts) -> Result<Self> {
        let repo = Repository::discover(opts.workdir)?;
        Ok(Self { inner: repo })
    }

    pub fn current_branch_name(&self) -> Result<String> {
        let head = self.inner.head()?;
        let name = head
            .shorthand()
            .ok_or_else(|| anyhow::anyhow!("detached HEAD"))?;
        Ok(name.to_string())
    }

    pub fn default_remote(&self) -> Result<String> {
        // TODO: read from config; fallback to "origin"
        Ok("origin".into())
    }

    pub fn remote_head_default_branch(&self, remote: &str) -> Result<String> {
        // Try to resolve refs/remotes/<remote>/HEAD -> refs/remotes/{remote}/<main>
        let sym = self
            .inner
            .find_reference(&format!("refs/remotes/{remote}/HEAD"))?;
        let target = sym
            .symbolic_target()
            .ok_or_else(|| anyhow::anyhow!("remote HEAD not smybolic"))?;
        let name = target.rsplit('/').next().context("parse default branch")?;
        Ok(name.to_string())
    }

    pub fn is_ff_up_to_remote(&self, branch: &str) -> Result<bool> {
        let local = self
            .branch_tip(branch, BranchType::Local)
            .context("local branch tip")?;
        let remote = self
            .branch_tip(branch, BranchType::Remote)
            .context("remote branch tip")?;
        let base = self.inner.merge_base(local, remote)?;
        Ok(base == remote) // remote is ancestor of local -> local contains remote
    }

    pub fn is_branch_ancestor_of(&self, child: &str, parent: &str) -> Result<bool> {
        let child_oid = self.branch_tip(child, BranchType::Local)?;
        let parent_oid = self
            .branch_tip(parent, BranchType::Remote)
            .or_else(|_| self.branch_tip(&format!("origin/{parent}"), BranchType::Remote))?;
        let base = self.inner.merge_base(child_oid, parent_oid)?;
        Ok(base == child_oid)
    }

    pub fn fetch_prune(&self, remote: &str) -> Result<()> {
        super::remote::fetch_prune(&self.inner, remote)
    }

    pub fn fast_forward_branch(&self, branch: &str) -> Result<()> {
        super::refs::fast_forward_to_upstream(&self.inner, branch)
    }

    pub fn rebase_onto(
        &self,
        src_branch: &str,
        onto_branch: &str,
        non_interactive: bool,
    ) -> Result<()> {
        super::rebase::rebase_onto(&self.inner, src_branch, onto_branch, non_interactive)
    }

    pub fn push_if_ff(&self, remote: &str, branch: &str) -> Result<()> {
        super::remote::push_ff_only(&self.inner, remote, branch)
    }

    fn branch_tip(&self, name: &str, kind: BranchType) -> Result<Oid> {
        self.find_branch(name, kind)
    }

    fn find_branch(&self, name: &str, kind: BranchType) -> Result<Oid> {
        let branch = self.inner.find_branch(name, kind)?;
        Ok(branch
            .get()
            .target()
            .context(format!("{kind:?} branch has no target"))?)
    }
}
