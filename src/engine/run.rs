use anyhow::Result;

use crate::engine::sink::EventSink;
use crate::{
    config::Config,
    engine::{
        events::{execution_created, ExecutionEvent, ExecutionEventKind},
        execute_action,
        summary::ExecutionSummary,
        validate::validate_config,
        validate_inline::validate_inline_config,
        ExecutionMode,
    },
    execution_id::ExecutionId,
    sinks::collecting::CollectingEventSink,
};
use crate::inline::InlineConfig;
use tempfile::tempdir;
use std::fs;
use std::path::Path;

/// CLI / file-based execution
pub async fn run_execution(
    cfg: Config,
    mode: ExecutionMode,
) -> Result<(ExecutionSummary, CollectingEventSink)> {
    let mut sink = CollectingEventSink::new();
    let execution_id = ExecutionId::new();

    sink.emit(execution_created(execution_id.clone()));
    sink.emit(validation_started(&execution_id));

    let validation = validate_config(&cfg)?;

    if !validation.is_valid() {
        sink.emit(validation_failed(&execution_id));
        return Ok((ExecutionSummary::validation_failed(execution_id), sink));
    }

    if mode == ExecutionMode::Validate {
        return Ok((ExecutionSummary::validated_only(execution_id), sink));
    }

    let result = execute_action(cfg, execution_id.clone(), &mut sink).await?;
    Ok((ExecutionSummary::executed(execution_id, result), sink))
}

/// HTTP / inline execution
pub async fn run_inline_execution(
    inline: InlineConfig,
    mode: ExecutionMode,
) -> Result<(ExecutionSummary, CollectingEventSink)> {
    let mut sink = CollectingEventSink::new();
    let execution_id = ExecutionId::new();

    sink.emit(execution_created(execution_id.clone()));
    sink.emit(validation_started(&execution_id));

    validate_inline_config(&inline)?;

    if mode == ExecutionMode::Validate {
        return Ok((ExecutionSummary::validated_only(execution_id), sink));
    }

    // ---- materialise workspace ----
    let temp = tempdir()?;
    let root = temp.path();

    // action
    write_file(root, &inline.action.entry, &inline.action.source)?;

    // fixtures
    for f in &inline.fixtures {
        write_file(root, &f.name, &f.source)?;
    }

    // convert inline → standard Config
    let cfg = inline.into_config(root);

    let result = execute_action(cfg, execution_id.clone(), &mut sink).await?;
    Ok((ExecutionSummary::executed(execution_id, result), sink))
}

/* ---------------- helpers ---------------- */

fn write_file(root: &Path, rel: &str, contents: &str) -> Result<()> {
    let path = root.join(rel);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, contents)?;
    Ok(())
}

fn validation_started(id: &ExecutionId) -> ExecutionEvent {
    ExecutionEvent {
        execution_id: id.clone(),
        kind: ExecutionEventKind::ValidationStarted,
        timestamp: std::time::SystemTime::now(),
        message: None,
    }
}

fn validation_failed(id: &ExecutionId) -> ExecutionEvent {
    ExecutionEvent {
        execution_id: id.clone(),
        kind: ExecutionEventKind::ValidationFailed,
        timestamp: std::time::SystemTime::now(),
        message: None,
    }
}
