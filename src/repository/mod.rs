pub mod memory;
pub mod postgres;

use diesel::{Selectable, Queryable};
use serde::Serialize;
use uuid::Uuid;
use async_trait::async_trait;

#[derive(Debug, Serialize, Clone)]
#[derive(Queryable, Selectable)]
#[diesel(table_name = postgres::todos)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Todo {
    id: Uuid,
    text: String,
    completed: bool,
}

#[derive(Debug)]
pub struct RepositoryError;

#[async_trait]
pub trait TodoRepository {
    async fn list(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Todo>, RepositoryError>;
    async fn get(&self, id: Uuid) -> Result<Todo, RepositoryError>;
    async fn update(
        &mut self,
        id: Uuid,
        text: Option<String>,
        completed: Option<bool>,
    ) -> Result<Todo, RepositoryError>;
    async fn create(&mut self, text: String) -> Result<Todo, RepositoryError>;
    async fn delete(&mut self, id: Uuid) -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait TodoRepositoryFactory {
    async fn create_repository() -> Box<dyn TodoRepository + Sync + Send + 'static>;
}
