# Octopus

## What This Is

A distributed MPP (Massively Parallel Processing) query engine built on Apache DataFusion, designed for fast interactive OLAP queries over large datasets. Octopus uses a coordinator-worker architecture with streaming pipeline execution to deliver sub-second query response times. Unlike batch-oriented engines (Ballista/Spark), Octopus follows a streaming/Trino-style architecture where data flows through operators in a pipeline without materialization between stages.

**Target users:** Data engineers and analysts who need fast analytical queries on big data without the overhead of batch processing latency.

## Core Value

Users can run fast interactive SQL queries on large distributed datasets with Rust-level performance and memory safety.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Coordinator-Worker architecture with distributed task scheduling
- [ ] DataFusion integration for single-node query execution
- [ ] Distributed optimization rules extending DataFusion optimizer
- [ ] Data locality aware scheduling (partition-aware task distribution)
- [ ] Pipeline execution with Exchange operators (no stage materialization)
- [ ] Multi-Coordinator support with optional consensus (external service or self-hosted Raft)
- [ ] Control plane: gRPC for coordination
- [ ] Data plane: Apache Arrow Flight for high-speed data transfer
- [ ] Pluggable metadata storage (trait-based, multiple implementations)
- [ ] Fast-fail fault tolerance with optional task-level retry
- [ ] File format support: Parquet, CSV, JSON on S3/HDFS/local
- [ ] RDBMS support: PostgreSQL, MySQL (federated queries)
- [ ] Full OLAP SQL: SELECT, AGGREGATION, UDF/UDTF, window functions
- [ ] Observability: metrics + tracing + logging

### Out of Scope

- Native Kafka/incremental streaming source — batch materialization first
- Multi-language SDK (HTTP API only for v1)
- Fine-grained checkpoint-based recovery — fast-fail is sufficient
- Graph/ML workloads — pure SQL OLAP focus

## Context

**Reference architecture:** DataFusion Ballista (coordinator-worker model) but with key architectural difference — Ballista is Spark-like batch, Octopus is Trino-like streaming.

**Key differentiation from Trino:** Rust-based execution (no JVM GC pauses), memory safety, potential for lower latency on compute-intensive workloads.

**Rust ecosystem:** DataFusion is the core execution engine; will extend its optimizer with distributed rules.

**Open source:** Apache 2.0 License.

## Constraints

- **Tech stack:** Rust (required) — DataFusion ecosystem
- **Scale:** 10-100 nodes (mid-scale deployment)
- **Timeline:** Experiment project, no hard deadline but want rapid prototype
- **Compatibility:** Must integrate with DataFusion's optimizer extension points

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust over Java | Memory safety + better compute performance than JVM | — Pending |
| Pipeline over batch | Trino-style streaming for low latency | — Pending |
| DataFusion as execution engine | Mature, community-backed, Rust-native | — Pending |
| gRPC + Arrow Flight | Control plane vs data plane separation | — Pending |
| Multi-Coordinator (configurable) | HA + horizontal scale | — Pending |
| Pluggable metadata store | Flexibility for different deployment scenarios | — Pending |
| Fast-fail + task retry | Balance between simplicity and reliability | — Pending |
| Apache 2.0 | Open source, commercial-friendly | — Pending |

## Current Milestone: v1.2 测试覆盖与代码注释

**Goal:** 为每个模块增加单元测试、集成测试，并在代码中补充注释

**Target features:**
- 为 octopus-coordinator 增加单元测试和集成测试
- 为 octopus-executor 增加单元测试和集成测试
- 为 octopus-worker 增加单元测试和集成测试
- 为 octopus-common 增加单元测试和集成测试
- 在关键代码路径上补充 Rustdoc 注释

---

*Last updated: 2026-05-21 after v1.1 milestone complete*