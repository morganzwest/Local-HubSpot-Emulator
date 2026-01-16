use crate::config::Config;
use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ExecutionResult {
    pub ok: bool,
    pub runs: u64,
    pub failures: Vec<String>,
    pub max_duration_ms: Option<u128>,
    pub max_memory_kb: Option<u64>,
    pub snapshots_ok: bool,
}

pub async fn execute_action(
    cfg: Config,
    assertion_file: Option<std::path::PathBuf>,
) -> Result<ExecutionResult> {
    let summary = crate::runner::execute(cfg, assertion_file).await?;

    Ok(ExecutionResult {
        ok: summary.ok,
        runs: summary.runs,
        failures: summary.failures,
        max_duration_ms: summary.max_duration_ms,
        max_memory_kb: summary.max_memory_kb,
        snapshots_ok: summary.snapshots_ok,
    })
}
