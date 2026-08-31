//! Task-level audit configuration.

use serde::{Deserialize, Serialize};

/// Task-level audit configuration.
///
/// Controls what's recorded in the audit trail, independent of who the
/// caller is (CLI, job, or MCP) — distinct from `agent:`, which only
/// affects MCP-mediated execution.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AuditSpec {
    /// Names of this task's env vars whose *values* should be included in
    /// audit events when overridden. Validation rejects listing a var typed
    /// `secret` here.
    #[serde(default)]
    pub log_env: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_empty() {
        let spec = AuditSpec::default();
        assert!(spec.log_env.is_empty());
    }

    #[test]
    fn test_parses_log_env_list() {
        let yaml = "log_env: [QUERY, LIMIT]";
        let spec: AuditSpec = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(spec.log_env, vec!["QUERY".to_string(), "LIMIT".to_string()]);
    }

    #[test]
    fn test_missing_log_env_defaults_empty() {
        let yaml = "";
        let spec: AuditSpec = serde_yaml::from_str(yaml).unwrap();
        assert!(spec.log_env.is_empty());
    }
}
