// See the License for the specific language governing permissions and
// limitations under the License.

//! In-memory implementations of federated connector traits for testing.
//!
//! Provides mock implementations of ConnectionPool and TypeAdapter for
//! testing without requiring actual database connections.

use async_trait::async_trait;
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use std::sync::{Arc, RwLock};
use std::any::Any;

use crate::federated::{ConnectionPool, DatabaseType, PoolStats, TypeAdapter};
use crate::OctopusError;

/// In-memory connection pool for testing.
///
/// This struct simulates a connection pool with configurable behavior.
/// All connections are stored in-memory as `Box<dyn Any + Send>` objects.
///
/// # Example
///
/// ```
/// use octopus_common::test_utils::federated::InMemoryConnectionPool;
///
/// let pool = InMemoryConnectionPool::new(10);
/// // Pool starts with 0 connections
///
/// let pool_with_connections = InMemoryConnectionPool::with_connections(5);
/// // Pool pre-populated with 5 fake connections
/// ```
pub struct InMemoryConnectionPool {
    connections: Arc<RwLock<Vec<Box<dyn Any + Send>>>>,
    max_connections: usize,
    stats: RwLock<PoolStats>,
}

impl InMemoryConnectionPool {
    /// Create a new pool with the specified maximum connection limit.
    pub fn new(max_connections: usize) -> Self {
        Self {
            connections: Arc::new(RwLock::new(Vec::new())),
            max_connections,
            stats: RwLock::new(PoolStats::default()),
        }
    }

    /// Create a new pool pre-populated with n fake connections.
    pub fn with_connections(n: usize) -> Self {
        let pool = Self::new(n);
        let mut connections = pool.connections.write().unwrap();
        for i in 0..n {
            connections.push(Box::new(FakeConnection(i)));
        }
        let mut stats = pool.stats.write().unwrap();
        stats.total_connections = n;
        stats.idle_connections = n;
        drop(stats);
        pool
    }

    /// Get the current number of idle connections.
    pub fn idle_count(&self) -> usize {
        self.stats.read().unwrap().idle_connections
    }

    /// Get the current number of used connections.
    pub fn used_count(&self) -> usize {
        self.stats.read().unwrap().used_connections
    }
}

struct FakeConnection(usize);

#[async_trait]
impl ConnectionPool for InMemoryConnectionPool {
    async fn get(&self) -> crate::Result<Box<dyn Any + Send>> {
        let mut connections = self.connections.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        if let Some(conn) = connections.pop() {
            stats.idle_connections = connections.len();
            stats.used_connections += 1;
            Ok(conn)
        } else {
            stats.waiting_tasks += 1;
            Err(OctopusError::ConnectionPoolError(
                "No connections available".to_string(),
            ))
        }
    }

    async fn release(&self, conn: Box<dyn Any + Send>) -> crate::Result<()> {
        let mut connections = self.connections.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        if connections.len() < self.max_connections {
            connections.push(conn);
            stats.idle_connections = connections.len();
            stats.used_connections = stats.used_connections.saturating_sub(1);
            stats.waiting_tasks = stats.waiting_tasks.saturating_sub(1);
            Ok(())
        } else {
            Err(OctopusError::ConnectionPoolError(
                "Pool at maximum capacity".to_string(),
            ))
        }
    }

    fn stats(&self) -> PoolStats {
        self.stats.read().unwrap().clone()
    }
}

/// Mock implementation of TypeAdapter for testing.
///
/// Provides simple type mappings without requiring a real database connection.
/// Use this for unit tests that need to verify type adapter behavior.
///
/// # Example
///
/// ```
/// use octopus_common::test_utils::federated::{MockTypeAdapter, DatabaseType};
/// use datafusion::arrow::datatypes::DataType;
///
/// let adapter = MockTypeAdapter::new(DatabaseType::PostgreSQL);
/// assert_eq!(adapter.database_type(), DatabaseType::PostgreSQL);
///
/// // Query a simple type mapping
/// let arrow_type = adapter.to_arrow_type("VARCHAR").unwrap();
/// assert_eq!(arrow_type, DataType::Utf8);
/// ```
pub struct MockTypeAdapter {
    db_type: DatabaseType,
}

impl MockTypeAdapter {
    /// Create a new MockTypeAdapter for the specified database type.
    pub fn new(db_type: DatabaseType) -> Self {
        Self { db_type }
    }

