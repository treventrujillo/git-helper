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
