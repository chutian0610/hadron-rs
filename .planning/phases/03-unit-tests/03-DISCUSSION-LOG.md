# Phase 03 Discussion Log

**Date:** 2026-05-22
**Phase:** 03 - Unit Tests
**Mode:** default (interactive)

## Gray Areas Discussed

### 1. Synthetic fixtures

**Question:** "For relational test data (orders, users, etc.), how should fixture data be defined?"
**Options presented:**
1. Minimal constants — Define small, fixed test datasets as constants
2. Generative factories — Builder-pattern factories generating arbitrary test data
3. External fixture files — Readable fixture files (YAML/JSON) loaded at test runtime

**Recommendation:** Option 1 (Minimal constants)
**Rationale:** Unit tests prioritize readability and traceability. Fixed constants let anyone
reading the test immediately understand what data is being verified. Generative factories are
used for boundary/variant data generation only.

**User decision:** Accepted recommendation — Minimal constants for relational test data

---

### 2. Realistic mocks

**Question:** "对于 worker 和外部依赖，你希望如何处理 mock？"
**Options presented:**
1. Mock workers/Flight only — Worker registry, Flight client 用 mockall mock
2. Mock all external deps — MockAll 覆盖 worker registry + database connectors
3. Mock minimal — 用真实的 worker registry (in-memory HashMap) 和 mock Flight

**Recommendation:** Option 1 (Mock workers/Flight only)
**Rationale:** Phase 3 focuses on scheduler logic (partition locality scoring, round-robin
fallback). MockWorkerRegistry from Phase 2 gives fine-grained control over worker state.
Real SessionContext provides complete SQL parsing without external dependencies.

**User decision:** Accepted recommendation — Mock workers/Flight only

---

### 3. Maximal real

**Question:** "对于 query execution 的测试，哪些部分应该用真实实现？"
**Options presented:**
1. Real DataFusion + in-memory — 用真实 SessionContext 解析和执行 SQL，数据存在内存
2. Real DataFusion + Parquet files — 真实 SessionContext + 真实文件系统上的 Parquet 文件
3. All real with tempfile — 所有组件用真实实现，但用 tempfile + in-memory pool

**Recommendation:** Option 1 (Real DataFusion + in-memory)
**Rationale:** In-memory DataFusion with synthetic fixtures provides complete query execution
capability without I/O overhead or external dependencies. Highest confidence in core algorithm
correctness while keeping tests fast and reliable.

**User decision:** Accepted recommendation — Real DataFusion + in-memory

---

## Summary

All three gray areas discussed with user accepting the recommended approach for each:
- **Fixtures:** Minimal constants (with TestRecordBatchFactory for boundary cases)
- **Mocks:** Mock workers/Flight only; real DataFusion
- **Real components:** Real DataFusion + in-memory; no external files or DBs

## Next Step

Ready for `/gsd-plan-phase 3` to create detailed implementation plans.