# Meriadoc – Architecture Documentation

## 1. Overview

**Meriadoc** is a local-first developer productivity tool designed to manage and execute ad‑hoc scripts, internal tools, and contextual development workflows. It targets both:

* **Solo developers** maintaining a personal script library
* **Teams** publishing internal tools and workflows via shared repositories

Meriadoc emphasizes:

* Portability across machines
* Explicit environments and contexts
* Interactive and non-interactive execution
* Minimal friction for onboarding

The tool is invoked via the CLI as:

```bash
meriadoc
# or
merry
```

and is callable from any directory.

---

## 2. Installation & Local Layout

### 2.1 Installation

Install via Homebrew, the one-line install script, or `cargo install meriadoc`. The binary is available as both `meriadoc` and `merry`.

---

### 2.2 Meriadoc Configuration Directory

All user-level state lives under `~/.config/meriadoc/` (XDG-compliant):

```text
~/.config/meriadoc/
├── config.yaml          # Global user configuration (discovery roots, cache, audit)
├── cache/               # Validation caches, one subdirectory per project
│   └── myproject-a1b2c3d4/
│       └── validation_cache.json
├── env/                 # Saved environment values, per project/task
│   └── myproject/
│       └── deploy.env
└── audit.log            # NDJSON audit log (when audit.enabled = true)
```

The config directory location can be overridden with the `MERIADOC_CONFIG` environment variable (point it at the config file, not the directory).

---

## 3. Configuration Model

### 3.1 Global Configuration (`config.yaml`)

The global configuration defines **how Meriadoc behaves on this machine** — where to find projects, how to cache validations, and where to write audit logs. It never describes tasks or commands.

```yaml
discovery:
  roots:
    - path: ~/projects
      enabled: true
      name: personal       # optional human-friendly name
  max_depth: 3
  validate_on_discovery: true
  spec_files:
    - meriadoc.yaml
    - meriadoc.yml
    - merry.yaml
    - merry.yml

cache:
  enabled: true
  dir: ~/.config/meriadoc/cache   # always normalized to absolute

audit:
  enabled: false
  sinks:
    - type: file
      path: ~/.config/meriadoc/audit.log
    # - type: stderr
```

Key points:

* Project directories are **external** to Meriadoc — specs live in project repos
* Meriadoc never modifies spec files automatically
* Relative paths in `cache.dir` and audit `path` are normalized to absolute on load

---

## 4. Project Model

A **project** is any directory registered in the global config that contains Meriadoc specification files.

### 4.1 Project Root

The project root is the directory containing at least one file matching
`config.discovery.spec_files` (default: `meriadoc.yaml`, `meriadoc.yml`,
`merry.yaml`, `merry.yml`).

All relative paths in specs are resolved **from this root**.

---

## 5. Execution Semantics

### 5.1 Default Working Directory Resolution

Meriadoc applies different defaults depending on what is executed.

#### Tasks & Jobs

* **Default working directory**: the directory containing the YAML file that defines the task or job
* If `workdir` is specified: resolved **relative to the project root**

#### Shells

* **Default working directory**: the directory from which `meriadoc` was invoked
* If `workdir` is specified: resolved **relative to the project root**

This distinction supports both:

* Repository-defined workflows
* Ad-hoc interactive usage

---

## 6. Specification Files

Spec files live in the project directory (`meriadoc.yaml`, `meriadoc.yml`, `merry.yaml`, or `merry.yml`). They are the source of truth for what a project can do.

---

### 6.1 Task Specification

A **Task** is the smallest execution unit.

```yaml
tasks:
  mytask:
    description?: string
    cmds: [string]
    workdir?: string            # relative to project root
    env?: { string: EnvVar }
    env_files?: [string]
    preconditions?: [Condition]
    on_failure?: FailurePolicy
    docs?: string
    agent?:
      enabled: bool             # default: true — set false to hide from agents
      risk_level: low|medium|high|critical   # default: low
      confirmation?: string     # message shown before execution
      requires_approval: bool   # default: false; true forces approval regardless of risk
```

**Semantics**:

* `cmds` execute sequentially
* Environment variables are resolved before execution
* `workdir` is relative to project root

---

### 6.2 Job Specification

A **Job** is a composition of tasks.

```yaml
jobs:
  myjob:
    description?: string
    tasks: [string]
    env?: { string: EnvVar }    # overrides task-level env
    env_files?: [string]
    on_failure?: FailurePolicy
    agent?:
      risk_level: low|medium|high|critical
      requires_approval: bool
      confirmation?: string
```

**Semantics**:

* Tasks run sequentially in the order listed
* Task names are resolved within the same project

---

### 6.3 Shell Specification

A **Shell** creates an interactive session with a resolved context.

