---
title: Crates
type: reference
tags: [crates,dependencies,workspace]
sources:
  - Cargo.toml
  - octopus-*/
related:
  - "[[Architecture]]"
  - "[[Overview]]"
---

# Crates

Octopus is organized as a Rust workspace with multiple crates.

## Workspace Structure

```toml
# Cargo.toml
[workspace]
members = [
    "octopus-coordinator",
    "octopus-executor",
    "octopus-worker",
    "octopus-cli",
    "octopus-jdbc",
    "octopus-common",
    "proto",
]
```

## Crate Overview

### octopus-coordinator

Central brain of the distributed query engine.

**Key dependencies:**
- `axum` - HTTP server
- `datafusion` - SQL parsing and execution
- `tokio` - Async runtime
- `tonic` - gRPC framework
- `tracing` - Structured logging

**Source:** `octopus-coordinator/`

### octopus-executor

Local query execution using DataFusion.

**Key dependencies:**
- `datafusion` - Core query execution
- `tokio` - Async runtime
- `arrow` - Arrow format support
- `parquet` - Parquet file support

**Source:** `octopus-executor/`

### octopus-worker

Worker process for distributed execution.

**Key dependencies:**
- `arrow-flight` - Flight gRPC for data transfer
- `tonic` - gRPC framework
- `tokio` - Async runtime
- `metrics` - Observability

**Source:** `octopus-worker/`

### octopus-cli

Client CLI with local/interactive/batch modes.

**Key dependencies:**
- `reqwest` - HTTP client
- `tokio` - Async runtime
- `rustyline` - REPL support

**Source:** `octopus-cli/`

### octopus-jdbc

JDBC Type 4 driver implementation.

**Key dependencies:**
- `jni` - Java Native Interface
- `arrow` - Arrow format for data transfer

**Source:** `octopus-jdbc/`

### octopus-common

Shared utilities and traits.

**Key dependencies:**
- `thiserror` - Error handling
- `async-trait` - Async trait methods
- `deadpool-postgres` - PostgreSQL connection pool
- `mysql_async` - MySQL async driver

**Source:** `octopus-common/`

### proto

gRPC protocol definitions.

**Source:** `proto/`

## DataFusion Version

Currently using **DataFusion 43** (API-stable).

All SQL execution uses `SessionContext::sql()` which parses SQL into LogicalPlan via DataFusion's built-in SQL planner.

## Internal Dependencies

```
octopus-common (no dependencies on other Octopus crates)
        ↑
        │
octopus-coordinator ──────→ octopus-common
octopus-executor ────────→ octopus-common
octopus-worker ──────────→ octopus-common, octopus-executor
octopus-cli ─────────────→ octopus-common
octopus-jdbc ────────────→ octopus-common
```

## External Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `datafusion` | 43 | SQL parsing and execution |
| `arrow` | latest | Columnar format |
| `parquet` | latest | Parquet file support |
| `axum` | latest | HTTP framework |
| `tonic` | latest | gRPC framework |
| `tokio` | 1.x | Async runtime |
