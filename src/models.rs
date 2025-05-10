use juniper::{GraphQLEnum, GraphQLInputObject, GraphQLObject};

#[derive(Clone, Copy, GraphQLEnum)]
pub enum Episode {
    NewHope,
    Empire,
    Jedi,
}

#[derive(GraphQLObject)]
#[graphql(description = "A humanoid creature in the Star Wars universe")]
pub struct Human {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) appears_in: Vec<Episode>,
    pub(crate) home_planet: String,
}

#[derive(GraphQLInputObject)]
#[graphql(description = "A humanoid creature in the Star Wars universe")]
pub struct NewHuman {
    pub(crate) name: String,
    pub(crate) appears_in: Vec<Episode>,
    pub(crate) home_planet: String,
}
