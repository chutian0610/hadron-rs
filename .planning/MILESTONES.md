# Milestones

## v1.0 MVP (Shipped: 2026-05-13)

**Phases completed:** 5 phases, 16 plans, 9 tasks

**Key accomplishments:**

- Cargo workspace with 4 crates, OctopusError types, and ExecutorSession wrapping DataFusion
- QueryExecutor with async SQL execution supporting SELECT, aggregation, JOIN, CTE, and subqueries
- DataSourceRegistrar for Parquet/CSV/JSON files and structured logging with QueryTrace
- Coordinator Core
- Gap Closure
- Worker Service Foundation
- Arrow Flight data plane
- Exchange operators
- Task retry mechanism and metrics collection
- Change:
- Issue:
- Change:
- 1. [Rule 2 - Missing] Add tokio dev dependency for async tests

---

## v1.1 去除 JDBC 实现 (Shipped: 2026-05-21)

**Phases completed:** 1 phase, 1 plan

**Key accomplishments:**
- 删除 `octopus-jdbc` crate
- 更新 `Cargo.toml` workspace
- 清理 `octopus-cli` 保留 interactive/repl 模式
- 移除 wiki 和 CLAUDE.md 中 JDBC 描述