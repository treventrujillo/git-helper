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
        if let Some(s) = file {
            if let Ok(file_config) = toml::from_str::<FileConfig>(&s) {
                main = file_config
                    .defaults
                    .as_ref()
                    .and_then(|defaults| defaults.main.clone());
                remote = file_config
                    .defaults
                    .as_ref()
                    .and_then(|defaults| defaults.remote.clone());
            }
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
