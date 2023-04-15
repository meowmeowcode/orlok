use anyhow::Result;
use async_trait::async_trait;

use crate::query::{Query, F};

#[async_trait]
pub trait Repo<T> {
    async fn get(&self, filter: &F) -> Result<Option<T>>;
    async fn get_many(&self, query: &Query) -> Result<Vec<T>>;
    async fn add(&self, entity: &T) -> Result<()>;
    async fn update(&self, filter: &F, entity: &T) -> Result<()>;
    async fn delete(&self, filter: &F) -> Result<()>;
}
