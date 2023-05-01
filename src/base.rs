use std::future::Future;
use std::pin::Pin;

use anyhow::Result;
use async_trait::async_trait;

use crate::query::{Query, F};

#[async_trait]
pub trait Repo<T> {
    type Db<'a>;
    async fn get<'a>(&self, db: &Self::Db<'a>, filter: &F) -> Result<Option<T>>;
    async fn get_many<'a>(&self, db: &Self::Db<'a>, query: &Query) -> Result<Vec<T>>;
    async fn add<'a>(&self, db: &Self::Db<'a>, entity: &T) -> Result<()>;
    async fn update<'a>(&self, db: &Self::Db<'a>, filter: &F, entity: &T) -> Result<()>;
    async fn delete<'a>(&self, db: &Self::Db<'a>, filter: &F) -> Result<()>;
    async fn exists<'a>(&self, db: &Self::Db<'a>, filter: &F) -> Result<bool>;
    async fn count<'a>(&self, db: &Self::Db<'a>, filter: &F) -> Result<i64>;
    async fn count_all<'a>(&self, db: &Self::Db<'a>) -> Result<i64>;
    async fn get_for_update<'a>(&self, transaction: &Self::Db<'a>, filter: &F)
        -> Result<Option<T>>;
}

#[async_trait]
pub trait Db {
    async fn transaction<A, T>(&self, action: A) -> Result<T>
    where
        A: for<'a> FnOnce(&'a Self) -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>> + Send,
        T: Send;
}
