# Roadmap: Octopus

## Milestones

- ✅ **v1.0 MVP** — Phases 1-5 (shipped 2026-05-13)
- 🚧 **v1.1** — Planned

## Progress

| Phase | Milestone | Plans | Status | Completed |
|-------|-----------|-------|--------|-----------|
| 1 | v1.0 | 3/3 | Complete | 2026-05-07 |
| 2 | v1.0 | 3/3 | Complete | 2026-05-11 |
| 3 | v1.0 | 4/4 | Complete | 2026-05-12 |
| 4 | v1.0 | 4/4 | Complete | 2026-05-12 |
| 5 | v1.0 | 2/2 | Complete | 2026-05-13 |

---

*v1.0 details archived to `.planning/milestones/v1.0-MVP.md`*

## v1.1 去除 JDBC 实现

### Phase 1: 清理 JDBC 和 CLI 模式

**Goal:** 移除 octopus-jdbc 模块和 CLI 中不需要的模式

**Requirements:**
- [CLEANUP-01] 删除 `octopus-jdbc` crate 及相关代码
- [CLEANUP-02] 更新 `Cargo.toml` workspace 移除 jdbc member
- [CLEANUP-03] 清理 `octopus-cli` 仅保留 interactive/repl 模式
- [CLEANUP-04] 更新 wiki 中关于 JDBC 的描述
- [CLEANUP-05] 更新 CLAUDE.md 中关于 JDBC driver 的描述

**Success criteria:**
1. `octopus-jdbc` 目录已删除
2. `Cargo.toml` workspace members 不包含 octopus-jdbc
3. `octopus-cli` 仅支持 interactive/repl 模式
4. wiki 文档中无 JDBC 相关描述
5. CLAUDE.md 中无 JDBC driver 描述

**Mode:** cleanup