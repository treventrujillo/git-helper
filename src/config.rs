use crate::gitx::GitRepo;
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
struct FileConfig {
    defaults: Option<Defaults>,
}

#[derive(Debug, Deserialize, Default)]
struct Defaults {
    main: Option<String>,
    remote: Option<String>,
}

pub struct ResolvedConfig {
    pub main: String,
    pub remote: String,
}

impl ResolvedConfig {
    pub fn load(path: Option<&str>, repo: &GitRepo, override_main: Option<&str>) -> Result<Self> {
        let file = path.and_then(|path| std::fs::read_to_string(path).ok());
        let mut main = None;
        let mut remote = None;
        if let Some(s) = file
            && let Ok(file_config) = toml::from_str::<FileConfig>(&s)
        {
            main = file_config
                .defaults
                .as_ref()
                .and_then(|defaults| defaults.main.clone());
            remote = file_config
                .defaults
                .as_ref()
                .and_then(|defaults| defaults.remote.clone());
        }

        let remote =
            remote.unwrap_or_else(|| repo.default_remote().unwrap_or_else(|_| "origin".into()));
        let main = override_main
            .map(|s| s.to_string())
            .or(main)
            .unwrap_or_else(|| {
                repo.remote_head_default_branch(&remote)
                    .unwrap_or_else(|_| "main".into())
            });

        Ok(Self { main, remote })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_config_deserialize_empty() {
        let toml_content = "";
        let config: Result<FileConfig, _> = toml::from_str(toml_content);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert!(config.defaults.is_none());
    }

    #[test]
    fn test_file_config_deserialize_with_defaults() {
        let toml_content = r#"
                                [defaults]
                                main = "master"
                                remote = "upstream"
                                "#;
        let config: Result<FileConfig, _> = toml::from_str(toml_content);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert!(config.defaults.is_some());
        let defaults = config.defaults.unwrap();
        assert_eq!(defaults.main, Some("master".to_string()));
        assert_eq!(defaults.remote, Some("upstream".to_string()));
    }

    #[test]
    fn test_file_config_deserialize_partial_defaults() {
        let toml_content = r#"
                                [defaults]
                                main = "develop"
                                "#;
        let config: Result<FileConfig, _> = toml::from_str(toml_content);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert!(config.defaults.is_some());
        let defaults = config.defaults.unwrap();
        assert_eq!(defaults.main, Some("develop".to_string()));
        assert_eq!(defaults.remote, None);
    }

    #[test]
    fn test_resolved_config_with_file() -> Result<()> {
        // Create a temporary config file
        let mut temp_file = NamedTempFile::new()?;
        writeln!(
            temp_file,
            r#"
            [defaults]
            main = "master"
            remote = "upstream"
            "#
        )?;
        temp_file.flush()?;

        // Create a temporary git repository for testing
        let temp_dir = tempfile::tempdir()?;
        let repo = git2::Repository::init(temp_dir.path())?;

        // Create an initial commit
        let sig = repo.signature()?;
        let tree_id = {
            let mut index = repo.index()?;
            index.write_tree()?
        };
        let tree = repo.find_tree(tree_id)?;
        let _ = repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])?;
        drop(tree);
        drop(sig);

        let git_repo = GitRepo { inner: repo };

        let config =
            ResolvedConfig::load(Some(temp_file.path().to_str().unwrap()), &git_repo, None)?;

        assert_eq!(config.main, "master");
        assert_eq!(config.remote, "upstream");

        Ok(())
    }

    #[test]
    fn test_resolved_config_with_override() -> Result<()> {
        // Create a temporary config file
        let mut temp_file = NamedTempFile::new()?;
        writeln!(
            temp_file,
            r#"
            [defaults]
            main = "master"
            remote = "upstream"
            "#
        )?;
        temp_file.flush()?;

        // Create a temporary git repository for testing
        let temp_dir = tempfile::tempdir()?;
        let repo = git2::Repository::init(temp_dir.path())?;

        // Create an initial commit
        let sig = repo.signature()?;
        let tree_id = {
            let mut index = repo.index()?;
            index.write_tree()?
        };
        let tree = repo.find_tree(tree_id)?;
        let _ = repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])?;
        drop(tree);
        drop(sig);

        let git_repo = GitRepo { inner: repo };

        let config = ResolvedConfig::load(
            Some(temp_file.path().to_str().unwrap()),
            &git_repo,
            Some("override-main"),
        )?;

        assert_eq!(config.main, "override-main");
        assert_eq!(config.remote, "upstream");

        Ok(())
    }

    #[test]
    fn test_resolved_config_defaults() -> Result<()> {
        // Create a temporary git repository for testing
        let temp_dir = tempfile::tempdir()?;
        let repo = git2::Repository::init(temp_dir.path())?;

        // Create an initial commit
        let sig = repo.signature()?;
        let tree_id = {
            let mut index = repo.index()?;
            index.write_tree()?
        };
        let tree = repo.find_tree(tree_id)?;
        let _ = repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])?;
        drop(tree);
        drop(sig);

        let git_repo = GitRepo { inner: repo };

        // Load with no config file
        let config = ResolvedConfig::load(None, &git_repo, None)?;

        // Should fallback to defaults
        assert_eq!(config.remote, "origin");
        // Main will be either "main" or determined from remote HEAD
        assert!(!config.main.is_empty());

        Ok(())
    }
}
