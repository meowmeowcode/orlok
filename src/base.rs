use async_trait::async_trait;

use crate::query::{Query, F};

#[async_trait]
pub trait Repo<T> {
    async fn get(&self, filter: &F) -> Option<T>;
    async fn get_many(&self, query: &Query) -> Vec<T>;
    async fn add(&self, entity: &T);
    async fn update(&self, filter: &F, entity: &T);
    async fn delete(&self, filter: &F);
}
