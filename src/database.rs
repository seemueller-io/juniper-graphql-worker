use crate::models;

pub struct DatabasePool;

impl DatabasePool {
    pub(crate) fn get_connection(&self) -> Result<DatabaseConnection, &'static str> {
        Ok(DatabaseConnection)
    }
}

pub struct DatabaseConnection;

impl DatabaseConnection {
    pub(crate) fn find_human(&self, id: &str) -> Result<models::Human, &'static str> {
        Ok(models::Human {
            id: id.to_string(),
            name: "Luke Skywalker".to_string(),
            appears_in: vec![
                models::Episode::NewHope,
                models::Episode::Empire,
                models::Episode::Jedi,
            ],
            home_planet: "Tatooine".to_string(),
        })
    }

    pub(crate) fn insert_human(
        &self,
        new_human: &models::NewHuman,
    ) -> Result<models::Human, &'static str> {
        Ok(models::Human {
            id: "generated-id".to_string(),
            name: new_human.name.clone(),
            appears_in: new_human.appears_in.clone(),
            home_planet: new_human.home_planet.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Episode, NewHuman};

    #[test]
    fn test_get_connection() {
        let pool = DatabasePool;
        let connection_result = pool.get_connection();

        // Verify that we can get a connection successfully
        assert!(connection_result.is_ok());
    }

    #[test]
    fn test_find_human() {
        let connection = DatabaseConnection;
        let human_result = connection.find_human("1");

        // Verify that we can find a human successfully
        assert!(human_result.is_ok());

        let human = human_result.unwrap();
        assert_eq!(human.id, "1");
        assert_eq!(human.name, "Luke Skywalker");
        assert_eq!(human.appears_in.len(), 3);
        assert_eq!(human.home_planet, "Tatooine");
    }

    #[test]
    fn test_insert_human() {
        let connection = DatabaseConnection;
        let new_human = NewHuman {
            name: "Han Solo".to_string(),
            appears_in: vec![Episode::NewHope, Episode::Empire, Episode::Jedi],
            home_planet: "Corellia".to_string(),
        };

        let human_result = connection.insert_human(&new_human);

        // Verify that we can insert a human successfully
        assert!(human_result.is_ok());

        let human = human_result.unwrap();
        assert_eq!(human.id, "generated-id");
        assert_eq!(human.name, "Han Solo");
        assert_eq!(human.appears_in.len(), 3);
        assert_eq!(human.home_planet, "Corellia");
    }
}
