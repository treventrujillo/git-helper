use git2::{Repository, Signature};
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper struct to manage a test git repository
struct TestRepo {
    _temp_dir: TempDir,
    repo: Repository,
}

impl TestRepo {
    fn new() -> anyhow::Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let repo = Repository::init(temp_dir.path())?;

        // Set user config for commits
        repo.config()?.set_str("user.name", "Test User")?;
        repo.config()?.set_str("user.email", "test@example.com")?;

        Ok(Self {
            _temp_dir: temp_dir,
            repo,
        })
    }

    fn path(&self) -> PathBuf {
        self.repo.path().parent().unwrap().to_path_buf()
    }

    fn commit(&self, message: &str) -> anyhow::Result<git2::Oid> {
        let sig = self.signature()?;
        let tree_id = self.repo.index()?.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;

        let parent_commit = self.repo.head().ok().and_then(|head| {
            head.target()
                .and_then(|oid| self.repo.find_commit(oid).ok())
        });

        let oid = if let Some(parent) = parent_commit {
            self.repo
                .commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])?
        } else {
            self.repo
                .commit(Some("HEAD"), &sig, &sig, message, &tree, &[])?
        };

        Ok(oid)
    }

    fn signature(&self) -> anyhow::Result<Signature<'_>> {
        Ok(self.repo.signature()?)
    }

    fn create_branch(&self, name: &str) -> anyhow::Result<()> {
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;
        self.repo.branch(name, &commit, false)?;
        Ok(())
    }

    fn checkout_branch(&self, name: &str) -> anyhow::Result<()> {
        let obj = self.repo.revparse_single(&format!("refs/heads/{}", name))?;
        self.repo.checkout_tree(&obj, None)?;
        self.repo.set_head(&format!("refs/heads/{}", name))?;
        Ok(())
    }

    fn current_branch(&self) -> anyhow::Result<String> {
        let head = self.repo.head()?;
        let name = head
            .shorthand()
            .ok_or_else(|| anyhow::anyhow!("no shorthand"))?;
        Ok(name.to_string())
    }

    fn add_file(&self, path: &str, content: &str) -> anyhow::Result<()> {
        let file_path = self.path().join(path);
        std::fs::write(file_path, content)?;
        let mut index = self.repo.index()?;
        index.add_path(std::path::Path::new(path))?;
        index.write()?;
        Ok(())
    }
}

#[test]
fn test_repo_creation() -> anyhow::Result<()> {
    let test_repo = TestRepo::new()?;
    assert!(test_repo.repo.is_empty()?);
    Ok(())
}

#[test]
fn test_initial_commit() -> anyhow::Result<()> {
    let test_repo = TestRepo::new()?;
    test_repo.add_file("test.txt", "hello")?;
    let oid = test_repo.commit("Initial commit")?;
    assert!(!oid.is_zero());
    assert!(!test_repo.repo.is_empty()?);
    Ok(())
}

#[test]
fn test_branch_creation() -> anyhow::Result<()> {
    let test_repo = TestRepo::new()?;
    test_repo.add_file("test.txt", "hello")?;
    test_repo.commit("Initial commit")?;

    test_repo.create_branch("feature")?;

    let branch = test_repo
        .repo
        .find_branch("feature", git2::BranchType::Local)?;
    assert!(branch.get().is_branch());
    Ok(())
}

#[test]
fn test_branch_checkout() -> anyhow::Result<()> {
    let test_repo = TestRepo::new()?;
    test_repo.add_file("test.txt", "hello")?;
    test_repo.commit("Initial commit")?;

    test_repo.create_branch("feature")?;
    test_repo.checkout_branch("feature")?;

    assert_eq!(test_repo.current_branch()?, "feature");
    Ok(())
}

#[test]
fn test_multiple_commits() -> anyhow::Result<()> {
    let test_repo = TestRepo::new()?;

    test_repo.add_file("file1.txt", "content 1")?;
    let oid1 = test_repo.commit("First commit")?;

    test_repo.add_file("file2.txt", "content 2")?;
    let oid2 = test_repo.commit("Second commit")?;

    assert_ne!(oid1, oid2);

    // Verify the second commit has the first as parent
    let commit2 = test_repo.repo.find_commit(oid2)?;
    assert_eq!(commit2.parent_count(), 1);
    assert_eq!(commit2.parent_id(0)?, oid1);

    Ok(())
}

#[test]
fn test_branch_divergence() -> anyhow::Result<()> {
    let test_repo = TestRepo::new()?;

    // Create initial commit on current branch (might be master or main)
    test_repo.add_file("base.txt", "base")?;
    test_repo.commit("Initial commit")?;

    let base_branch = test_repo.current_branch()?;

    // Create and checkout feature branch
    test_repo.create_branch("feature")?;
    test_repo.checkout_branch("feature")?;

    // Make commit on feature
    test_repo.add_file("feature.txt", "feature work")?;
    let feature_oid = test_repo.commit("Feature commit")?;

    // Checkout base branch and make different commit
    test_repo.checkout_branch(&base_branch)?;
    test_repo.add_file("base_work.txt", "base work")?;
    let base_oid = test_repo.commit("Base branch commit")?;

    // Verify commits are different
    assert_ne!(feature_oid, base_oid);

    // Verify both branches exist
    let base_branch_ref = test_repo
        .repo
        .find_branch(&base_branch, git2::BranchType::Local)?;
    let feature_branch = test_repo
        .repo
        .find_branch("feature", git2::BranchType::Local)?;

    assert_ne!(
        base_branch_ref.get().target(),
        feature_branch.get().target()
    );

    Ok(())
}

#[test]
fn test_repo_with_remote_simulation() -> anyhow::Result<()> {
    // Create "remote" repository
    let remote_repo = TestRepo::new()?;
    remote_repo.add_file("readme.txt", "readme")?;
    remote_repo.commit("Initial commit")?;

    // Create "local" repository
    let local_repo = TestRepo::new()?;
    local_repo.add_file("local.txt", "local")?;
    local_repo.commit("Local initial")?;

    // Add remote (we can test the remote operations structure)
    local_repo
        .repo
        .remote("origin", remote_repo.path().to_str().unwrap())?;

    // Verify remote exists
    let remote = local_repo.repo.find_remote("origin")?;
    assert_eq!(remote.name(), Some("origin"));

    Ok(())
}
