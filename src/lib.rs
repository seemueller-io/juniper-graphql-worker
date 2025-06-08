mod context;
mod database;
mod models;
mod schema;

use crate::database::DatabasePool;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{http, routing::get, Router};
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use juniper::EmptySubscription;
use tower_service::Service;
use worker::wasm_bindgen::__rt::IntoJsResult;
use futures::StreamExt;
use worker::*;
use axum::http::header;

fn is_valid_ws_upgrade(req: &HttpRequest) -> bool {
    if let Some(upgrade_value) = req.headers().get(header::UPGRADE) {
        if upgrade_value.to_str().map(|v| v.eq_ignore_ascii_case("websocket")).unwrap_or(false) {
            if let Some(connection_value) = req.headers().get(header::CONNECTION) {
                let conn_str = connection_value.to_str().unwrap_or("").to_lowercase();
                return conn_str.contains("upgrade");
            }
        }
    }
    false
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    _env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    console_error_panic_hook::set_once();

    if is_valid_ws_upgrade(&req) {
        // Create the server / client pair Cloudflare expects.
        let pair = WebSocketPair::new()?;
        let mut server = pair.server;
        let client     = pair.client;

        // Accept the connection on the server side.
        server.accept()?;

        // Spawn an async task that handles this socket.
        wasm_bindgen_futures::spawn_local(async move {
            let mut event_stream = server.events().expect("could not open stream");
            while let Some(event) = event_stream.next().await {
                match event.expect("received error in websocket") {
                    WebsocketEvent::Message(msg) => server.send(&msg.text()).unwrap(),
                    WebsocketEvent::Close(event) => console_log!("{:?}", event),
                }
            }
        });

        // Build the response that initiates the upgrade.
        let mut cf_resp = Response::from_websocket(client)?;
        // Optional: add CORS or any other headers you want.
        cf_resp
            .headers_mut()
            .append("Access-Control-Allow-Origin", "*").expect("TODO: panic message");

        // Convert the Cloudflare `worker::Response` into the type that
        // `fetch` promises to return (`axum::http::Response<Body>`).
        let axum_resp: axum::http::Response<axum::body::Body> = cf_resp.try_into().unwrap();
        return Ok(axum_resp);
    }

    Ok(router(_env).call(req).await?.into_response())
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
