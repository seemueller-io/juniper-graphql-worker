mod context;
mod database;
mod models;
mod schema;

use crate::database::DatabasePool;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{routing::get, Router};
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use juniper::EmptySubscription;
use juniper::http::playground::playground_source;
use tower_service::Service;
use worker::wasm_bindgen::__rt::IntoJsResult;
use worker::*;

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    _env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    console_error_panic_hook::set_once();
    Ok(router(_env).call(req).await?)
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

    // These are accessible inside the graphql resolvers
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
        env: Some(state.env.clone()),
    };

    let result = Ok(juniper::execute(
        graphql_req.query.as_str(),
        graphql_req.operation_name.as_deref(),
        &schema::Schema::new(
            schema::Query,
            schema::Mutation,
            EmptySubscription::<context::Context>::new(),
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
        .body(String::from(playground_source("/graphql", None)))
        .unwrap()
}
// serves a web gui to interact with the api
async fn graphiql(State(state): State<AppState>) -> impl IntoResponse {
    axum::http::Response::builder()
        .header("content-type", "text/html")
        .body(String::from(graphiql_source("/graphql", None)))
        .unwrap()
}

#[derive(Clone)]
struct AppState {
    env: Env,
}

impl AppState {
    pub fn new(env: Env) -> Self {
        Self { env }
    }
}

fn router(env: Env) -> Router {
    let app_state = AppState::new(env);

    Router::new()
        .route("/", get(homepage))
        .route("/playground", get(playground))
        .route("/graphiql", get(graphiql))
        .route("/graphql", get(graphql_server))
        .route("/graphql", post(graphql_server))
        .with_state(app_state)
}

async fn homepage() -> axum::response::Html<&'static str> {
    "<html><h1>juniper_axum/custom example</h1>\
           <div>visit <a href=\"/graphiql\">GraphiQL</a></div>\
           <div>visit <a href=\"/playground\">GraphQL Playground</a></div>\
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