    /// Build a mock schema for testing purposes.
    ///
    /// # Arguments
    ///
    /// * `table_name` - Name of the table
    /// * `columns` - Vector of (column_name, sql_type) tuples
    ///
    /// # Example
    ///
    /// ```
    /// use octopus_common::test_utils::federated::MockTypeAdapter;
    ///
    /// let schema = MockTypeAdapter::mock_schema(
    ///     "users",
    ///     vec![
    ///         ("id", "INTEGER"),
    ///         ("name", "VARCHAR"),
    ///         ("balance", "DECIMAL"),
    ///     ],
    /// );
    /// ```
    pub fn mock_schema(table_name: &str, columns: Vec<(&str, &str)>) -> Schema {
        let fields: Vec<Field> = columns
            .into_iter()
            .map(|(name, sql_type)| {
                let data_type = match sql_type.to_uppercase().as_str() {
                    "INTEGER" | "INT" | "BIGINT" => DataType::Int64,
                    "VARCHAR" | "TEXT" | "CHAR" | "STRING" => DataType::Utf8,
                    "DECIMAL" | "NUMERIC" | "FLOAT" | "DOUBLE" => DataType::Float64,
                    "BOOLEAN" | "BOOL" => DataType::Boolean,
                    "DATE" => DataType::Date32,
                    "TIMESTAMP" => DataType::Timestamp(datafusion::arrow::datatypes::TimeUnit::Microsecond, None),
                    _ => DataType::Utf8,
                };
                Field::new(name, data_type, true)
            })
            .collect();

        Schema::new(fields)
    }
}

#[async_trait]
impl TypeAdapter for MockTypeAdapter {
    fn database_type(&self) -> DatabaseType {
        self.db_type
    }

    fn to_arrow_type(&self, sql_type: &str) -> crate::Result<DataType> {
        let dt = match sql_type.to_uppercase().as_str() {
            "INTEGER" | "INT" | "BIGINT" | "SERIAL" => DataType::Int64,
            "VARCHAR" | "TEXT" | "CHAR" | "STRING" | "NVARCHAR" => DataType::Utf8,
            "DECIMAL" | "NUMERIC" | "REAL" | "FLOAT" | "DOUBLE" => DataType::Float64,
            "BOOLEAN" | "BOOL" => DataType::Boolean,
            "DATE" => DataType::Date32,
            "TIME" => DataType::Time64(datafusion::arrow::datatypes::TimeUnit::Millisecond),
            "TIMESTAMP" | "DATETIME" => {
                DataType::Timestamp(datafusion::arrow::datatypes::TimeUnit::Microsecond, None)
            }
            "BLOB" | "BYTEA" | "BINARY" | "VARBINARY" => DataType::Binary,
            _ => DataType::Utf8,
        };
        Ok(dt)
    }

    fn from_arrow_type(&self, arrow_type: &DataType) -> crate::Result<String> {
        let sql_type = match arrow_type {
            DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 => "BIGINT",
            DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 => "BIGINT",
            DataType::Float32 | DataType::Float64 => "DOUBLE",
            DataType::Utf8 | DataType::LargeUtf8 => "VARCHAR",
            DataType::Boolean => "BOOLEAN",
            DataType::Date32 | DataType::Date64 => "DATE",
            DataType::Time32(_) | DataType::Time64(_) => "TIME",
            DataType::Timestamp(_, None) => "TIMESTAMP",
            DataType::Binary | DataType::LargeBinary => "VARBINARY",
            _ => "VARCHAR",
        };
        Ok(sql_type.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_connection_pool_get_release() {
        let pool = InMemoryConnectionPool::new(5);
        assert_eq!(pool.idle_count(), 0);

        // Get a connection
        let conn = pool.get().await;
        assert!(conn.is_ok());

        // Release it back
        let result = pool.release(conn.unwrap()).await;
        assert!(result.is_ok());
        assert_eq!(pool.idle_count(), 1);
    }

    #[tokio::test]
    async fn test_pool_exhaustion() {
        let pool = InMemoryConnectionPool::new(1);

        // Get the only connection
        let conn1 = pool.get().await.unwrap();
        assert!(pool.get().await.is_err()); // No more connections

        // Release and get again
        pool.release(conn1).await.unwrap();
        let conn2 = pool.get().await;
        assert!(conn2.is_ok());
    }

    #[test]
    fn test_mock_type_adapter_to_arrow() {
        let adapter = MockTypeAdapter::new(DatabaseType::PostgreSQL);

        assert_eq!(adapter.to_arrow_type("VARCHAR").unwrap(), DataType::Utf8);
        assert_eq!(adapter.to_arrow_type("INTEGER").unwrap(), DataType::Int64);
        assert_eq!(adapter.to_arrow_type("DECIMAL").unwrap(), DataType::Float64);
    }

    #[test]
    fn test_mock_schema_builder() {
        let schema = MockTypeAdapter::mock_schema(
            "orders",
            vec![
                ("order_id", "INTEGER"),
                ("customer_name", "VARCHAR"),
                ("amount", "DECIMAL"),
            ],
        );

        assert_eq!(schema.fields.len(), 3);
        assert_eq!(schema.field("order_id").unwrap().data_type(), &DataType::Int64);
        assert_eq!(schema.field("customer_name").unwrap().data_type(), &DataType::Utf8);
        assert_eq!(schema.field("amount").unwrap().data_type(), &DataType::Float64);
    }
}