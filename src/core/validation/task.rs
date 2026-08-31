use crate::core::spec::task::TaskSpec;
use crate::core::validation::{
    ConditionValidator, EnvironmentValidator, ValidationError, ValidationResult,
};

pub struct TaskValidator;

impl TaskValidator {
    pub fn validate(task_name: &str, task: &TaskSpec) -> ValidationResult {
        let mut result = ValidationResult::new();
        let context = format!("task '{task_name}'");

        // Must have at least one command
        if task.cmds.is_empty() {
            result.push(
                &context,
                ValidationError::EmptyTask {
                    task: task_name.to_string(),
                },
            );
        }

        // Commands must not be empty
        for cmd in &task.cmds {
            if cmd.trim().is_empty() {
                result.push(
                    &context,
                    ValidationError::EmptyCommandTask {
                        task: task_name.to_string(),
                    },
                );
            }
        }

        // Environment validation
        let env_result = EnvironmentValidator::validate_map(&task.env, &context);
        result.merge(env_result);

        // Preconditions
        for condition in &task.preconditions {
            let condition_result = ConditionValidator::validate(condition);

            for err in condition_result.into_errors() {
                result.push(format!("{context} precondition"), err.error);
            }
        }

        // audit.log_env must reference declared, non-secret env vars
        if let Some(audit) = &task.audit {
            for var in &audit.log_env {
                match task.env.get(var) {
                    None => result.push(
                        &context,
                        ValidationError::AuditLogEnvUnknownVar {
                            task: task_name.to_string(),
                            var: var.clone(),
                        },
                    ),
                    Some(spec) if spec.var_type.is_sensitive() => result.push(
                        &context,
                        ValidationError::AuditLogEnvIsSecret {
                            task: task_name.to_string(),
                            var: var.clone(),
                        },
                    ),
                    Some(_) => {}
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_task(cmds: Vec<&str>) -> TaskSpec {
        TaskSpec {
            description: None,
            cmds: cmds.into_iter().map(|s| s.to_string()).collect(),
            workdir: None,
            env: HashMap::new(),
            env_files: vec![],
            preconditions: vec![],
            on_failure: None,
            docs: None,
            agent: None,
            audit: None,
        }
    }

    #[test]
    fn test_valid_task() {
        let task = make_task(vec!["echo hello", "echo world"]);
        let result = TaskValidator::validate("my_task", &task);
        assert!(result.is_ok());
    }

    #[test]
    fn test_single_command_valid() {
        let task = make_task(vec!["echo hello"]);
        let result = TaskValidator::validate("my_task", &task);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_cmds_error() {
        let task = make_task(vec![]);
        let result = TaskValidator::validate("my_task", &task);

        assert!(!result.is_ok());
        let errors = result.errors();
        assert!(
            errors.iter().any(
                |e| matches!(&e.error, ValidationError::EmptyTask { task } if task == "my_task")
            )
        );
    }

    #[test]
    fn test_empty_command_string_error() {
        let task = make_task(vec!["echo hello", "", "echo world"]);
        let result = TaskValidator::validate("my_task", &task);

        assert!(!result.is_ok());
        let errors = result.errors();
        assert!(errors.iter().any(
            |e| matches!(&e.error, ValidationError::EmptyCommandTask { task } if task == "my_task")
        ));
    }

    #[test]
    fn test_whitespace_only_command_error() {
        let task = make_task(vec!["echo hello", "   ", "echo world"]);
        let result = TaskValidator::validate("my_task", &task);

        assert!(!result.is_ok());
    }

    #[test]
    fn test_task_with_invalid_env() {
        use crate::core::spec::{EnvVarSpec, VarType};

        let mut task = make_task(vec!["echo hello"]);
        task.env.insert(
            "BAD_VAR".to_string(),
            EnvVarSpec {
                var_type: VarType::Choice, // Choice without options is invalid
                default: None,
                options: vec![],
                required: false,
            },
        );

        let result = TaskValidator::validate("my_task", &task);
        assert!(!result.is_ok());
    }

    #[test]
    fn test_task_with_valid_env() {
        use crate::core::spec::{EnvVarSpec, VarType};

        let mut task = make_task(vec!["echo $MY_VAR"]);
        task.env.insert(
            "MY_VAR".to_string(),
            EnvVarSpec {
                var_type: VarType::String,
                default: Some("default".to_string()),
                options: vec![],
                required: false,
            },
        );

        let result = TaskValidator::validate("my_task", &task);
        assert!(result.is_ok());
    }

    #[test]
    fn test_task_with_preconditions() {
        use crate::core::spec::ConditionSpec;

        let mut task = make_task(vec!["echo main"]);
        task.preconditions.push(ConditionSpec {
            cmds: vec!["test -f file.txt".to_string()],
            on_failure: None,
        });

        let result = TaskValidator::validate("my_task", &task);
        assert!(result.is_ok());
    }

    #[test]
    fn test_task_with_empty_precondition_cmd() {
        use crate::core::spec::ConditionSpec;

        let mut task = make_task(vec!["echo main"]);
        task.preconditions.push(ConditionSpec {
            cmds: vec!["".to_string()], // Empty command in precondition
            on_failure: None,
        });

        let result = TaskValidator::validate("my_task", &task);
        assert!(!result.is_ok());
    }

    #[test]
    fn test_audit_log_env_unknown_var_errors() {
        use crate::core::spec::AuditSpec;

        let mut task = make_task(vec!["echo hello"]);
        task.audit = Some(AuditSpec {
            log_env: vec!["NOT_DECLARED".to_string()],
        });

        let result = TaskValidator::validate("my_task", &task);
        assert!(!result.is_ok());
        let errors = result.errors();
        assert!(errors.iter().any(|e| matches!(
            &e.error,
            ValidationError::AuditLogEnvUnknownVar { task, var }
                if task == "my_task" && var == "NOT_DECLARED"
        )));
    }

    #[test]
    fn test_audit_log_env_secret_var_errors() {
        use crate::core::spec::{AuditSpec, EnvVarSpec, VarType};

        let mut task = make_task(vec!["echo hello"]);
        task.env.insert(
            "API_KEY".to_string(),
            EnvVarSpec {
                var_type: VarType::Secret,
                default: None,
                options: vec![],
                required: true,
            },
        );
        task.audit = Some(AuditSpec {
            log_env: vec!["API_KEY".to_string()],
        });

        let result = TaskValidator::validate("my_task", &task);
        assert!(!result.is_ok());
        let errors = result.errors();
        assert!(errors.iter().any(|e| matches!(
            &e.error,
            ValidationError::AuditLogEnvIsSecret { task, var }
                if task == "my_task" && var == "API_KEY"
        )));
    }

    #[test]
    fn test_audit_log_env_valid_var_ok() {
        use crate::core::spec::{AuditSpec, EnvVarSpec, VarType};

        let mut task = make_task(vec!["echo hello"]);
        task.env.insert(
            "QUERY".to_string(),
            EnvVarSpec {
                var_type: VarType::String,
                default: None,
                options: vec![],
                required: true,
            },
        );
        task.audit = Some(AuditSpec {
            log_env: vec!["QUERY".to_string()],
        });

        let result = TaskValidator::validate("my_task", &task);
        assert!(result.is_ok());
    }
}
