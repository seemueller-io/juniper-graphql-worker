# Juniper GraphQL Worker Test Implementation Guide

This guide provides concrete examples and implementation details for the tests outlined in the [Test Plan](./test_plan.md). It aims to help developers implement effective tests for the Juniper GraphQL Worker application.

## Setting Up the Test Environment

### Required Dependencies

Add the following dependencies to your `Cargo.toml` for testing:

```toml
[dev-dependencies]
tokio-test = "0.4"
mockall = "0.11"
async-trait = "0.1"
serde_json = "1.0"
```

### Test Directory Structure

Organize your tests following this structure:

```
tests/
├── unit/
│   ├── schema_tests.rs
│   ├── model_tests.rs
│   ├── database_tests.rs
│   └── context_tests.rs
├── integration/
│   ├── api_tests.rs
│   └── workflow_tests.rs
└── common/
    └── test_utils.rs
```

## Unit Test Examples

### GraphQL Schema Tests

```rust
// tests/unit/schema_tests.rs
use juniper::{EmptySubscription, RootNode};
use juniper_graphql_worker::{context, models, schema};

#[tokio::test]
async fn test_api_version() {
    // Create a test schema
    let schema = RootNode::new(
        schema::Query,
        schema::Mutation,
        EmptySubscription::<context::Context>::new(),
    );

    // Create a test context
    let ctx = create_test_context();

    // Execute the query
    let query = "{ apiVersion }";
    let result = juniper::execute(query, None, &schema, &juniper::Variables::new(), &ctx)
        .await
        .unwrap();

    // Assert the result
    let data = result.data.as_object().unwrap();
    assert_eq!(data["apiVersion"], "1.0");
}

#[tokio::test]
async fn test_human_query() {
    // Create a test schema
    let schema = RootNode::new(
        schema::Query,
        schema::Mutation,
        EmptySubscription::<context::Context>::new(),
    );

    // Create a test context with mock database
    let ctx = create_test_context_with_mock_db();

    // Execute the query
    let query = r#"
        {
            human(id: "1") {
                id
                name
                appearsIn
                homePlanet
            }
        }
    "#;
    
    let result = juniper::execute(query, None, &schema, &juniper::Variables::new(), &ctx)
        .await
        .unwrap();

    // Assert the result
    let data = result.data.as_object().unwrap();
    let human = data["human"].as_object().unwrap();
    assert_eq!(human["id"], "1");
    assert_eq!(human["name"], "Luke Skywalker");
    // Additional assertions...
}

// Helper functions
fn create_test_context() -> context::Context {
    // Create a test context with mock dependencies
    // ...
}
```

### Model Tests

```rust
// tests/unit/model_tests.rs
use juniper_graphql_worker::models::{Episode, Human, NewHuman};

#[test]
fn test_human_creation() {
    let human = Human {
        id: "1".to_string(),
        name: "Luke Skywalker".to_string(),
        appears_in: vec![Episode::NewHope, Episode::Empire, Episode::Jedi],
        home_planet: "Tatooine".to_string(),
    };

    assert_eq!(human.id, "1");
    assert_eq!(human.name, "Luke Skywalker");
    assert_eq!(human.appears_in.len(), 3);
    assert_eq!(human.home_planet, "Tatooine");
}

#[test]
fn test_episode_enum() {
    let episodes = vec![Episode::NewHope, Episode::Empire, Episode::Jedi];
    
    assert_eq!(episodes.len(), 3);
    // Test enum serialization/deserialization if needed
}
```

### Database Tests

```rust
// tests/unit/database_tests.rs
use juniper_graphql_worker::database::{DatabaseConnection, DatabasePool};
use juniper_graphql_worker::models::{Episode, NewHuman};

#[test]
fn test_database_connection() {
    let pool = DatabasePool;
    let conn_result = pool.get_connection();
    
    assert!(conn_result.is_ok());
}

#[test]
fn test_find_human() {
    let conn = DatabaseConnection;
    let human_result = conn.find_human("1");
    
    assert!(human_result.is_ok());
    let human = human_result.unwrap();
    assert_eq!(human.id, "1");
    assert_eq!(human.name, "Luke Skywalker");
}

#[test]
fn test_insert_human() {
    let conn = DatabaseConnection;
    let new_human = NewHuman {
        name: "Han Solo".to_string(),
        appears_in: vec![Episode::NewHope, Episode::Empire, Episode::Jedi],
        home_planet: "Corellia".to_string(),
    };
    
    let result = conn.insert_human(&new_human);
    assert!(result.is_ok());
    let human = result.unwrap();
    assert_eq!(human.name, "Han Solo");
}
```

## Integration Test Examples

### GraphQL API Tests

