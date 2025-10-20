use crate::config::ResolvedConfig;
use crate::gitx::{GitRepo, OpenRepoOpts};
use crate::plan::{SyncOp, SyncPlan};
use crate::util::print_plan;
use anyhow::Result;
use tracing::{info, warn};

pub struct SyncArgs {
    pub dry_run: bool,
    pub main_override: Option<String>,
    pub push: bool,
    pub non_interactive: bool,
    pub config_path: Option<String>,
}

pub fn run_sync(args: SyncArgs) -> Result<()> {
    let repo = GitRepo::discover(OpenRepoOpts {
        workdir: ".".into(),
    })?;
    let config = ResolvedConfig::load(
        args.config_path.as_deref(),
        &repo,
        args.main_override.as_deref(),
    )?;

    info!("default branch: {}", config.main);

    let plan = build_sync_plan(&repo, &config, &args)?;

    print_plan(&plan, args.dry_run);

    if !args.dry_run {
        apply_plan(&repo, &config, &plan, &args)?;
    } else {
        info!("dry-run: no changes applied");
    }

    Ok(())
}

fn build_sync_plan(repo: &GitRepo, config: &ResolvedConfig, args: &SyncArgs) -> Result<SyncPlan> {
    let mut plan = SyncPlan::new();

    plan.push(SyncOp::FetchPrune {
        remote: config.remote.clone(),
    });

    let main = config.main.clone();
    if !repo.is_ff_up_to_remote(&main)? {
        plan.push(SyncOp::FastForward {
            branch: main.clone(),
        });
    }

    let current = repo.current_branch_name()?;
    if current != main && !repo.is_branch_ancestor_of(&current, &main)? {
        plan.push(SyncOp::RebaseOnto {
            src_branch: current.clone(),
            onto_branch: main.clone(),
            non_interactive: args.non_interactive,
        });
    }

    if args.push {
        plan.push(SyncOp::PushIfFastForward {
            remote: config.remote.clone(),
            branch: current.clone(),
        });
        if current != main {
            plan.push(SyncOp::PushIfFastForward {
                remote: config.remote.clone(),
                branch: current.clone(),
            })
        }
    }

    Ok(plan)
}

fn apply_plan(
    repo: &GitRepo,
    config: &ResolvedConfig,
    plan: &SyncPlan,
    args: &SyncArgs,
) -> Result<()> {
    for op in &plan.ops {
        match op {
            SyncOp::FetchPrune { remote } => {
                repo.fetch_prune(remote)?;
            }
            SyncOp::FastForward { branch } => {
                repo.fast_forward_branch(branch)?;
            }
            SyncOp::RebaseOnto {
                src_branch,
                onto_branch,
                non_interactive,
            } => {
                repo.rebase_onto(src_branch, onto_branch, *non_interactive)?;
            }
            SyncOp::PushIfFastForward { branch, remote } => {
                if let Err(e) = repo.push_if_ff(remote, branch) {
                    warn!("push skipped for {} ({})", remote, branch);
                }
            }
        }
    }
    Ok(())
}
