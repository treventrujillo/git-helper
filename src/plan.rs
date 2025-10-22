use std::fmt;

#[derive(Debug, Clone)]
pub enum SyncOp {
    FetchPrune {
        remote: String,
    },
    FastForward {
        branch: String,
    },
    RebaseOnto {
        src_branch: String,
        onto_branch: String,
        non_interactive: bool,
    },
    PushIfFastForward {
        remote: String,
        branch: String,
    },
}

#[derive(Debug, Default, Clone)]
pub struct SyncPlan {
    pub ops: Vec<SyncOp>,
}

impl SyncPlan {
    pub fn new() -> Self {
        Self { ops: vec![] }
    }
    pub fn push(&mut self, op: SyncOp) {
        self.ops.push(op);
    }
}

impl fmt::Display for SyncPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for op in &self.ops {
            match op {
                SyncOp::FetchPrune { remote } => writeln!(f, "• fetch --prune {remote}")?,
                SyncOp::FastForward { branch } => {
                    writeln!(f, "• fast-forward {branch} from its upstream")?
                }
                SyncOp::RebaseOnto {
                    src_branch,
                    onto_branch,
                    ..
                } => writeln!(f, "• rebase {src_branch} onto {onto_branch}")?,
                SyncOp::PushIfFastForward { remote, branch } => {
                    writeln!(f, "• push {branch} -> {remote}/{branch} (ff-only)")?
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_plan_new() {
        let plan = SyncPlan::new();
        assert_eq!(plan.ops.len(), 0);
    }

    #[test]
    fn test_sync_plan_push() {
        let mut plan = SyncPlan::new();
        plan.push(SyncOp::FetchPrune {
            remote: "origin".to_string(),
        });
        assert_eq!(plan.ops.len(), 1);
    }

    #[test]
    fn test_sync_plan_display_fetch_prune() {
        let mut plan = SyncPlan::new();
        plan.push(SyncOp::FetchPrune {
            remote: "origin".to_string(),
        });
        let output = format!("{}", plan);
        assert_eq!(output, "• fetch --prune origin\n");
    }

    #[test]
    fn test_sync_plan_display_fast_forward() {
        let mut plan = SyncPlan::new();
        plan.push(SyncOp::FastForward {
            branch: "main".to_string(),
        });
        let output = format!("{}", plan);
        assert_eq!(output, "• fast-forward main from its upstream\n");
    }

    #[test]
    fn test_sync_plan_display_rebase_onto() {
        let mut plan = SyncPlan::new();
        plan.push(SyncOp::RebaseOnto {
            src_branch: "feature".to_string(),
            onto_branch: "main".to_string(),
            non_interactive: false,
        });
        let output = format!("{}", plan);
        assert_eq!(output, "• rebase feature onto main\n");
    }

    #[test]
    fn test_sync_plan_display_push_if_fast_forward() {
        let mut plan = SyncPlan::new();
        plan.push(SyncOp::PushIfFastForward {
            remote: "origin".to_string(),
            branch: "feature".to_string(),
        });
        let output = format!("{}", plan);
        assert_eq!(output, "• push feature -> origin/feature (ff-only)\n");
    }

    #[test]
    fn test_sync_plan_display_multiple_ops() {
        let mut plan = SyncPlan::new();
        plan.push(SyncOp::FetchPrune {
            remote: "origin".to_string(),
        });
        plan.push(SyncOp::FastForward {
            branch: "main".to_string(),
        });
        plan.push(SyncOp::RebaseOnto {
            src_branch: "feature".to_string(),
            onto_branch: "main".to_string(),
            non_interactive: true,
        });
        let output = format!("{}", plan);
        assert_eq!(
            output,
            "• fetch --prune origin\n\
             • fast-forward main from its upstream\n\
             • rebase feature onto main\n"
        );
    }
}
