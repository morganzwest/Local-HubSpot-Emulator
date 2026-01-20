use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant, SystemTime};
use tempfile::tempdir;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;

use crate::config::Config;
use crate::engine::events::{ExecutionEvent, ExecutionEventKind};
use crate::engine::sink::EventSink;
use crate::engine::ExecutionResult;
use crate::execution_id::ExecutionId;
use crate::metrics::{InvocationMetrics, MemoryTracker};
use crate::shim::{node_shim, python_shim};

/* ============================================================
   Public entrypoint
============================================================ */

pub async fn execute_action(
    cfg: Config,
    execution_id: ExecutionId,
    sink: &mut dyn EventSink,
) -> Result<ExecutionResult> {
    sink.emit(event(
        execution_id.clone(),
        ExecutionEventKind::ExecutionStarted,
    ));

    let action = cfg
        .action
        .as_ref()
        .context("Missing action configuration")?;

    let action_file = PathBuf::from(&action.entry)
        .canonicalize()
        .context("Unable to resolve action entry")?;

    let mut runs: u64 = 0;
    let mut failures: Vec<String> = Vec::new();
    let mut max_duration_ms: Option<u128> = None;
    let mut max_memory_kb: Option<u64> = None;

    for fixture in &cfg.fixtures {
        let event: Value = serde_json::from_str(
            &std::fs::read_to_string(fixture)
                .with_context(|| format!("Failed to read fixture: {}", fixture))?,
        )
        .with_context(|| format!("Fixture is not valid JSON: {}", fixture))?;

        let repeat = cfg.repeat.max(1);

        for _ in 0..repeat {
            runs += 1;

            let (output, metrics) =
                invoke_once(&cfg, &execution_id, &action_file, &event, sink).await?;

            max_duration_ms = Some(
                max_duration_ms
                    .map(|d| d.max(metrics.duration_ms))
                    .unwrap_or(metrics.duration_ms),
            );

            if let Some(mem) = metrics.max_rss_kb {
                max_memory_kb = Some(max_memory_kb.map(|m| m.max(mem)).unwrap_or(mem));
            }

            if !output.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                failures.push("Action returned ok=false".to_string());
            }
        }
    }

    sink.emit(event(
        execution_id.clone(),
        ExecutionEventKind::ExecutionFinished,
    ));

    Ok(ExecutionResult {
        ok: failures.is_empty(),
        runs,
        failures,
        max_duration_ms,
        max_memory_kb,
        snapshots_ok: true,
    })
}

/* ============================================================
   Single invocation (NO tokio::spawn)
============================================================ */

async fn invoke_once(
    cfg: &Config,
    execution_id: &ExecutionId,
    action_file: &Path,
    event: &Value,
    sink: &mut dyn EventSink,
) -> Result<(Value, InvocationMetrics)> {
    let tmp = tempdir().context("Failed to create temp dir")?;

    let event_path = tmp.path().join("event.json");
    std::fs::write(&event_path, serde_json::to_vec_pretty(event)?)
        .context("Failed to write event.json")?;

    let ext = action_file
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let (runtime, shim_name, shim_code) = match ext.as_str() {
        "py" => (&cfg.runtime.python, "hs_python_runner.py", python_shim()),
        "js" | "mjs" | "cjs" => (&cfg.runtime.node, "hs_node_runner.mjs", node_shim()),
        _ => bail!("Unsupported action file extension: {}", ext),
    };

    let shim_path = tmp.path().join(shim_name);
    std::fs::write(&shim_path, shim_code).context("Failed to write runner shim")?;

    let mut cmd = TokioCommand::new(runtime);
    cmd.arg(&shim_path)
        .arg(action_file)
        .arg(&event_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    for (k, v) in &cfg.env {
        cmd.env(k, v);
    }

    let start = Instant::now();
    let mut child = cmd.spawn().context("Failed to spawn runtime")?;

    let pid = child.id().context("Failed to get child PID")?;
    let mem = MemoryTracker::start(pid, Duration::from_millis(20));

    let mut stdout_lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let mut stderr_lines = BufReader::new(child.stderr.take().unwrap()).lines();

    let mut final_stdout = String::new();

    loop {
        tokio::select! {
            line = stdout_lines.next_line() => {
                match line? {
                    Some(l) => final_stdout.push_str(&l),
                    None => break,
                }
            }
            line = stderr_lines.next_line() => {
                if let Some(l) = line? {
                    sink.emit(ExecutionEvent {
                        execution_id: execution_id.clone(),
                        kind: ExecutionEventKind::Stderr,
                        timestamp: SystemTime::now(),
                        message: Some(l),
                    });
                }
            }
        }
    }

    let status = child.wait().await?;
    let duration_ms = start.elapsed().as_millis();
    let max_rss_kb = mem.stop_and_take();

    if !status.success() {
        bail!("Runtime exited with non-zero status");
    }

    let parsed: Value =
        serde_json::from_str(final_stdout.trim()).context("Shim did not emit valid JSON")?;

    Ok((
        parsed,
        InvocationMetrics {
            duration_ms,
            max_rss_kb,
        },
    ))
}

/* ============================================================
   Helpers
============================================================ */

fn event(id: ExecutionId, kind: ExecutionEventKind) -> ExecutionEvent {
    ExecutionEvent {
        execution_id: id,
        kind,
        timestamp: SystemTime::now(),
        message: None,
    }
}
