use std::future::Future;
use std::pin::Pin;

use anyhow::Result;
use async_trait::async_trait;

use crate::query::{Query, F};

#[async_trait]
pub trait Repo<T> {
    type Transaction;
    async fn get(&self, filter: &F) -> Result<Option<T>>;
    async fn get_many(&self, query: &Query) -> Result<Vec<T>>;
    async fn add(&self, entity: &T) -> Result<()>;
    async fn update(&self, filter: &F, entity: &T) -> Result<()>;
    async fn delete(&self, filter: &F) -> Result<()>;
    async fn exists(&self, filter: &F) -> Result<bool>;
    async fn count(&self, filter: &F) -> Result<i64>;
    async fn count_all(&self) -> Result<i64>;
    async fn get_for_update(
        &self,
        transaction: &mut Self::Transaction,
        filter: &F,
    ) -> Result<Option<T>>;
    async fn get_within(
        &self,
        transaction: &mut Self::Transaction,
        filter: &F,
    ) -> Result<Option<T>>;
    async fn get_many_within(
        &self,
        transaction: &mut Self::Transaction,
        query: &Query,
    ) -> Result<Vec<T>>;
    async fn add_within(&self, transaction: &mut Self::Transaction, entity: &T) -> Result<()>;
    async fn update_within(
        &self,
        transaction: &mut Self::Transaction,
        filter: &F,
        entity: &T,
    ) -> Result<()>;
    async fn delete_within(&self, transaction: &mut Self::Transaction, filter: &F) -> Result<()>;
    async fn exists_within(&self, transaction: &mut Self::Transaction, filter: &F) -> Result<bool>;
    async fn count_within(&self, transaction: &mut Self::Transaction, filter: &F) -> Result<i64>;
    async fn count_all_within(&self, transaction: &mut Self::Transaction) -> Result<i64>;
}

#[async_trait]
pub trait TxManager {
    type Transaction;

    async fn run<A, T>(&self, action: A) -> Result<T>
    where
        A: for<'a> FnOnce(
                &'a mut Self::Transaction,
            ) -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>
            + Send,
        T: Send;
}
