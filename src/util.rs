use crate::plan::SyncPlan;
use indicatif::HumanDuration;
use std::time::Instant;
use tracing::info;

pub fn print_plan(plan: &SyncPlan) {
    info!("sync plan:\n{}", plan);
}

#[allow(dead_code)]
pub fn time_it<F, T>(label: &str, mut f: F) -> T
where
    F: FnMut() -> T,
{
    let start = Instant::now();
    let out = f();
    let elapsed = HumanDuration(start.elapsed());
    tracing::debug!("{label} took {elapsed}");
    out
}
