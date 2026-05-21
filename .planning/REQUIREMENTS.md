# Requirements: Octopus v1.1

**Defined:** 2026-05-21
**Milestone:** 去除 JDBC 实现
**Core Value:** Users can run fast interactive SQL queries on large distributed datasets with Rust-level performance and memory safety.

## Cleanup Requirements

### JDBC and CLI Cleanup

- [ ] **CLEANUP-01**: 删除 `octopus-jdbc` crate 及相关代码
- [ ] **CLEANUP-02**: 更新 `Cargo.toml` workspace 移除 jdbc member
- [ ] **CLEANUP-03**: 清理 `octopus-cli` 仅保留 interactive/repl 模式（移除 batch 和 local 模式）
- [ ] **CLEANUP-04**: 更新 wiki 中关于 JDBC 的描述
- [ ] **CLEANUP-05**: 更新 CLAUDE.md 中关于 JDBC driver 的描述

## Out of Scope

| Feature | Reason |
|---------|--------|
| JDBC 客户端支持 | 用户反馈实现有问题，Octopus 仅通过 HTTP API 提交查询 |
| CLI batch 模式 | 用户反馈不需要，保留 interactive/repl 模式即可 |
| CLI local 模式 | 用户反馈不需要，保留 interactive/repl 模式即可 |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CLEANUP-01 | — | — |
| CLEANUP-02 | — | — |
| CLEANUP-03 | — | — |
| CLEANUP-04 | — | — |
| CLEANUP-05 | — | — |

---
*Requirements defined: 2026-05-21*
*Last updated: 2026-05-21 during milestone v1.1 start*