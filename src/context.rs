use crate::{database, CustomEnv};
use worker::Env;


pub struct Context {
    pub(crate) db: database::DatabasePool,
    #[cfg(feature = "local")]
    #[cfg(not(test))]
    pub(crate) env: CustomEnv,
    
    #[cfg(not(test))]
    #[cfg(not(feature = "local"))]
    pub(crate) env: Env,
    #[cfg(not(feature = "local"))]
    #[cfg(test)]
    pub(crate) env: Option<Env>,
    
    #[cfg(feature = "local")]
    #[cfg(test)]
    pub(crate) env: Option<CustomEnv>,
}

// enables passing variables through to the resolvers
impl juniper::Context for Context {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_implements_juniper_context() {
        // This test verifies that Context implements the juniper::Context trait
        // We can test this at compile time without creating an actual instance
        fn takes_juniper_context<T: juniper::Context>() {}

        // If this compiles, it means Context implements juniper::Context
        takes_juniper_context::<Context>();
    }
}
