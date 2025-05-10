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
                models::Episode::Jedi
            ],
            home_planet: "Tatooine".to_string(),
        })
    }

    pub(crate) fn insert_human(&self, new_human: &models::NewHuman) -> Result<models::Human, &'static str> {
        Ok(models::Human {
            id: "generated-id".to_string(),
            name: new_human.name.clone(),
            appears_in: new_human.appears_in.clone(),
            home_planet: new_human.home_planet.clone(),
        })
    }
}