use std::io::Write;
use std::path::PathBuf;

use super::{AuditError, AuditEvent, AuditSink};

pub struct FileSink {
    path: PathBuf,
}

impl FileSink {
    pub fn new(path: PathBuf) -> Result<Self, AuditError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(Self { path })
    }
}

impl AuditSink for FileSink {
    fn emit(&self, event: &AuditEvent) -> Result<(), AuditError> {
        let mut line = serde_json::to_string(event)?;
        line.push('\n');
        // O_APPEND + O_CREAT: POSIX guarantees write atomicity for writes <= PIPE_BUF (~4096
        // bytes) on Linux/macOS. NDJSON lines for this schema are well under that limit.
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        file.write_all(line.as_bytes())?;
        Ok(())
    }
}

pub struct StderrSink;

impl AuditSink for StderrSink {
    fn emit(&self, event: &AuditEvent) -> Result<(), AuditError> {
        let mut line = serde_json::to_string(event)?;
        line.push('\n');
        std::io::stderr().lock().write_all(line.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{AuditAction, AuditOutcome, CallerKind, build_event};
    use std::path::Path;
    use tempfile::TempDir;

    fn sample_event() -> AuditEvent {
        build_event(
            CallerKind::Cli,
            AuditAction::TaskRun,
            "build",
            None,
            "demo",
            Path::new("/tmp/demo"),
            "low",
            Some(0),
            Some(12),
            vec![],
            std::collections::BTreeMap::new(),
            AuditOutcome::Success,
        )
    }

    #[test]
    fn test_file_sink_creates_missing_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nested").join("audit.log");
        let sink = FileSink::new(path.clone()).unwrap();
        sink.emit(&sample_event()).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn test_file_sink_appends_one_line_per_emit() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("audit.log");
        let sink = FileSink::new(path.clone()).unwrap();

        sink.emit(&sample_event()).unwrap();
        sink.emit(&sample_event()).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 2);
        for line in lines {
            let parsed: serde_json::Value = serde_json::from_str(line).unwrap();
            assert_eq!(parsed["action"], "task.run");
        }
    }
}
