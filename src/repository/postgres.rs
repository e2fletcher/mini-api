use super::{RepositoryError, Todo, TodoRepository, TodoRepositoryFactory};
use async_trait::async_trait;
use uuid::Uuid;

use diesel::prelude::*;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::RunQueryDsl;

pub struct PostgresTodoRepository(Pool<diesel_async::AsyncPgConnection>);

diesel::table! {
   todos(id) {
       id -> Uuid,
       text -> Text,
       completed -> Bool,
   }
}

#[derive(Insertable)]
#[diesel(table_name = todos)]
pub struct NewTodo {
    pub id: Uuid,
    pub text: String,
}

#[derive(AsChangeset)]
#[diesel(table_name = todos)]
struct UpdateTodo {
    text: Option<String>,
    completed: Option<bool>,
}

#[async_trait]
impl TodoRepository for PostgresTodoRepository {
    async fn list(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Todo>, RepositoryError> {
        use self::todos::dsl::*;
        let mut conn = self.0.get().await.unwrap();

        let offset: i64 = offset.unwrap_or(0).try_into().unwrap();
        let limit: i64 = match limit {
            Some(limit) => limit.try_into().unwrap(),
            None => i64::MAX,
        };

        todos
            .offset(offset)
            .limit(limit)
            .load(&mut conn)
            .await
            .map_err(|_| RepositoryError)
    }

    async fn get(&self, todo_id: Uuid) -> Result<Todo, RepositoryError> {
        use self::todos::dsl::*;
        let mut conn = self.0.get().await.unwrap();

        todos
            .filter(id.eq(todo_id))
            .first(&mut conn)
            .await
            .map_err(|_| RepositoryError)
    }

    async fn update(
        &mut self,
        todo_id: Uuid,
        todo_text: Option<String>,
        todo_completed: Option<bool>,
    ) -> Result<Todo, RepositoryError> {
        let mut conn = self.0.get().await.unwrap();

        let todo = diesel::update(todos::table)
            .filter(todos::dsl::id.eq(todo_id))
            .set(&UpdateTodo {
                text: todo_text,
                completed: todo_completed,
            })
            .get_result(&mut conn)
            .await
            .map_err(|_| RepositoryError)?;

        Ok(todo)
    }

    async fn create(&mut self, text: String) -> Result<Todo, RepositoryError> {
        let mut conn = self.0.get().await.unwrap();
        let id = Uuid::new_v4();
        let new_todo = NewTodo { id, text };

        diesel::insert_into(todos::table)
            .values(&new_todo)
            .returning(Todo::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|_| RepositoryError)
    }

    async fn delete(&mut self, todo_id: Uuid) -> Result<(), RepositoryError> {
        let mut conn = self.0.get().await.unwrap();

        diesel::delete(todos::table)
            .filter(todos::dsl::id.eq(todo_id))
            .execute(&mut conn)
            .await
            .map_err(|_| RepositoryError)?;

        Ok(())
    }
}

#[async_trait]
impl TodoRepositoryFactory for PostgresTodoRepository {
    async fn create_repository() -> Box<dyn TodoRepository + Sync + Send + 'static> {
        let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
            "postgres://default:secret@localhost:5432/todo_api",
        );
        let pool = Pool::builder(config).build().unwrap();

        Box::new(Self(pool))
    }
}
