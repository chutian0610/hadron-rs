//! Mock implementations for testing Octopus components.
//!
//! Provides mockable substitutes for real registry and service types.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Partition information for data locality tracking.
/// Mirrors [`crate::worker_registry::PartitionInfo`] for test isolation.
#[derive(Debug, Clone)]
pub struct PartitionInfo {
    pub partition_id: String,
    pub table_name: String,
    pub file_path: String,
}

/// Worker information for registry operations.
/// Mirrors [`crate::worker_registry::WorkerInfo`] for test isolation.
#[derive(Debug, Clone)]
pub struct WorkerInfo {
    pub worker_id: String,
    pub host: String,
    pub port: u16,
    pub slots: u32,
    pub registered_at: std::time::Instant,
    pub last_heartbeat: std::time::Instant,
    pub partitions: Vec<PartitionInfo>,
}

/// MockWorkerRegistry provides a controllable worker list for scheduler testing.
///
/// Unlike the real [`crate::worker_registry::WorkerRegistry`], this mock
/// allows tests to:
/// - Pre-set worker lists for deterministic behavior
/// - Inspect worker state after operations
/// - Control registration outcomes
pub struct MockWorkerRegistry {
    workers: Arc<RwLock<HashMap<String, WorkerInfo>>>,
    registration_counter: AtomicUsize,
}

impl MockWorkerRegistry {
    /// Creates a new empty mock registry.
    pub fn new() -> Self {
        Self {
            workers: Arc::new(RwLock::new(HashMap::new())),
            registration_counter: AtomicUsize::new(0),
        }
    }

    /// Registers a new worker with the given host/port/slots.
    ///
    /// Returns a predictable UUID based on an internal counter,
    /// making test assertions deterministic.
    pub async fn register(&self, host: String, port: u16, slots: u32) -> String {
        let counter = self.registration_counter.fetch_add(1, Ordering::SeqCst);
        let worker_id = format!("mock-worker-{:08x}", counter);
        let info = WorkerInfo {
            worker_id: worker_id.clone(),
            host,
            port,
            slots,
            registered_at: std::time::Instant::now(),
            last_heartbeat: std::time::Instant::now(),
            partitions: Vec::new(),
        };
        self.workers.write().await.insert(worker_id.clone(), info);
        worker_id
    }

    /// Adds a worker to the registry with a specific ID.
    pub async fn add_worker(&self, info: WorkerInfo) {
        self.workers.write().await.insert(info.worker_id.clone(), info);
    }

    /// Retrieves a worker by ID, if present.
    pub async fn get_worker(&self, worker_id: &str) -> Option<WorkerInfo> {
        self.workers.read().await.get(worker_id).cloned()
    }

    /// Lists all registered workers.
    pub async fn list_workers(&self) -> Vec<WorkerInfo> {
        self.workers.read().await.values().cloned().collect()
    }

    /// Updates partition information for a worker.
    pub async fn update_partition(
        &self,
        worker_id: &str,
        partition_id: String,
        table_name: String,
        file_path: String,
    ) -> bool {
        if let Some(info) = self.workers.write().await.get_mut(worker_id) {
            info.partitions.push(PartitionInfo {
                partition_id,
                table_name,
                file_path,
            });
            true
        } else {
            false
        }
    }

    /// Returns the current number of workers in the registry.
    pub async fn worker_count(&self) -> usize {
        self.workers.read().await.len()
    }
}

impl Default for MockWorkerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_registry_empty() {
        let registry = MockWorkerRegistry::new();
        assert_eq!(registry.worker_count().await, 0);
    }

    #[tokio::test]
    async fn test_mock_registry_register() {
        let registry = MockWorkerRegistry::new();
        let id = registry.register("localhost".to_string(), 8080, 4).await;
        assert_eq!(id, "mock-worker-00000000");
        assert_eq!(registry.worker_count().await, 1);
    }

    #[tokio::test]
    async fn test_mock_registry_add_worker() {
        let registry = MockWorkerRegistry::new();
        let worker = WorkerInfo {
            worker_id: "test-worker".to_string(),
            host: "localhost".to_string(),
            port: 8080,
            slots: 4,
            registered_at: std::time::Instant::now(),
            last_heartbeat: std::time::Instant::now(),
            partitions: vec![],
        };
        registry.add_worker(worker).await;
        assert_eq!(registry.worker_count().await, 1);
        let workers = registry.list_workers().await;
        assert_eq!(workers.len(), 1);
        assert_eq!(workers[0].worker_id, "test-worker");
    }

    #[tokio::test]
    async fn test_mock_registry_update_partition() {
        let registry = MockWorkerRegistry::new();
        let worker = WorkerInfo {
            worker_id: "w1".to_string(),
            host: "localhost".to_string(),
            port: 8080,
            slots: 4,
            registered_at: std::time::Instant::now(),
            last_heartbeat: std::time::Instant::now(),
            partitions: vec![],
        };
        registry.add_worker(worker).await;

        let result = registry
            .update_partition(
                "w1",
                "p1".to_string(),
                "orders".to_string(),
                "/data/orders/p1.parquet".to_string(),
            )
            .await;
        assert!(result);

        let worker = registry.get_worker("w1").await.unwrap();
        assert_eq!(worker.partitions.len(), 1);
        assert_eq!(worker.partitions[0].partition_id, "p1");
    }
}