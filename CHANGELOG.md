# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1] - 2026-08-31

### Added

- **`audit.log_env`**: a new, task-level `audit:` block (`tasks.<name>.audit.log_env: [VAR, ...]`) opts specific env vars into having their actual *values* — not just names — recorded in audit events, for whichever fields you explicitly decide are worth it. Separate from `agent:` (which only affects MCP-mediated calls) since this applies to every caller — CLI, job, or MCP alike. `meriadoc validate` hard-rejects listing a `type: secret` var here.
- `AuditEvent` gained two fields: `logged_env` (the opted-in values, always present, possibly empty) and `job` (the job a task ran as part of, `null` for standalone runs).

### Fixed

- **Jobs now emit an audit event per task.** Previously `meriadoc job <name>` bypassed the audit trail entirely — including for tasks explicitly marked `risk_level: critical` — because job execution never routed through the code path that builds audit events. This was a real gap relative to the audit system's own intent (`caller: cli` already existed to record human-driven runs); it's now closed for both real runs and dry-runs.
- Removed non-functional `agent:` blocks from job- and shell-level examples (`examples/jobs.yaml`, `examples/complete-project.yaml`, `examples/shells.yaml`) and corrected `docs/architecture.md`'s Job/Shell schema — `JobSpec` and `ShellSpec` have no `agent` field, so these blocks were silently dropped and never did anything. Agent risk annotations only ever apply to tasks; jobs and shells are never exposed to MCP.

## [0.2.0] - 2026-08-23

### Added

- **Audit Logging**: structured NDJSON audit trail for every task execution — including dry-runs and blocked attempts — written to pluggable sinks (`file`, `stderr`). Enabled via `audit:` in the global config; off by default. Records caller identity, risk level, outcome, duration, and env override keys (never values).

### Fixed

- Global config sections (`discovery`, `cache`, `audit`) now accept partial YAML — omitting a field (e.g. `validate_on_discovery`, `spec_files`) falls back to its documented default instead of failing to parse. Previously any omitted field in a written-out section caused a hard `missing field` error, including in the config example shown in the README.
- CLI `--dry-run` now emits a `task.dry_run` audit event, matching the MCP dry-run path and the documented "every execution, including dry-runs, is audited" behavior.
- `docs/architecture.md`: corrected the "Project Root" section, which still described `tasks.yaml`/`jobs.yaml`/`shells.yaml` as root markers instead of the actual `meriadoc.yaml`/`meriadoc.yml`/`merry.yaml`/`merry.yml` spec files.

## [0.1.3] - 2026-04-03

### Fixed

- Tasks executed as part of a job now resolve their environment from the enclosing job's `env`, so a required task-level variable satisfied by the job no longer fails resolution.

## [0.1.2] - 2026-04-03

### Changed

- Validation cache moved to a central location under `~/.config/meriadoc/cache/`, with a collision-safe `<project-dir-name>-<hash8>` slug per project (8-char hex of a SHA-256 of the canonicalized root path). Cache files are never written inside project repos.
- `ConfigLoader` now normalizes `cache.dir` to an absolute path on load; any relative value (including a legacy `.meriadoc/cache`) is replaced with the absolute default.

### Fixed

- `cache clear` now removes the entire base cache directory, clearing orphaned entries left by renamed or removed projects instead of only the currently-discovered set.

## [0.1.1] - 2026-03-15

### Fixed

- Release workflow: fixed SHA256 checksum extraction for Homebrew formula (`grep -h` to suppress filename prefix)
- Release workflow: replaced deprecated `macos-13` runner with `macos-14` for `x86_64-apple-darwin` cross-compilation
- Install script: detect musl libc on Linux (Alpine) and select the correct binary variant

## [0.1.0] - 2025-02-11

### Added

- **Tasks**: Run sequential shell commands with environment variables, working directory, and preconditions
- **Jobs**: Compose multiple tasks into workflows with shared environment
- **Shells**: Start interactive shell sessions with pre-configured environments and custom prompts
- **Project Discovery**: Automatically find projects across configured directories
- **Validation**: Check spec files before execution with comprehensive error messages
- **Validation Caching**: Skip re-validation of unchanged files using SHA-256 hashing
- **Entity Resolution**: Support qualified names (`project:task`) to disambiguate between projects
- **Environment Variables**: Priority-based resolution (CLI > inline > env_files)
- **Choice Validation**: Runtime validation of `choice` type env vars against allowed options
- **Interactive Prompting**: Prompt for missing required variables in TTY mode
- **Saved Environments**: Store prompted values in `~/.config/meriadoc/env/<project>/<task>.env`
- **Variable Interpolation**: `${VAR}`, `$VAR`, and `${VAR:-default}` syntax in commands
- **Special Variables**: `${MERIADOC_PROJECT_ROOT}` and `${MERIADOC_SPEC_DIR}` automatically available
- **Dry-Run Mode**: Preview what would happen without executing (`--dry-run`)
- **Preconditions**: Check conditions before task execution with on_failure handlers
- **On-Failure Handlers**: Run cleanup commands when tasks fail
- **CLI Commands**:
  - `meriadoc run task/job/shell <name>` - Execute tasks, jobs, or shells
  - `meriadoc task <name>` / `meriadoc t <name>` - Shortcut for run task
  - `meriadoc job <name>` / `meriadoc j <name>` - Shortcut for run job
  - `meriadoc shell <name>` / `meriadoc s <name>` - Shortcut for run shell
  - `-n` / `--no-interactive` - Never prompt, fail on missing vars
  - `-i` / `--interactive` - Always prompt for variables
  - `meriadoc ls projects/tasks/jobs/shells` - List entities
  - `meriadoc info task/job/shell/project <name>` - Show detailed information
  - `meriadoc validate` - Validate all spec files
  - `meriadoc config add/rm/ls` - Manage discovery roots
  - `meriadoc cache ls/clear` - Manage validation cache
  - `meriadoc env show task/job/shell <name>` - Show environment variable requirements
  - `meriadoc doctor` - Diagnose common issues

### Spec File Format

- Support for `meriadoc.yaml`, `meriadoc.yml`, `merry.yaml`, `merry.yml`
- Version `v1` spec format with tasks, jobs, and shells sections
- Environment variable specs with type hints, defaults, options, and required flags
- Preconditions with on_failure policies
- Job-level and task-level on_failure handlers

[0.2.1]: https://github.com/meriadoc-dev/meriadoc/releases/tag/v0.2.1
[0.2.0]: https://github.com/meriadoc-dev/meriadoc/releases/tag/v0.2.0
[0.1.3]: https://github.com/meriadoc-dev/meriadoc/releases/tag/v0.1.3
[0.1.2]: https://github.com/meriadoc-dev/meriadoc/releases/tag/v0.1.2
[0.1.1]: https://github.com/meriadoc-dev/meriadoc/releases/tag/v0.1.1
[0.1.0]: https://github.com/meriadoc-dev/meriadoc/releases/tag/v0.1.0
