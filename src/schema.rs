use crate::{context, models};
use futures::{Stream, StreamExt};
use juniper::{graphql_object, graphql_subscription, FieldError, FieldResult, Value};
use std::pin::Pin;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

// Global channel for human creation events
lazy_static::lazy_static! {
    static ref HUMAN_CREATED_CHANNEL: (broadcast::Sender<models::Human>, broadcast::Receiver<models::Human>) = {
        let (tx, rx) = broadcast::channel(100);
        (tx, rx)
    };
}

pub(crate) struct Query;

#[graphql_object]
#[graphql(context = context::Context)]
impl Query {
    fn api_version() -> &'static str {
        "1.0"
    }

    fn human(id: String, context: &context::Context) -> FieldResult<models::Human> {
        let conn = context
            .db
            .get_connection()
            .map_err(|e| FieldError::new(e.to_string(), Value::null()))?;
        let human = conn
            .find_human(&id)
            .map_err(|e| FieldError::new(e.to_string(), Value::null()))?;
        Ok(human)
    }
}

pub(crate) struct Mutation;

#[graphql_object]
#[graphql(context = context::Context)]
impl Mutation {
    fn create_human(
        new_human: models::NewHuman,
        context: &context::Context,
    ) -> FieldResult<models::Human> {
        let db = context
            .db
            .get_connection()
            .map_err(|e| FieldError::new(e.to_string(), Value::null()))?;
        let human = db
            .insert_human(&new_human)
            .map_err(|e| FieldError::new(e.to_string(), Value::null()))?;

        // Publish the human creation event to subscribers
        let _ = HUMAN_CREATED_CHANNEL.0.send(human.clone());

        Ok(human)
    }
}

// Define the Subscription type
pub(crate) struct Subscription;

type HumanStream = Pin<Box<dyn Stream<Item = Result<models::Human, FieldError>> + Send>>;

#[graphql_subscription]
#[graphql(context = context::Context)]
impl Subscription {
    /// Subscribe to human creation events
    async fn human_created(_context: &context::Context) -> HumanStream {
        // Create a new receiver by subscribing to the channel
        let mut rx = HUMAN_CREATED_CHANNEL.0.subscribe();

        // Convert the receiver into a stream
        let stream = BroadcastStream::new(rx)
            .map(|result| match result {
                Ok(human) => Ok(human),
                Err(e) => Err(FieldError::new(e.to_string(), Value::null())),
            });

        Box::pin(stream)
    }
}

pub type Schema = juniper::RootNode<'static, Query, Mutation, Subscription>;

// Helper functions for testing
#[cfg(test)]
pub mod test_helpers {
    use super::*;
    use crate::database::DatabasePool;
    use crate::models::{Episode, NewHuman};
    use futures::StreamExt;

    // Create a test context
    pub fn create_test_context() -> context::Context {
        context::Context {
            db: DatabasePool,
            env: None, // No environment needed for tests
        }
    }

    // Test the api_version query
    pub fn test_api_version() -> &'static str {
        Query::api_version()
    }

    // Test the human query
    pub fn test_human_query(id: String) -> FieldResult<models::Human> {
        let ctx = create_test_context();
        Query::human(id, &ctx)
    }

    // Test the create_human mutation
    pub fn test_create_human_mutation(new_human: NewHuman) -> FieldResult<models::Human> {
        let ctx = create_test_context();
        Mutation::create_human(new_human, &ctx)
    }

    // Test the human_created subscription
    pub async fn test_human_created_subscription() -> FieldResult<models::Human> {
        let ctx = create_test_context();

        // Get the subscription stream
        let mut stream = Subscription::human_created(&ctx).await;

        // Create a new human to trigger the subscription
        let new_human = NewHuman {
            name: "Test Subscription".to_string(),
            appears_in: vec![Episode::NewHope],
            home_planet: "Test Planet".to_string(),
        };

        // Trigger the subscription by creating a human
        let _ = Mutation::create_human(new_human, &ctx)?;

        // Get the first item from the stream
        if let Some(result) = stream.next().await {
            result
        } else {
            Err(FieldError::new("No subscription events received", Value::null()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DatabasePool;
    use crate::models::{Episode, NewHuman};
    use juniper::Variables;
    use std::sync::Arc;

    #[test]
    fn test_api_version_query() {
        // Test the api_version resolver using the test helper
        let result = test_helpers::test_api_version();
        assert_eq!(result, "1.0");
    }

    #[test]
    fn test_human_query() {
        // Test the human resolver using the test helper
        let result = test_helpers::test_human_query("1".to_string());
        assert!(result.is_ok());

        let human = result.unwrap();
        assert_eq!(human.id, "1");
        assert_eq!(human.name, "Luke Skywalker");
    }

    #[test]
    fn test_create_human_mutation() {
        let new_human = NewHuman {
            name: "Han Solo".to_string(),
            appears_in: vec![Episode::NewHope, Episode::Empire, Episode::Jedi],
            home_planet: "Corellia".to_string(),
        };

        // Test the create_human resolver using the test helper
        let result = test_helpers::test_create_human_mutation(new_human);
        assert!(result.is_ok());

        let human = result.unwrap();
        assert_eq!(human.id, "generated-id");
        assert_eq!(human.name, "Han Solo");
        assert_eq!(human.home_planet, "Corellia");
    }

    #[tokio::test]
    async fn test_human_created_subscription() {
        // Test the human_created subscription using the test helper
        let result = test_helpers::test_human_created_subscription().await;
        assert!(result.is_ok());

        let human = result.unwrap();
        assert_eq!(human.id, "generated-id");
        assert_eq!(human.name, "Test Subscription");
        assert_eq!(human.home_planet, "Test Planet");
    }
}
