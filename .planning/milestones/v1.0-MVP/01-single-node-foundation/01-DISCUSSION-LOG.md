# Phase 1: Single-Node Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-22
**Phase:** 01-single-node-foundation
**Areas discussed:** Session/config approach, Runtime & observability, CLI interface design, Project structure

---

## Session/config approach

| Option | Description | Selected |
|--------|-------------|----------|
| Runtime-only | DataFusion context configured at runtime, no config file | ✓ |
| Config file | YAML/TOML config file on startup | |
| All of the above | CLI args + env vars + config file | |

**User's choice:** Runtime-only (Recommended)
**Notes:** User prefers runtime configuration for flexibility

---

## Runtime & observability (multi-question)

### Tokio runtime configuration

| Option | Description | Selected |
|--------|-------------|----------|
| Single runtime | Single Tokio runtime for simplicity, matches DataFusion patterns | |
| Multi-runtime | Separate runtime for CPU vs IO tasks, more isolation but more complex | ✓ |
| Per-session runtime | Custom runtime per query session | |

**User's choice:** Multi-runtime

### Logging/tracing approach

| Option | Description | Selected |
|--------|-------------|----------|
| Structured | Log with structured fields (JSON or key=value), for machine parsing | |
| Pretty print | Human-readable log format with colors | |
| Both | Both formats, configurable via flag | ✓ |

**User's choice:** Both (structured + pretty), configurable

---

## CLI interface design (multi-question)

### CLI interface: subcommands vs separate binaries

| Option | Description | Selected |
|--------|-------------|----------|
| Subcommands | Single octopus command with subcommands | |
| Separate binaries | Separate executables: octopus-cli, octopus-server | ✓ |
| Mode flag | One binary with --mode flag | |

**User's choice:** Separate binaries

### CLI interaction mode

| Option | Description | Selected |
|--------|-------------|----------|
| Interactive REPL | User types SQL, gets streamed results | |
| Single query mode | octopus-cli --sql 'SELECT * FROM t' | |
| Both | Both interactive and single query | ✓ |

**User's choice:** Both interactive + single query

### Result format

| Option | Description | Selected |
|--------|-------------|----------|
| Pretty table | Box-drawing chars, aligned columns | ✓ |
| JSON | Structured JSON output | |
| CSV | Simple CSV format | |
| All formats | All three formats, user picks | |

**User's choice:** Pretty table by default

---

## Project structure (multi-question)

### Project structure approach

| Option | Description | Selected |
|--------|-------------|----------|
| Single workspace crate | All in one crate for simplicity | |
| Two crates | Core + CLI separate, CLI depends on core | |
| Multiple crates | coordinator, worker, cli, common separate | ✓ |

**User's choice:** Multiple crates

### Which crates for the initial structure (multi-select)

| Option | Description | Selected |
|--------|-------------|----------|
| octopus-common | Shared types, error handling, traits | ✓ |
| octopus-executor | DataFusion query execution wrapper | ✓ |
| octopus-cli | Coordinator service + CLI client | ✓ |
| octopus-worker | Worker service (for future distributed) | ✓ |

**User's choice:** octopus-common, octopus-executor, octopus-cli, octopus-worker

### Executor crate organization

| Option | Description | Selected |
|--------|-------------|----------|
| By concern | session.rs, query.rs, planner.rs, physical.rs | ✓ |
| By domain | sql/ dir, data/ dir, exec/ dir | |
| Flat | Keep it flat, not too many nested dirs | |

**User's choice:** By concern

### CLI internal organization

| Option | Description | Selected |
|--------|-------------|----------|
| REPL + subcommands | cli.rs with subcommands via clap, repl.rs for REPL | ✓ |
| Single command mode | commands like postgres (psql) | |
| Interactive REPL only | Interactive mode only | |

**User's choice:** REPL + subcommands via clap

---

## Phase 1 Context Ready

User confirmed: "Ready to create context"

---

*Phase: 01-single-node-foundation*
*Discussion completed: 2026-04-22*