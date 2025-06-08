Subscriptions
GraphQL subscriptions are a way to push data from a server to clients requesting real-time messages from a server. Subscriptions are similar to queries in that they specify a set of fields to be delivered to a client, but instead of immediately returning a single answer a result is sent every time a particular event happens on a server.

In order to execute subscriptions in Juniper, we need a coordinator (spawning long-lived connections) and a GraphQL object with fields resolving into a Stream of elements which will then be returned to a client. The juniper_subscriptions crate provides a default implementation of these abstractions.

The subscription root is just a GraphQL object, similar to the query root and mutations root that we define for operations in our GraphQL schema. For subscriptions all fields should be async and return a Stream of some GraphQL type values, rather than direct values.

type StringStream = Pin<Box<dyn Stream<Item = Result<String, FieldError>> + Send>>;

pub struct Subscription;

#[graphql_subscription]
#[graphql(context = Database)]
impl Subscription {
// This subscription operation emits two values sequentially:
// the `String`s "Hello" and "World!".
async fn hello_world() -> StringStream {
let stream = futures::stream::iter([
Ok(String::from("Hello")),
Ok(String::from("World!")),
]);
Box::pin(stream)
}
}
Coordinator
GraphQL subscriptions require a bit more resources than regular queries and provide a great vector for DoS attacks. This can can bring down a server easily if not handled correctly. The SubscriptionCoordinator trait provides coordination logic to enable functionality like DoS attacks mitigation and resource limits.

The SubscriptionCoordinator contains the schema and can keep track of opened connections, handle subscription start and end, and maintain a global ID for each subscription. Each time a connection is established, the SubscriptionCoordinator spawns a [32], which handles a single connection, providing resolver logic for a client stream as well as reconnection and shutdown logic.

While we can implement SubscriptionCoordinator ourselves, Juniper contains a simple and generic implementation called Coordinator. The subscribe method returns a Future resolving into a Result<Connection, GraphQLError>, where Connection is a Stream of values returned by the operation, and a GraphQLError is the error when the subscription operation fails.

type Schema = RootNode<'static, Query, EmptyMutation<Database>, Subscription>;

fn schema() -> Schema {
Schema::new(Query, EmptyMutation::new(), Subscription)
}

async fn run_subscription() {
let schema = schema();
let coordinator = Coordinator::new(schema);
let db = Database::new();

    let req: GraphQLRequest<DefaultScalarValue> = serde_json::from_str(
        r#"{
            "query": "subscription { helloWorld }"
        }"#,
    ).unwrap();
    
    let mut conn = coordinator.subscribe(&req, &db).await.unwrap();
    while let Some(result) = conn.next().await {
        println!("{}", serde_json::to_string(&result).unwrap());
    }
}
WebSocket
For information about serving GraphQL subscriptions over WebSocket, see the "Serving" chapter.

---

WebSocket
NOTE: WebSocket is a crucial part for serving GraphQL subscriptions over HTTP.

There are two widely adopted protocols for serving GraphQL over WebSocket:

Legacy graphql-ws GraphQL over WebSocket Protocol, formerly used by Apollo and the subscriptions-transport-ws npm package, and now being deprecated.
New graphql-transport-ws GraphQL over WebSocket Protocol, provided by the graphql-ws npm package and being used by Apollo as for now.
In the Juniper ecosystem, both implementations are provided by the juniper_graphql_ws crate. Most of the officially supported web server framework integrations are able to serve a GraphQL schema over WebSocket (including subscriptions) and even support auto-negotiation of the correct protocol based on the Sec-Websocket-Protocol HTTP header value. See their API docs and usage examples (accessible from API docs) for further details of how to do so.
---
Juniper Book
Serving
Once we have built a GraphQL schema, the next obvious step would be to serve it, so clients can interact with our GraphQL API. Usually, GraphQL APIs are served via HTTP.

Web server frameworks
Though the juniper crate doesn't provide a built-in HTTP server, the surrounding ecosystem does.

Officially supported
Juniper officially supports the following widely used and adopted web server frameworks in Rust ecosystem:

actix-web (juniper_actix crate)
axum (juniper_axum crate)
hyper ([juniper_hyper] crate)
rocket (juniper_rocket crate)
warp (juniper_warp crate)
See their API docs and usage examples (accessible from API docs) for further details of how they should be used.

NOTE: All the officially supported web server framework integrations provide a simple and convenient way for exposing GraphiQL and/or GraphQL Playground with the GraphQL schema along. These powerful tools ease the development process by enabling you to explore and send client requests to the GraphQL API under development.

