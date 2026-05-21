# Phase 2 Discussion Log

**Phase:** 2 - Test Infrastructure
**Date:** 2026-05-21
**Areas Discussed:** 4

## Area: Mock Infrastructure Approach

**Question:** How should we handle trait mocking in test infrastructure?

| Option | Description |
|--------|-------------|
| mockall #[automock] | Native async trait support in 0.13, works with #[async_trait], no additional crates |
| Manual in-memory fakes | Simpler but requires manual impl, no codegen needed |
| Hybrid | Combine both - mockall for complex traits, simple fakes for stateful components |

**Selected:** mockall #[automock] (方案1)

**Comparison provided:** User requested comparison of three approaches

---

## Area: Test Data Creation

**Question:** 测试数据（Arrow RecordBatch/Schema）创建方式？

| Option | Description |
|--------|-------------|
| TestRecordBatchFactory + Builder | TestRecordBatchFactory::new().build()，统一入口，builder 模式链式调用 |
| Inline RecordBatch::try_from_iter | 每次 new RecordBatch::try_from_iter，省去 factory 但调用分散 |
| Per-crate test_utils module | 每个模块维护自己的 test_utils，靠近使用地点但分散 |

**Selected:** TestRecordBatchFactory + Builder (Recommended)

---

## Area: Async Test Utilities

**Question:** 异步测试工具：超时处理策略？

| Option | Description |
|--------|-------------|
| Always wrap in tokio::time::timeout | 在测试代码中为 tokio::spawn 任务加 #[超时]，防止死测阻塞 CI |
| Only add timeout when needed | 信任任务会正常完成，不过度设计 |
| Use tokio::test with #[track_caller] | 使用 tracing-enabled runtime 捕获异步调试信息 |

**Selected:** Always wrap in tokio::time::timeout (Recommended)

---

## Area: Test Utilities Organization

**Question:** 测试工具组织：集中在 common 还是分散到各 crate？

| Option | Description |
|--------|-------------|
| Centralized in octopus-common::test_utils | 将 mock/builder 等工具集中在 octopus-common 的 test-utils 模块，各 crate 依赖 common::test_utils |
| Per-crate tests directory | 每个 crate 有自己的 tests 或 test-utils 模块，职责清晰但可能重复代码 |
| Dedicated octopus-test crate | 顶层有个测试助手 crate，职责分明但增加 workspace 复杂度 |

**Selected:** Centralized in octopus-common::test_utils (Recommended)

---

*Discussion completed: 2026-05-21*