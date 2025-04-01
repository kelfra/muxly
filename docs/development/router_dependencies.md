# Router Module Dependencies

This document lists the dependencies required for the Router module, particularly for the new routing and transformation features.

## Cargo.toml Dependencies

Add the following dependencies to your `Cargo.toml` file:

```toml
[dependencies]
# Existing dependencies
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
async-trait = "0.1"
tracing = "0.1"
uuid = { version = "1.3", features = ["v4"] }

# New dependencies for routing features
jsonpath-lib = "0.3.0"  # For JSON path expressions in conditions
```

## Feature Description

### JSON Path Support

The `jsonpath-lib` crate provides support for evaluating JSONPath expressions, which are used in the routing conditions to extract and test values from the data. This allows for complex conditions like:

```
$.user.profile.country == 'US' AND $.metrics.revenue > 1000
```

### Expression Evaluation

The current implementation includes a basic expression evaluator. For more complex expressions, consider adding a dedicated expression evaluation library such as:

```toml
# Optional: For more advanced expression evaluation
eval = "0.4"
```

### Future Dependencies

As the Router module evolves, you might need additional dependencies for features like:

- **Database transformations**: SQL query generators for more complex database operations
- **Advanced aggregations**: Statistical libraries for complex data aggregation
- **Template rendering**: Templating engines for more sophisticated output formatting
- **Caching**: Caching libraries for improved performance on frequent operations 