```yaml
shells:
  dev:
    description?: string
    workdir?: string            # relative to project root; default: invocation dir
    env?: { string: EnvVar }
    env_files?: [string]
    init_cmds?: [string]        # run before handing control to the user
```

**Semantics**:

* An interactive shell is spawned
* Environment variables are injected
* `init_cmds` run before handing control to the user
* User may execute arbitrary commands until exit

---

### 6.4 Environment Variable Specification

```yaml
EnvVar:
  type: string
  default?: string
  options?: [string]
  required?: boolean
```

**Purpose**:

* Typed, user-selectable configuration
* Enables validation and safe prompting

---

### 6.5 Condition Specification

```yaml
Condition:
  cmds: [string]
  on_failure?: FailurePolicy
```

Conditions must succeed for execution to proceed.

---

### 6.6 Failure Policy

```yaml
FailurePolicy:
  continue: boolean
  cmds?: [string]
```

Controls error handling behavior.

---

## 7. Audit Logging

### 7.1 Overview

Every task execution — including dry-runs and blocked attempts — can be written to one or more **sinks** as a structured NDJSON record. Logging is disabled by default and never surprises existing users.

### 7.2 AuditEvent Schema (v1)

```json
{
  "timestamp": "2026-06-02T14:30:00.123Z",
  "schema_version": "1",
  "caller": "cli | api | mcp-stdio | mcp-http",
  "action": "task.run | task.dry_run | task.blocked",
  "task": "deploy-staging",
  "project": "myapp",
  "project_root": "/home/user/projects/myapp",
  "risk_level": "low | medium | high | critical",
  "exit_code": 0,
  "duration_ms": 1423,
  "env_override_keys": ["ENV"],
  "outcome": "success | failure | blocked | dry_run",
  "meriadoc_version": "0.1.3",
  "pid": 12345
}
```

`exit_code` and `duration_ms` are `null` for `task.blocked` and `task.dry_run`.
Environment variable **values** are never logged — only key names appear in `env_override_keys`.

### 7.3 Sink Architecture

```
AuditEvent
    │
    ▼
AuditLogger (fan-out)
    ├── FileSink   → ~/.config/meriadoc/audit.log  (NDJSON, O_APPEND)
    ├── StderrSink → stderr                         (container-friendly)
    └── (v2) OtlpSink → OpenTelemetry collector
```

The `AuditSink` trait is the extension boundary:

```rust
pub trait AuditSink: Send + Sync {
    fn emit(&self, event: &AuditEvent) -> Result<(), AuditError>;
}
```

Adding a new sink type (OTLP, webhook) requires implementing this trait and adding one match arm in `audit/builder.rs`. No other code changes.

### 7.4 Caller Identity

Each execution path sets its own `CallerKind`:

| Entry point | `caller` value |
|---|---|
| `meriadoc task <name>` | `cli` |
| `POST /api/tasks/:name/run` | `api` |
| MCP stdio (`meriadoc serve`) | `mcp-stdio` |
| MCP over HTTP (`meriadoc server`) | `mcp-http` |

### 7.5 Non-Fatal Design

Sink errors are printed to stderr but never propagate. A broken audit log never aborts task execution — the tool's primary job is running tasks, not managing logs.

### 7.6 Concurrency Safety

`FileSink` opens with `O_APPEND | O_CREAT` on each emit. POSIX guarantees write atomicity for writes ≤ `PIPE_BUF` (~4 KB) on Linux and macOS. NDJSON records for this schema are well under that limit, so concurrent processes writing to the same file produce valid, interleave-free records.

---

## 8. Architectural Principles


* **Local-first**: no daemon, no cloud dependency
* **Explicit roots**: all relative paths resolve from a project root
* **Portability**: specs never reference absolute user paths
* **Additive evolution**: schemas grow without breaking old projects
* **Separation of concerns**:

  * Specs define intent
  * Executor handles processes
  * UI orchestrates interaction

---

## 10. Future Extensions

Planned:

* **OTLP sink** — stream audit events to OpenTelemetry collectors (Datadog, Grafana, Honeycomb, etc.)
* **Webhook sink** — HTTP POST audit events to Slack or custom endpoints
* **Approval gates** — pause execution of high-risk tasks for human confirmation via webhook or CLI prompt
* **Scoped shells** — allowlist/blocklist for commands available to agents in interactive shells
* **Rate limiting** — cap how often agents can call specific tasks per hour
* **Parallel execution** — run independent tasks in a job concurrently
* **Task DAGs** — declare task dependencies instead of explicit ordering

All planned features are additive — existing spec files require no changes.

---

## 9. Summary


Meriadoc treats scripts as **products**, not one-off commands. By anchoring execution to project roots and separating global configuration from project specs, it enables teams and individuals to share reliable, portable tooling without sacrificing local control.
