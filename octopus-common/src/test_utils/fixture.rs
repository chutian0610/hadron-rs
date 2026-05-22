//! TestRecordBatchFactory for creating Arrow test data.
//!
//! Provides a builder pattern for creating RecordBatch instances
//! with flexible column definitions for unit tests.

use datafusion::arrow::array::{ArrayRef, Int64Array, StringArray};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::error::ArrowError;
use datafusion::arrow::record_batch::RecordBatch;
use std::collections::HashMap;

/// Builder for creating test [`RecordBatch`] instances with flexible columns.
///
/// # Example
/// ```
/// use octopus_common::test_utils::fixture::TestRecordBatchFactory;
///
/// let batch = TestRecordBatchFactory::new()
///     .add_integer_column("id", vec![1, 2, 3])
///     .add_string_column("name", vec!["alice", "bob", "charlie"])
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone, Default)]
pub struct TestRecordBatchFactory {
    columns: HashMap<String, ArrayRef>,
}

impl TestRecordBatchFactory {
    /// Creates a new empty factory.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an integer column to the factory.
    pub fn add_integer_column(mut self, name: &str, values: Vec<i64>) -> Self {
        let array: ArrayRef = std::sync::Arc::new(Int64Array::from(values));
        self.columns.insert(name.to_string(), array);
        self
    }

    /// Adds a string column to the factory.
    pub fn add_string_column(mut self, name: &str, values: Vec<&str>) -> Self {
        let array: ArrayRef = std::sync::Arc::new(StringArray::from(values));
        self.columns.insert(name.to_string(), array);
        self
    }

    /// Adds an integer column from a slice of i64 values.
    pub fn add_integer_slice(self, name: &str, values: &[i64]) -> Self {
        self.add_integer_column(name, values.to_vec())
    }

    /// Builds the final [`RecordBatch`].
    ///
    /// Returns an error if the columns have mismatched lengths.
    pub fn build(&self) -> Result<RecordBatch, ArrowError> {
        if self.columns.is_empty() {
            return Err(ArrowError::InvalidArgumentError(
                "Cannot build empty RecordBatch".to_string(),
            ));
        }

        // Get the length from the first column
        let len = self.columns.values().next().map(|a| a.len()).unwrap_or(0);

        // Verify all columns have the same length
        for (name, array) in &self.columns {
            if array.len() != len {
                return Err(ArrowError::InvalidArgumentError(format!(
                    "Column '{}' has length {} but expected {}",
                    name,
                    array.len(),
                    len
                )));
            }
        }

        // Build schema from column names and types
        let fields: Vec<Field> = self.columns
            .iter()
            .map(|(name, array)| {
                let dtype = match array.data_type() {
                    DataType::Int64 => DataType::Int64,
                    DataType::Utf8 => DataType::Utf8,
                    DataType::Float64 => DataType::Float64,
                    DataType::Boolean => DataType::Boolean,
                    DataType::Date32 => DataType::Date32,
                    other => other.clone(),
                };
                Field::new(name, dtype, false)
            })
            .collect();

        let schema = Schema::new(fields);
        RecordBatch::try_new(std::sync::Arc::new(schema), self.columns.values().cloned().collect())
    }

    /// Creates a factory pre-configured with an "orders" table schema.
    ///
    /// Columns: order_id (i64), amount (i64), date (string)
    pub fn orders_table() -> Self {
        Self::new()
            .add_integer_column("order_id", vec![])
            .add_integer_column("amount", vec![])
            .add_string_column("date", vec![])
    }

    /// Creates a factory pre-configured with a "users" table schema.
    ///
    /// Columns: id (i64), name (string), value (i64)
    pub fn users_table() -> Self {
        Self::new()
            .add_integer_column("id", vec![])
            .add_string_column("name", vec![])
            .add_integer_column("value", vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_empty() {
        let factory = TestRecordBatchFactory::new();
        let result = factory.build();
        assert!(result.is_err());
    }

    #[test]
    fn test_factory_single_column() {
        let factory = TestRecordBatchFactory::new().add_integer_column("id", vec![1, 2, 3]);
        let batch = factory.build().unwrap();
        assert_eq!(batch.num_rows(), 3);
        assert_eq!(batch.num_columns(), 1);
    }

    #[test]
    fn test_factory_multiple_columns() {
        let factory = TestRecordBatchFactory::new()
            .add_integer_column("id", vec![1, 2, 3])
            .add_string_column("name", vec!["a", "b", "c"]);
        let batch = factory.build().unwrap();
        assert_eq!(batch.num_rows(), 3);
        assert_eq!(batch.num_columns(), 2);
    }

    #[test]
    fn test_factory_mismatched_lengths() {
        let factory = TestRecordBatchFactory::new()
            .add_integer_column("id", vec![1, 2, 3])
            .add_string_column("name", vec!["a", "b"]); // Different length
        let result = factory.build();
        assert!(result.is_err());
    }

    #[test]
    fn test_orders_table_factory() {
        let factory = TestRecordBatchFactory::orders_table()
            .add_integer_column("order_id", vec![100, 200])
            .add_integer_column("amount", vec![50, 100])
            .add_string_column("date", vec!["2024-01-01", "2024-01-02"]);
        let batch = factory.build().unwrap();
        assert_eq!(batch.num_rows(), 2);
        assert_eq!(batch.num_columns(), 3);
    }
}