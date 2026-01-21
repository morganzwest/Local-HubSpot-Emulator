use crate::execution_id::ExecutionId;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ExecutionEventKind {
    ExecutionCreated,
    ValidationFailed,
    ValidationSucceeded,
    ExecutionStarted,
    ExecutionFinished,
    ValidationStarted,
    Stdout,
    Stderr,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutionEvent {
    pub execution_id: ExecutionId,
    pub kind: ExecutionEventKind,
    pub timestamp: SystemTime,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

pub fn execution_created(execution_id: ExecutionId) -> ExecutionEvent {
    ExecutionEvent {
        execution_id,
        kind: ExecutionEventKind::ExecutionCreated,
        timestamp: SystemTime::now(),
        message: None,
    }
}

pub fn stdout_event(execution_id: ExecutionId, message: String) -> ExecutionEvent {
    ExecutionEvent {
        execution_id,
        kind: ExecutionEventKind::Stdout,
        timestamp: SystemTime::now(),
        message: Some(message),
    }
}

pub fn stderr_event(execution_id: ExecutionId, message: String) -> ExecutionEvent {
    ExecutionEvent {
        execution_id,
        kind: ExecutionEventKind::Stderr,
        timestamp: SystemTime::now(),
        message: Some(message),
    }
}