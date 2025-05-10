use crate::{context, models};
use juniper::{graphql_object, EmptySubscription, FieldError, FieldResult, Value};

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
        Ok(human)
    }
}

pub type Schema = juniper::RootNode<'static, Query, Mutation, EmptySubscription<context::Context>>;
