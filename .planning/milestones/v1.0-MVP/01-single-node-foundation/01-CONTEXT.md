# Phase 1: Single-Node Foundation - Context

**Gathered:** 2026-04-22
**Status:** Ready for planning

## Phase Boundary

Users can execute SQL queries locally on Parquet/CSV/JSON files with Rust-level performance. This establishes the single-node DataFusion foundation with correct streaming patterns before distribution.

## Implementation Decisions

### Session/Config Approach

- **D-01:** Runtime-only configuration — DataFusion context configured at runtime without config files. No persistent config file needed for Phase 1.

### Runtime & Observability

- **D-02:** Multi-runtime — Separate Tokio runtime for CPU vs IO tasks. CPU-intensive work (query execution) gets `spawn_blocking` or separate runtime to avoid blocking async tasks.
- **D-03:** Both structured and pretty-printed logging — Structured format (JSON/key=value) for machine parsing, pretty print for human readability. Configurable via flag.

### CLI Interface

- **D-04:** Separate binaries — `octopus-cli` and future `octopus-server` as separate executables rather than subcommands.
- **D-05:** Both interactive REPL and single query mode — User can use CLI as interactive SQL REPL or pass SQL via `--sql` flag.
- **D-06:** Pretty table output by default — Box-drawing chars, aligned columns for result display.

### Project Structure

- **D-07:** Multi-crate workspace with 4 initial crates:
  - `octopus-common` — Shared types, error handling, traits
  - `octopus-executor` — DataFusion query execution wrapper
  - `octopus-cli` — Coordinator service + CLI client
  - `octopus-worker` — Worker service (for future distributed)
- **D-08:** Executor crate organized by concern — `session.rs`, `query.rs`, `planner.rs`, `physical.rs`
- **D-09:** CLI organized with REPL + subcommands via clap

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `.planning/ROADMAP.md` §Phase 1 — Phase goal and success criteria
- `.planning/REQUIREMENTS.md` §SQL-01~05, DATA-01~03, OBS-03 — Requirements mapped to this phase
- `.planning/PROJECT.md` — Core value and tech stack decisions (Rust + DataFusion)
- `.planning/research/STACK.md` — Technology stack with versions and rationale
- `.planning/research/PITFALLS.md` — Critical pitfalls to avoid (pipeline breakers, Tokio runtime contention)

### Architecture References

- `.planning/research/ARCHITECTURE.md` — Component boundaries and data flow
- `.planning/research/SUMMARY.md` — Build order and phase ordering rationale

## Existing Code Insights

### Reusable Assets

- None yet — greenfield project

### Established Patterns

- DataFusion single-node execution patterns (from DataFusion docs)
- Streaming operator patterns (avoid materialization between operators)

### Integration Points

- CLI connects to executor via session context
- Common crate shared between all binaries for error types and traits

## Specific Ideas

- Multi-runtime setup: CPU tasks on blocking thread pool, IO tasks on async runtime
- Structured logging with configurable pretty/JSON output via `--log-format` flag
- Separate `octopus-worker` crate even for Phase 1 to establish workspace pattern early

## Deferred Ideas

None — discussion stayed within phase scope

---

*Phase: 01-single-node-foundation*
*Context gathered: 2026-04-22*