pub mod builder;
pub mod sinks;

use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CallerKind {
    McpStdio,
    McpHttp,
    Cli,
    Api,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[allow(clippy::enum_variant_names)]
pub enum AuditAction {
    #[serde(rename = "task.run")]
    TaskRun,
    #[serde(rename = "task.dry_run")]
    TaskDryRun,
    #[serde(rename = "task.blocked")]
    TaskBlocked,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    Success,
    Failure,
    Blocked,
    DryRun,
}

#[derive(Debug, Serialize)]
pub struct AuditEvent {
    pub timestamp: String,
    pub schema_version: &'static str,
    pub caller: CallerKind,
    pub action: AuditAction,
    pub task: String,
    pub project: String,
    pub project_root: String,
    pub risk_level: String,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<u64>,
    pub env_override_keys: Vec<String>,
    pub outcome: AuditOutcome,
    pub meriadoc_version: &'static str,
    pub pid: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("audit io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("audit serialize error: {0}")]
    Serialize(#[from] serde_json::Error),
}

pub trait AuditSink: Send + Sync {
    fn emit(&self, event: &AuditEvent) -> Result<(), AuditError>;
}

pub struct AuditLogger {
    sinks: Vec<Box<dyn AuditSink>>,
}

impl AuditLogger {
    pub fn new(sinks: Vec<Box<dyn AuditSink>>) -> Self {
        Self { sinks }
    }

    pub fn disabled() -> Self {
        Self { sinks: vec![] }
    }

    pub fn emit(&self, event: &AuditEvent) {
        for sink in &self.sinks {
            if let Err(e) = sink.emit(event) {
                eprintln!("[meriadoc audit error] {e}");
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_event(
    caller: CallerKind,
    action: AuditAction,
    task: &str,
    project: &str,
    project_root: &Path,
    risk_level: &str,
    exit_code: Option<i32>,
    duration_ms: Option<u64>,
    env_override_keys: Vec<String>,
    outcome: AuditOutcome,
) -> AuditEvent {
    AuditEvent {
        timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        schema_version: "1",
        caller,
        action,
        task: task.to_string(),
        project: project.to_string(),
        project_root: project_root.to_string_lossy().into_owned(),
        risk_level: risk_level.to_string(),
        exit_code,
        duration_ms,
        env_override_keys,
        outcome,
        meriadoc_version: env!("CARGO_PKG_VERSION"),
        pid: std::process::id(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_build_event_populates_fields() {
        let event = build_event(
            CallerKind::McpStdio,
            AuditAction::TaskBlocked,
            "deploy",
            "myapp",
            Path::new("/home/user/myapp"),
            "critical",
            None,
            None,
            vec!["ENV".to_string()],
            AuditOutcome::Blocked,
        );

        assert_eq!(event.task, "deploy");
        assert_eq!(event.project, "myapp");
        assert_eq!(event.risk_level, "critical");
        assert_eq!(event.exit_code, None);
        assert_eq!(event.duration_ms, None);
        assert_eq!(event.env_override_keys, vec!["ENV".to_string()]);
        assert_eq!(event.schema_version, "1");
    }

    #[derive(Default)]
    struct RecordingSink {
        events: Mutex<Vec<String>>,
    }

    impl AuditSink for Arc<RecordingSink> {
        fn emit(&self, event: &AuditEvent) -> Result<(), AuditError> {
            self.events.lock().unwrap().push(event.task.clone());
            Ok(())
        }
    }

    #[test]
    fn test_disabled_logger_has_no_sinks_and_is_a_no_op() {
        let logger = AuditLogger::disabled();
        assert!(logger.sinks.is_empty());
        // Should not panic with zero sinks.
        let event = build_event(
            CallerKind::Cli,
            AuditAction::TaskRun,
            "build",
            "demo",
            Path::new("/tmp/demo"),
            "low",
            Some(0),
            Some(1),
            vec![],
            AuditOutcome::Success,
        );
        logger.emit(&event);
    }

    #[test]
    fn test_logger_forwards_events_to_all_sinks() {
        let sink = Arc::new(RecordingSink::default());
        let logger = AuditLogger::new(vec![Box::new(sink.clone())]);

        let event = build_event(
            CallerKind::Cli,
            AuditAction::TaskRun,
            "build",
            "demo",
            Path::new("/tmp/demo"),
            "low",
            Some(0),
            Some(1),
            vec![],
            AuditOutcome::Success,
        );
        logger.emit(&event);

        assert_eq!(sink.events.lock().unwrap().as_slice(), ["build"]);
    }
}
