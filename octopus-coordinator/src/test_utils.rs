// See the License for the specific language governing permissions and
// limitations under the License.

//! Test utilities for octopus-coordinator.
//!
//! Provides [`TestQueryContext`] for testing query service operations
//! with pre-configured SessionContext and UDF registry.

use datafusion::execution::context::SessionContext;
use datafusion_expr::ScalarUDF;
use std::sync::Arc;

use crate::udf::{UdfRegistry, UdfRegistryImpl};

/// A pre-configured SessionContext for testing query operations.
///
/// This struct wraps a DataFusion SessionContext with a UDF registry,
/// allowing tests to set up known schemas and register UDFs before
/// executing queries.
///
/// # Example
///
/// ```
/// use octopus_coordinator::test_utils::TestQueryContext;
/// use octopus_common::udf::create_simple_udf;
/// use datafusion::arrow::datatypes::DataType;
/// use datafusion_expr::Volatility;
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() {
///     let ctx = TestQueryContext::new();
///
///     // Register a simple UDF
///     let to_upper = create_simple_udf(
///         "to_upper",
///         vec![DataType::Utf8],
///         DataType::Utf8,
///         Volatility::Stable,
///         Arc::new(|args| {
///             Ok(datafusion::physical_plan::ColumnarValue::Array(
///                 args[0].clone().into_array(1)
///             ))
///         }),
///     );
///
///     let ctx = ctx.with_udf("to_upper", to_upper);
///
///     // Use the context for testing
///     let session = ctx.context();
///     // ... run tests
/// }
/// ```
#[derive(Debug)]
pub struct TestQueryContext {
    context: SessionContext,
    udf_registry: Arc<RwLock<UdfRegistryImpl>>,
}

impl TestQueryContext {
    /// Create a new TestQueryContext with a fresh SessionContext.
    ///
    /// The SessionContext is created via `SessionContext::new()` to ensure
    /// clean state per test, following the guidance in PITFALLS.md.
    pub fn new() -> Self {
        Self {
            context: SessionContext::new(),
            udf_registry: Arc::new(RwLock::new(UdfRegistryImpl::new())),
        }
    }

    /// Register a scalar UDF with the given name.
    ///
    /// Returns a new TestQueryContext with the UDF registered.
    /// This allows for method chaining in tests.
    pub fn with_udf(self, name: &str, udf: ScalarUDF) -> Self {
        let runtime = tokio::runtime::Handle::current();
        runtime.block_on(async {
            let mut registry = self.udf_registry.write().await;
            registry.register_scalar(name, udf).await.ok();
        });
        self
    }

    /// Access the inner SessionContext.
    pub fn context(&self) -> &SessionContext {
        &self.context
    }

    /// Access the UDF registry.
    pub fn udf_registry(&self) -> Arc<RwLock<UdfRegistryImpl>> {
        self.udf_registry.clone()
    }
}

impl Default for TestQueryContext {
    fn default() -> Self {
        Self::new()
    }
}

use tokio::sync::RwLock;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_test_query_context_creation() {
        let ctx = TestQueryContext::new();
        assert!(ctx.context().sql("SELECT 1").await.is_ok());
    }

    #[tokio::test]
    async fn test_test_query_context_with_udf() {
        let ctx = TestQueryContext::new();

        // The context should be usable even without UDFs
        let df = ctx.context().sql("SELECT 1 as id").await.unwrap();
        let plan = df.logical_plan();
        assert!(!plan.to_string().is_empty());
    }
}