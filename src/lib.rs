mod context;
mod database;
mod models;
mod schema;

use juniper_subscriptions::Coordinator;
use crate::database::DatabasePool;
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{routing::get, Router};
use futures::StreamExt;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use juniper::{DefaultScalarValue, ExecutionError, GraphQLError, SubscriptionCoordinator, Value};
use std::net::{SocketAddr, TcpListener};
use std::time::Duration;
use tower_service::Service;
use worker::wasm_bindgen::JsCast;
use worker::wasm_bindgen::__rt::IntoJsResult;
use worker::*;

#[cfg(not(feature = "local"))]
#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    _env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    console_error_panic_hook::set_once();
    Ok(router(RouterConfig { env: _env }).call(req).await?)
}

#[derive(Clone)]
struct CustomEnv {
    inner: std::collections::HashMap<String, String>,
}

impl CustomEnv {
    fn new() -> Self {
        let mut env = std::collections::HashMap::new();
        env.insert(
            "DATABASE_URL".to_string(),
            "postgres://localhost:5432/db".to_string(),
        );
        env.insert("API_KEY".to_string(), "test-key".to_string());
        Self { inner: env }
    }
}

#[cfg(feature = "local")]
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let env = CustomEnv::new();
    let state = AppState::new(env);

    #[cfg(not(test))]
    let ctx = context::Context {
        db: DatabasePool,
        env: state.env.clone(),
    };

    let router_config = RouterConfig {
        env: state.env.clone(),
    };

    let app = router(router_config);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tracing::info!("listening on {addr}");

    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("failed to run `axum::serve`: {e}"));
}

#[worker::send]
async fn graphql_server(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> impl IntoResponse {
    let body = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .unwrap();
    let graphql_req: GraphQLRequest = serde_json::from_slice(&body).unwrap();

    #[cfg(not(test))]
    let ctx = context::Context {
        // hardcoded simple api
        db: DatabasePool,
        env: state.env.clone(),
    };

    #[cfg(test)]
    let ctx = context::Context {
        // hardcoded simple api
        db: DatabasePool,
        env: Some(CustomEnv::new()),
    };

    let result: std::result::Result<(Value, Vec<ExecutionError<DefaultScalarValue>>), GraphQLError> = Ok(juniper::execute(
        graphql_req.query.as_str(),
        graphql_req.operation_name.as_deref(),
        &schema::Schema::new(
            schema::Query,
            schema::Mutation,
            schema::Subscription,
        ),
        &graphql_req.variables(),
        &ctx,
    )
    .await
    .unwrap());

    axum::Json(juniper::http::GraphQLResponse::from_result(result))
}

// serves a web gui to interact with the api
async fn playground(State(state): State<AppState>) -> impl IntoResponse {
    axum::http::Response::builder()
        .header("content-type", "text/html")
        .body(String::from(graphiql_source("/graphql", Some("/subscriptions"))))
        .unwrap()
}

#[cfg(feature = "local")]
#[derive(Clone)]
struct AppState {
    env: CustomEnv,
}

#[cfg(not(feature = "local"))]
#[derive(Clone)]
struct AppState {
    env: Env,
}

impl AppState {
    #[cfg(feature = "local")]
    pub fn new(env: CustomEnv) -> Self {
        Self { env }
    }
    #[cfg(not(feature = "local"))]
    pub fn new(env: Env) -> Self {
        Self { env }
    }
}

#[cfg(feature = "local")]
pub struct RouterConfig {
    pub env: CustomEnv,
}

#[cfg(not(feature = "local"))]
pub struct RouterConfig {
    pub env: Env,
}

// WebSocket handler for GraphQL subscriptions

async fn graphql_subscriptions(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |mut socket| async move {
        // Create a context for the subscription

        #[cfg(not(test))]
        let ctx = context::Context {
            db: DatabasePool,
            env: state.env.clone(),
        };

        #[cfg(test)]
        let ctx = context::Context {
            db: DatabasePool,
            env: Some(CustomEnv::new()),
        };


        // Create a schema
        let schema = schema::Schema::new(
            schema::Query,
            schema::Mutation,
            schema::Subscription,
        );

        // Create a coordinator for handling subscriptions
        let coordinator = Coordinator::new(schema);

        // Simple ping-pong keep-alive
        let keep_alive_interval = Duration::from_secs(15);
        let keep_alive = tokio::time::interval(keep_alive_interval);
        tokio::pin!(keep_alive);

        // Handle the WebSocket connection
        loop {
            tokio::select! {
                // Handle WebSocket messages
                Some(msg) = socket.recv() => {
                    if let Ok(msg) = msg {
                        if let axum::extract::ws::Message::Text(text) = msg {
                            // Parse the GraphQL subscription request
                            if let Ok(request) = serde_json::from_str::<juniper::http::GraphQLRequest>(&text) {
                                // Execute the subscription
                                let operation_name = request.operation_name.as_deref();
                                let variables = request.variables();

                                if let Ok(stream) = coordinator.subscribe(&request, &ctx).await {
                                    // Send subscription updates to the client
                                    let mut stream = stream;
                                    while let Some(result) = stream.next().await {
                                        let response = serde_json::to_string(&result).unwrap();
                                        if socket.send(axum::extract::ws::Message::Text(response)).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        // Client disconnected
                        break;
                    }
                }
                // Send keep-alive ping
                _ = keep_alive.tick() => {
                    if socket.send(axum::extract::ws::Message::Ping(vec![])).await.is_err() {
                        break;
                    }
                }
            }
        }
    })
}

fn router(config: RouterConfig) -> Router {
    let app_state = AppState::new(config.env);

    Router::new()
        .route("/", get(homepage))
        .route("/playground", get(playground))
        .route("/graphql", get(graphql_server))
        .route("/graphql", post(graphql_server))
        .route("/subscriptions", get(graphql_subscriptions))
        .with_state(app_state)
}

async fn homepage() -> axum::response::Html<&'static str> {
    "<html><h1>juniper_axum/custom example</h1>\
           <div>visit <a href=\"/graphiql\">GraphiQL</a></div>\
           <div>visit <a href=\"/playground\">GraphQL Playground</a></div>\
           <div>GraphQL Subscriptions are available at <code>/subscriptions</code></div>\
    </html>"
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Episode, NewHuman};
    use crate::schema::test_helpers;
    use juniper::Value;

    #[test]
    fn test_api_version() {
        // Test the api_version query using the test helper
        let version = test_helpers::test_api_version();
        assert_eq!(version, "1.0");
    }

    #[test]
    fn test_human_query() {
        // Test the human query using the test helper
        let result = test_helpers::test_human_query("1".to_string());

        assert!(result.is_ok());
        let human = result.unwrap();
        assert_eq!(human.id, "1");
        assert_eq!(human.name, "Luke Skywalker");
        assert_eq!(human.home_planet, "Tatooine");
    }

    #[test]
    fn test_create_human_mutation() {
        // Create a new human
        let new_human = NewHuman {
            name: "Han Solo".to_string(),
            appears_in: vec![Episode::NewHope, Episode::Empire, Episode::Jedi],
            home_planet: "Corellia".to_string(),
        };

        // Test the create_human mutation using the test helper
        let result = test_helpers::test_create_human_mutation(new_human);

        assert!(result.is_ok());
        let human = result.unwrap();
        assert_eq!(human.id, "generated-id");
        assert_eq!(human.name, "Han Solo");
        assert_eq!(human.home_planet, "Corellia");
    }
}
