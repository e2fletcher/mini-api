use super::{RepositoryError, Todo, TodoRepository, TodoRepositoryFactory};
use std::collections::HashMap;
use async_trait::async_trait;
use uuid::Uuid;

pub struct MemoryTodoRepository {
    pub db: HashMap<Uuid, Todo>,
}

#[async_trait]
impl TodoRepository for MemoryTodoRepository {
    async fn list(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Todo>, RepositoryError> {
        let todos = self
            .db
            .values()
            .skip(offset.unwrap_or_default())
            .take(limit.unwrap_or(usize::MAX))
            .cloned()
            .collect::<Vec<_>>();

        Ok(todos)
    }

    async fn get(&self, id: Uuid) -> Result<Todo, RepositoryError> {
        let todo = self.db.get(&id).cloned().ok_or(RepositoryError)?;
        Ok(todo)
    }

    async fn update(
        &mut self,
        id: Uuid,
        text: Option<String>,
        completed: Option<bool>,
    ) -> Result<Todo, RepositoryError> {
        let mut todo = self.get(id).await?;

        if let Some(text) = text {
            todo.text = text;
        }

        if let Some(completed) = completed {
            todo.completed = completed;
        }

        self.db.insert(todo.id, todo.clone());

        Ok(todo)
    }

    async fn create(&mut self, text: String) -> Result<Todo, RepositoryError> {
        let todo = Todo {
            id: Uuid::new_v4(),
            text,
            completed: false,
        };

        self.db.insert(todo.id, todo.clone());

        Ok(todo)
    }

    async fn delete(&mut self, id: Uuid) -> Result<(), RepositoryError> {
        if self.db.remove(&id).is_some() {
            Ok(())
        } else {
            Err(RepositoryError)
        }
    }
}

#[async_trait]
impl TodoRepositoryFactory for MemoryTodoRepository {
    async fn create_repository() -> Box<dyn TodoRepository + Send + Sync + 'static> {
        Box::new(Self { db: HashMap::new() })
    }
}
