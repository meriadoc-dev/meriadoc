use super::sinks::{FileSink, StderrSink};
use super::{AuditLogger, AuditSink};
use crate::config::spec::{AuditConfig, AuditSinkConfig};

pub fn build_logger(config: &AuditConfig) -> AuditLogger {
    if !config.enabled {
        return AuditLogger::disabled();
    }
    let sinks: Vec<Box<dyn AuditSink>> = config
        .sinks
        .iter()
        .filter_map(|sink_cfg| match sink_cfg {
            AuditSinkConfig::File { path } => match FileSink::new(path.clone()) {
                Ok(s) => Some(Box::new(s) as Box<dyn AuditSink>),
                Err(e) => {
                    eprintln!("[meriadoc] failed to init audit file sink: {e}");
                    None
                }
            },
            AuditSinkConfig::Stderr => Some(Box::new(StderrSink) as Box<dyn AuditSink>),
        })
        .collect();
    AuditLogger::new(sinks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_disabled_config_yields_zero_sinks() {
        let dir = TempDir::new().unwrap();
        let config = AuditConfig {
            enabled: false,
            sinks: vec![AuditSinkConfig::File {
                path: dir.path().join("audit.log"),
            }],
        };
        let logger = build_logger(&config);
        assert!(logger.sinks.is_empty());
    }

    #[test]
    fn test_enabled_config_builds_one_sink_per_entry() {
        let dir = TempDir::new().unwrap();
        let config = AuditConfig {
            enabled: true,
            sinks: vec![
                AuditSinkConfig::File {
                    path: dir.path().join("audit.log"),
                },
                AuditSinkConfig::Stderr,
            ],
        };
        let logger = build_logger(&config);
        assert_eq!(logger.sinks.len(), 2);
    }
}