```rust
// tests/integration/api_tests.rs
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use juniper_graphql_worker::router;

#[tokio::test]
async fn test_graphql_endpoint() {
    // Create a test app
    let app = router(create_test_env());

    // Create a GraphQL query request
    let query = r#"{"query": "{ apiVersion }"}"#;
    let request = Request::builder()
        .uri("/graphql")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(query))
        .unwrap();

    // Execute the request
    let response = app.oneshot(request).await.unwrap();
    
    // Assert the response
    assert_eq!(response.status(), StatusCode::OK);
    
    // Parse and verify the response body
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(json.get("data").is_some());
    assert_eq!(json["data"]["apiVersion"], "1.0");
}

#[tokio::test]
async fn test_graphql_playground() {
    // Create a test app
    let app = router(create_test_env());

    // Create a request to the playground endpoint
    let request = Request::builder()
        .uri("/playground")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    // Execute the request
    let response = app.oneshot(request).await.unwrap();
    
    // Assert the response
    assert_eq!(response.status(), StatusCode::OK);
    
    // Verify the content type is HTML
    let headers = response.headers();
    assert_eq!(headers["content-type"], "text/html");
}

// Helper function to create a test environment
fn create_test_env() -> worker::Env {
    // Create a mock Env for testing
    // ...
}
```

### End-to-End Workflow Tests

```rust
// tests/integration/workflow_tests.rs
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use juniper_graphql_worker::router;

#[tokio::test]
async fn test_create_and_query_human() {
    // Create a test app
    let app = router(create_test_env());

    // Step 1: Create a new human
    let mutation = r#"{
        "query": "mutation { createHuman(newHuman: { name: \"Han Solo\", appearsIn: [NEW_HOPE, EMPIRE, JEDI], homePlanet: \"Corellia\" }) { id name } }"
    }"#;
    
    let request = Request::builder()
        .uri("/graphql")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(mutation))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    // Extract the ID of the created human
    let id = json["data"]["createHuman"]["id"].as_str().unwrap();
    
    // Step 2: Query for the created human
    let query = format!(
        r#"{{ "query": "{{ human(id: \"{}\") {{ id name appearsIn homePlanet }} }}" }}"#,
        id
    );
    
    let request = Request::builder()
        .uri("/graphql")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(query))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    // Verify the queried human matches what we created
    assert_eq!(json["data"]["human"]["id"], id);
    assert_eq!(json["data"]["human"]["name"], "Han Solo");
    assert_eq!(json["data"]["human"]["homePlanet"], "Corellia");
}
```

## Performance Test Examples

### Load Testing

For load testing, you can use tools like `wrk` or `k6`. Here's an example script for k6:

```javascript
// tests/performance/load_test.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  vus: 10,
  duration: '30s',
};

export default function () {
  const url = 'https://your-worker-url.workers.dev/graphql';
  const payload = JSON.stringify({
    query: '{ apiVersion }',
  });

  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
  };

  const res = http.post(url, payload, params);
  
  check(res, {
    'status is 200': (r) => r.status === 200,
    'response has data': (r) => JSON.parse(r.body).data !== undefined,
  });

  sleep(1);
}
```

### Cold Start Testing

```rust
// tests/performance/cold_start_test.rs
use std::time::{Duration, Instant};
use juniper_graphql_worker::router;

#[tokio::test]
async fn test_cold_start_performance() {
    // Measure the time it takes to initialize the router
    let start = Instant::now();
    let _app = router(create_test_env());
    let initialization_time = start.elapsed();
    
    println!("Router initialization time: {:?}", initialization_time);
    
    // You can set thresholds based on your requirements
    assert!(initialization_time < Duration::from_millis(100));
}
```

## Continuous Integration Setup

Here's an example GitHub Actions workflow for running tests:

```yaml
# .github/workflows/test.yml
name: Test

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true
          
      - name: Build
        uses: actions-rs/cargo@v1
        with:
          command: build
          
      - name: Run tests
        uses: actions-rs/cargo@v1
        with:
          command: test
          
      - name: Generate coverage report
        uses: actions-rs/cargo@v1
        with:
          command: tarpaulin
          args: --out Xml
          
      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v1
```

## Best Practices

1. **Mock External Dependencies**: Use mockall to create mock implementations of external dependencies.
2. **Test Edge Cases**: Include tests for error conditions and edge cases.
3. **Keep Tests Fast**: Optimize tests to run quickly to encourage frequent testing.
4. **Isolate Tests**: Ensure tests don't depend on each other or external state.
5. **Use Test Fixtures**: Create reusable test fixtures for common test scenarios.
6. **Continuous Testing**: Run tests automatically on every commit.
7. **Monitor Test Coverage**: Track test coverage to identify untested code.

## Conclusion

This guide provides a starting point for implementing tests for the Juniper GraphQL Worker application. Adapt these examples to your specific needs and expand them as the application evolves.