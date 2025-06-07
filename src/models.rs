use juniper::{GraphQLEnum, GraphQLInputObject, GraphQLObject};

#[derive(Clone, Copy, Debug, PartialEq, GraphQLEnum)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_episode_enum() {
        // Test that Episode enum values can be created and compared
        let new_hope = Episode::NewHope;
        let empire = Episode::Empire;
        let jedi = Episode::Jedi;

        assert_ne!(new_hope, empire);
        assert_ne!(new_hope, jedi);
        assert_ne!(empire, jedi);
    }

    #[test]
    fn test_human_creation() {
        // Test that Human struct can be created with valid values
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
    fn test_new_human_creation() {
        // Test that NewHuman struct can be created with valid values
        let new_human = NewHuman {
            name: "Han Solo".to_string(),
            appears_in: vec![Episode::NewHope, Episode::Empire, Episode::Jedi],
            home_planet: "Corellia".to_string(),
        };

        assert_eq!(new_human.name, "Han Solo");
        assert_eq!(new_human.appears_in.len(), 3);
        assert_eq!(new_human.home_planet, "Corellia");
    }
}
