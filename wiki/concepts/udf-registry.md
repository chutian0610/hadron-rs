---
title: UDF Registry
type: concept
tags: [UDF,scalar-functions,DataFusion,custom-functions]
sources:
  - octopus-common/src/udf_registry.rs
  - octopus-coordinator/src/query_service.rs
related:
  - "[[Coordinator]]"
  - "[[Execution]]"
---

# UDF Registry

The UDF (User-Defined Function) Registry provides a mechanism to register custom scalar functions with DataFusion, making them callable in SQL queries.

## UdfRegistry Trait

`octopus-common/src/udf_registry.rs:1`

```rust
pub trait UdfRegistry: Send + Sync {
    fn name(&self) -> &str;
    fn functions(&self) -> Vec<ScalarFunction>;
}
```

### ScalarFunction Structure

```rust
pub struct ScalarFunction {
    pub name: String,
    pub signature: Signature,
    pub return_type: ReturnType,
    pub fun: ScalarFunctionImpl,
}
```

## Integration with DataFusion

QueryService integrates with UdfRegistry to register custom functions:

`octopus-coordinator/src/query_service.rs:1`

### Registration Flow

1. UDF implementations register with the registry at startup
2. QueryService retrieves all registered functions
3. Functions are registered with DataFusion's `SessionContext`
4. SQL queries can now use the custom functions

### Example: to_upper UDF

```rust
// In UDF registry implementation
pub struct MyUdfRegistry;

impl UdfRegistry for MyUdfRegistry {
    fn name(&self) -> &str {
        "my_udfs"
    }

    fn functions(&self) -> Vec<ScalarFunction> {
        vec![
            ScalarFunction {
                name: "to_upper".to_string(),
                signature: Signature::uniform(1, vec![Utf8]),
                return_type: Arc::new(Utf8),
                fun: Arc::new(|args| {
                    // Implementation
                }),
            },
        ]
    }
}
```

### Usage in SQL

```sql
SELECT to_upper(name) FROM users;
```

## Built-in UDFs

Octopus may include built-in UDFs for common operations. See the registry implementations in `octopus-common/src/` for available functions.

## Creating Custom UDFs

### Step 1: Implement the Function

```rust
use datafusion::prelude::*;
use datafusion::arrow::datatypes::DataType;

fn my_function(args: &[ColumnarValue]) -> Result<ArrayRef, DataFusionError> {
    // Implement function logic
    Ok(result_array)
}
```

### Step 2: Create ScalarFunction

```rust
let scalar_fn = ScalarFunction {
    name: "my_function".to_string(),
    signature: Signature::exact(vec![DataType::Int64], Volatility::Immutable),
    return_type: Arc::new(DataType::Int64),
    fun: Arc::new(my_function),
};
```

### Step 3: Register with Registry

```rust
registry.register_function(scalar_fn);
```

### Step 4: Use in SQL

```sql
SELECT my_function(id) FROM table;
```

## Code References

- `octopus-common/src/udf_registry.rs:1` - UdfRegistry trait and implementations
- `octopus-coordinator/src/query_service.rs:1` - Integration with QueryService
