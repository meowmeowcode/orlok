//! Main traits are here.
use std::future::Future;
use std::pin::Pin;

use anyhow::Result;
use async_trait::async_trait;

use crate::query::{Query, F};

/// Trait that must be implemented for a repository.
#[async_trait]
pub trait Repo<T> {
    /// Type of a database-connection wrapper.
    type Db<'a>;
    /// Finds an entity and returns it.
    /// Returns `None` if the entity is missing.
    async fn get<'a>(&self, db: &Self::Db<'a>, filter: &F) -> Result<Option<T>>;
    /// Finds and returns several entities.
    async fn get_many<'a>(&self, db: &Self::Db<'a>, query: &Query) -> Result<Vec<T>>;
    /// Adds a new entry to the repository.
    async fn add<'a>(&self, db: &Self::Db<'a>, entity: &T) -> Result<()>;
    /// Saves an updated entity.
    async fn update<'a>(&self, db: &Self::Db<'a>, filter: &F, entity: &T) -> Result<()>;
    /// Deletes entities matching a given filter.
    async fn delete<'a>(&self, db: &Self::Db<'a>, filter: &F) -> Result<()>;
    /// Checks if there is an entity matching a given filter.
    async fn exists<'a>(&self, db: &Self::Db<'a>, filter: &F) -> Result<bool>;
    /// Counts entities matching a given filter.
    async fn count<'a>(&self, db: &Self::Db<'a>, filter: &F) -> Result<i64>;
    /// Counts entities in the repository.
    async fn count_all<'a>(&self, db: &Self::Db<'a>) -> Result<i64>;
    /// Finds an entity and locks it for update. Returns `None` if the entity is missing.
    async fn get_for_update<'a>(&self, transaction: &Self::Db<'a>, filter: &F)
        -> Result<Option<T>>;
}

/// Trait that must be implemented for a database-connection wrapper.
#[async_trait]
pub trait Db {
    async fn transaction<A, T>(&self, action: A) -> Result<T>
    where
        A: for<'a> FnOnce(&'a Self) -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>> + Send,
        T: Send;
}
