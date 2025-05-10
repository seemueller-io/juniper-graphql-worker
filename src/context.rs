use crate::database;
use worker::Env;

pub struct Context {
    pub(crate) db: database::DatabasePool,
    pub(crate) env: Env,
}

// enables passing variables through to the resolvers
impl juniper::Context for Context {}
