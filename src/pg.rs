//! Repository implementation for PostgreSQL.
use chrono::DateTime;
use chrono::Utc;
use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

use std::future::Future;
use std::pin::Pin;

use anyhow::{bail, Result};
use async_trait::async_trait;

use sqlx::postgres::PgRow;
use sqlx::{PgConnection, PgPool, Postgres, QueryBuilder, Row};
use tokio::sync::RwLock;

use crate::base::{Db, Repo};
use crate::query::{Op, Order, Query, F};

/// Value that can be saved to a database.
pub enum Value {
    Str(String),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    Bool(bool),
    Decimal(Decimal),
    Uuid(Uuid),
    DateTime(DateTime<Utc>),
    Null,
}

impl Value {
    fn push_to(&self, builder: &mut QueryBuilder<Postgres>) {
        match self {
            Self::Str(val) => builder.push_bind(val.clone()),
            Self::Int8(val) => builder.push_bind(*val),
            Self::Int16(val) => builder.push_bind(*val),
            Self::Int32(val) => builder.push_bind(*val),
            Self::Int64(val) => builder.push_bind(*val),
            Self::Float32(val) => builder.push_bind(*val),
            Self::Float64(val) => builder.push_bind(*val),
            Self::Bool(val) => builder.push_bind(*val),
            Self::Decimal(val) => builder.push_bind(*val),
            Self::Uuid(val) => builder.push_bind(*val),
            Self::DateTime(val) => builder.push_bind(*val),
            Self::Null => builder.push("null"),
        };
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::Str(value)
    }
}

impl From<i8> for Value {
    fn from(value: i8) -> Self {
        Self::Int8(value)
    }
}

impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Self::Int16(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Int32(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Int64(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Self::Float32(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Float64(value)
    }
}

impl From<Decimal> for Value {
    fn from(value: Decimal) -> Self {
        Self::Decimal(value)
    }
}

impl From<Uuid> for Value {
    fn from(value: Uuid) -> Self {
        Self::Uuid(value)
    }
}

impl From<DateTime<Utc>> for Value {
    fn from(value: DateTime<Utc>) -> Self {
        Self::DateTime(value)
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            None => Self::Null,
            Some(value) => value.into(),
        }
    }
}

use sqlx::database::HasArguments;
// type PgQuery<'a> = sqlx::query::Query<'a, Postgres, Postgres>;
type PgQuery<'a> = sqlx::query::Query<'a, Postgres, <Postgres as HasArguments<'a>>::Arguments>;

/// Repository that stores entities in PostgreSQL.
#[derive(Clone)]
pub struct PgRepo<T> {
    table: String,
    query: String,
    dump: fn(entity: &T) -> HashMap<String, Value>,
    load: fn(row: &PgRow) -> T,
    after_add_hook: Option<fn(&T) -> Vec<PgQuery>>,
    after_update_hook: Option<fn(&T) -> Vec<PgQuery>>,
    after_delete_hook: Option<fn(&F) -> Vec<PgQuery>>,
}

impl<T> PgRepo<T> {
    pub fn new(
        table: impl Into<String>,
        dump: fn(entity: &T) -> HashMap<String, Value>,
        load: fn(row: &PgRow) -> T,
    ) -> Self {
        let table: String = table.into();
        let query = format!("select * from {}", table);

        Self {
            table,
            dump,
            load,
            query,
            after_add_hook: None,
            after_update_hook: None,
            after_delete_hook: None,
        }
    }

    /// Sets a query to select records from a database.
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = query.into();
        self
    }

    /// Sets a function that returns a vector of queries
    /// to execute after a new entity is saved to a database.
    pub fn after_add(mut self, hook: fn(&T) -> Vec<PgQuery>) -> Self {
        self.after_add_hook = Some(hook);
        self
    }

    /// Sets a function that returns a vector of queries
    /// to execute after an updated entity is saved to a database.
    pub fn after_update(mut self, hook: fn(&T) -> Vec<PgQuery>) -> Self {
        self.after_update_hook = Some(hook);
        self
    }

    /// Sets a function that returns a vector of queries
    /// to execute after an entity is deleted from a database.
    pub fn after_delete(mut self, hook: fn(&F) -> Vec<PgQuery>) -> Self {
        self.after_delete_hook = Some(hook);
        self
    }

    fn apply_filter(&self, builder: &mut QueryBuilder<Postgres>, filter: &F) {
        builder.push(" where ");
        self.add_condition(builder, filter);
    }

    fn add_condition(&self, builder: &mut QueryBuilder<Postgres>, filter: &F) {
        match filter {
            F::And(filters) => {
                builder.push("(");
                for (n, filter) in filters.iter().enumerate() {
                    if n != 0 {
                        builder.push(" and ");
                    }
                    self.add_condition(builder, filter);
                }
                builder.push(")");
            }
            F::Or(filters) => {
                builder.push("(");
                for (n, filter) in filters.iter().enumerate() {
                    if n != 0 {
                        builder.push(" or ");
                    }
                    self.add_condition(builder, filter);
                }
                builder.push(")");
            }
            F::IsNone(field) => {
                builder.push(field).push(" is null");
            }
            F::Value { field, op } => match op {
                Op::StrEq(val) => {
                    builder.push(field).push(" = ").push_bind(val.clone());
                }
                Op::StrNe(val) => {
                    builder.push(field).push(" != ").push_bind(val.clone());
                }
                Op::StrContains(val) => {
                    builder
                        .push(field)
                        .push(" like '%' || ")
                        .push_bind(val.clone())
                        .push(" || '%' ");
                }
                Op::StrStartsWith(val) => {
                    builder
                        .push(field)
                        .push(" like ")
                        .push_bind(val.clone())
                        .push(" || '%' ");
                }
                Op::StrEndsWith(val) => {
                    builder
                        .push(field)
                        .push(" like '%' || ")
                        .push_bind(val.clone());
                }
                Op::StrIn(values) => {
                    builder.push(field).push(" in (");
                    let mut sep = builder.separated(", ");
                    for v in values {
                        sep.push_bind(v.clone());
                    }
                    builder.push(")");
                }
                Op::IntEq(val) => {
                    builder.push(field).push(" = ").push_bind(val.clone());
                }
                Op::IntNe(val) => {
                    builder.push(field).push(" != ").push_bind(val.clone());
                }
                Op::IntLt(val) => {
                    builder.push(field).push(" < ").push_bind(val.clone());
                }
                Op::IntGt(val) => {
                    builder.push(field).push(" > ").push_bind(val.clone());
                }
                Op::IntLte(val) => {
                    builder.push(field).push(" <= ").push_bind(val.clone());
                }
                Op::IntGte(val) => {
                    builder.push(field).push(" >= ").push_bind(val.clone());
                }
                Op::IntBetween(x, y) => {
                    builder
                        .push(field)
                        .push(" between ")
                        .push_bind(x.clone())
                        .push(" and ")
                        .push_bind(y.clone());
                }
                Op::IntIn(values) => {
                    builder.push(field).push(" in (");
                    let mut sep = builder.separated(", ");
                    for v in values {
                        sep.push_bind(v.clone());
                    }
                    builder.push(")");
                }
                Op::BoolEq(val) => {
                    builder.push(field).push(" = ").push_bind(val.clone());
                }
                Op::BoolNe(val) => {
                    builder.push(field).push(" != ").push_bind(val.clone());
                }
                Op::FloatEq(val) => {
                    builder.push(field).push(" = ").push_bind(val.clone());
                }
                Op::FloatNe(val) => {
                    builder.push(field).push(" != ").push_bind(val.clone());
                }
                Op::FloatLt(val) => {
                    builder.push(field).push(" < ").push_bind(val.clone());
                }
                Op::FloatGt(val) => {
                    builder.push(field).push(" > ").push_bind(val.clone());
                }
                Op::FloatLte(val) => {
                    builder.push(field).push(" <= ").push_bind(val.clone());
                }
                Op::FloatGte(val) => {
                    builder.push(field).push(" >= ").push_bind(val.clone());
                }
                Op::DateTimeEq(val) => {
                    builder.push(field).push(" = ").push_bind(val.clone());
                }
                Op::DateTimeNe(val) => {
                    builder.push(field).push(" != ").push_bind(val.clone());
                }
                Op::DateTimeLt(val) => {
                    builder.push(field).push(" < ").push_bind(val.clone());
                }
                Op::DateTimeGt(val) => {
                    builder.push(field).push(" > ").push_bind(val.clone());
                }
                Op::DateTimeLte(val) => {
                    builder.push(field).push(" <= ").push_bind(val.clone());
                }
                Op::DateTimeGte(val) => {
                    builder.push(field).push(" >= ").push_bind(val.clone());
                }
                Op::DecimalEq(val) => {
                    builder.push(field).push(" = ").push_bind(val.clone());
                }
                Op::DecimalNe(val) => {
                    builder.push(field).push(" != ").push_bind(val.clone());
                }
                Op::DecimalLt(val) => {
                    builder.push(field).push(" < ").push_bind(val.clone());
                }
                Op::DecimalGt(val) => {
                    builder.push(field).push(" > ").push_bind(val.clone());
                }
                Op::DecimalLte(val) => {
                    builder.push(field).push(" <= ").push_bind(val.clone());
                }
                Op::DecimalGte(val) => {
                    builder.push(field).push(" >= ").push_bind(val.clone());
                }
                Op::UuidEq(val) => {
                    builder.push(field).push(" = ").push_bind(val.clone());
                }
                Op::UuidNe(val) => {
                    builder.push(field).push(" != ").push_bind(val.clone());
                }
                Op::UuidIn(values) => {
                    builder.push(field).push(" in (");
                    let mut sep = builder.separated(", ");
                    for v in values {
                        sep.push_bind(v.clone());
                    }
                    builder.push(")");
                }
            },
        }
    }

    async fn get_via(
        &self,
        conn: &mut PgConnection,
        filter: &F,
        for_update: bool,
    ) -> Result<Option<T>> {
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(self.query.clone());
        self.apply_filter(&mut builder, filter);

        if for_update {
            builder.push(" for update");
        }

        let query = builder.build();
        let result = query.fetch_one(&mut *conn).await;

        match result {
            Ok(row) => Ok(Some((self.load)(&row))),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(err) => bail!(err),
        }
    }

    async fn get_many_via(&self, conn: &mut PgConnection, query: &Query) -> Result<Vec<T>> {
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(self.query.clone());

        if let Some(filter) = &query.filter {
            self.apply_filter(&mut builder, &filter);
        }

        if let Some(order) = &query.order {
            builder.push(" order by ");
            for (n, order_item) in order.iter().enumerate() {
                if n > 0 {
                    builder.push(", ");
                }
                match order_item {
                    Order::Asc(field) => {
                        builder.push(field).push(" asc");
                    }
                    Order::Desc(field) => {
                        builder.push(field).push(" desc");
                    }
                }
            }
        }

        if let Some(limit) = query.limit {
            builder.push(" limit ").push_bind(limit as i64);
        }

        if let Some(offset) = query.offset {
            builder.push(" offset ").push_bind(offset as i64);
        }

        let query = builder.build();
        let result = query.fetch_all(&mut *conn).await;

        match result {
            Ok(rows) => Ok(rows.iter().map(self.load).collect()),
            Err(err) => bail!(err),
        }
    }

    async fn update_via(&self, conn: &mut PgConnection, filter: &F, entity: &T) -> Result<()> {
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("update ");
        builder.push(&self.table);
        builder.push(" set ");

        let data = (self.dump)(entity);

        for (n, (key, value)) in data.iter().enumerate() {
            if n != 0 {
                builder.push(", ");
            }
            builder.push(key).push(" = ");
            value.push_to(&mut builder);
        }

        self.apply_filter(&mut builder, filter);
        let query = builder.build();
        query.execute(&mut *conn).await?;

        if let Some(after_update) = self.after_update_hook {
            for q in after_update(entity) {
                q.execute(&mut *conn).await?;
            }
        }

        Ok(())
    }

    async fn delete_via(&self, conn: &mut PgConnection, filter: &F) -> Result<()> {
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("delete from ");
        builder.push(&self.table);
        self.apply_filter(&mut builder, filter);
        let query = builder.build();
        query.execute(&mut *conn).await?;

        if let Some(after_delete) = self.after_delete_hook {
            for q in after_delete(filter) {
                q.execute(&mut *conn).await?;
            }
        }

        Ok(())
    }

    async fn add_via(&self, conn: &mut PgConnection, entity: &T) -> Result<()> {
        let data = (self.dump)(entity);
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("insert into ");
        builder.push(&self.table);
        builder.push(" (");
        let mut separated = builder.separated(", ");
        for key in data.keys() {
            separated.push(key);
        }
        builder.push(") values (");
        for (n, val) in data.values().enumerate() {
            if n > 0 {
                builder.push(", ");
            }
            val.push_to(&mut builder);
        }
        builder.push(")");
        let query = builder.build();
        query.execute(&mut *conn).await?;

        if let Some(after_add) = self.after_add_hook {
            for q in after_add(entity) {
                q.execute(&mut *conn).await?;
            }
        }

        Ok(())
    }

    async fn exists_via(&self, conn: &mut PgConnection, filter: &F) -> Result<bool> {
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("select exists (");
        builder.push(self.query.clone());
        self.apply_filter(&mut builder, filter);
        builder.push(") as result");
        let query = builder.build();
        let result = query.fetch_one(&mut *conn).await;

        match result {
            Ok(row) => Ok(row.get("result")),
            Err(err) => bail!(err),
        }
    }

    async fn count_via(&self, conn: &mut PgConnection, filter: &F) -> Result<i64> {
        let mut builder: QueryBuilder<Postgres> =
            QueryBuilder::new("select count(1) as result from (");
        builder.push(self.query.clone());
        self.apply_filter(&mut builder, filter);
        builder.push(") as q");
        let query = builder.build();
        let result = query.fetch_one(&mut *conn).await;

        match result {
            Ok(row) => Ok(row.get("result")),
            Err(err) => bail!(err),
        }
    }

    async fn count_all_via(&self, conn: &mut PgConnection) -> Result<i64> {
        let mut builder: QueryBuilder<Postgres> =
            QueryBuilder::new("select count(1) as result from (");
        builder.push(self.query.clone()).push(") as q");
        let query = builder.build();
        let result = query.fetch_one(&mut *conn).await;

        match result {
            Ok(row) => Ok(row.get("result")),
            Err(err) => bail!(err),
        }
    }
}

#[async_trait]
impl<T> Repo<T> for PgRepo<T>
where
    T: Sync + Send,
{
    type Db<'a> = PgDb<'a>;

    async fn get<'a>(&self, db: &Self::Db<'a>, filter: &F) -> Result<Option<T>> {
        match db {
            PgDb::Pool(p) => {
                let mut conn = p.acquire().await?;
                self.get_via(&mut conn, filter, false).await
            }
            PgDb::Transaction(t) => {
                let mut t = t.write().await;
                self.get_via(&mut *t, filter, false).await
            }
        }
    }

    async fn get_many<'a>(&self, db: &Self::Db<'a>, query: &Query) -> Result<Vec<T>> {
        match db {
            PgDb::Pool(p) => {
                let mut conn = p.acquire().await?;
                self.get_many_via(&mut conn, query).await
            }
            PgDb::Transaction(t) => {
                let mut t = t.write().await;
                self.get_many_via(&mut *t, query).await
            }
        }
    }

    async fn update<'a>(&self, db: &Self::Db<'a>, filter: &F, entity: &T) -> Result<()> {
        match db {
            PgDb::Pool(p) => {
                let mut conn = p.acquire().await?;
                self.update_via(&mut conn, filter, entity).await
            }
            PgDb::Transaction(t) => {
                let mut t = t.write().await;
                self.update_via(&mut *t, filter, entity).await
            }
        }
    }

    async fn delete<'a>(&self, db: &Self::Db<'a>, filter: &F) -> Result<()> {
        match db {
            PgDb::Pool(p) => {
                let mut c = p.acquire().await?;
                self.delete_via(&mut c, filter).await
            }
            PgDb::Transaction(t) => {
                let mut t = t.write().await;
                self.delete_via(&mut t, filter).await
            }
        }
    }

    async fn add<'a>(&self, db: &Self::Db<'a>, entity: &T) -> Result<()> {
        match db {
            PgDb::Pool(p) => {
                let mut conn = p.acquire().await?;
                self.add_via(&mut conn, entity).await
            }
            PgDb::Transaction(t) => {
                let mut t = t.write().await;
                self.add_via(&mut *t, entity).await
            }
        }
    }

    async fn exists<'a>(&self, db: &Self::Db<'a>, filter: &F) -> Result<bool> {
        match db {
            PgDb::Pool(p) => {
                let mut conn = p.acquire().await?;
                self.exists_via(&mut conn, filter).await
            }
            PgDb::Transaction(t) => {
                let mut t = t.write().await;
                self.exists_via(&mut *t, filter).await
            }
        }
    }

    async fn count<'a>(&self, db: &Self::Db<'a>, filter: &F) -> Result<i64> {
        match db {
            PgDb::Pool(p) => {
                let mut conn = p.acquire().await?;
                self.count_via(&mut conn, filter).await
            }
            PgDb::Transaction(t) => {
                let mut t = t.write().await;
                self.count_via(&mut *t, filter).await
            }
        }
    }

    async fn count_all<'a>(&self, db: &Self::Db<'a>) -> Result<i64> {
        match db {
            PgDb::Pool(p) => {
                let mut conn = p.acquire().await?;
                self.count_all_via(&mut conn).await
            }
            PgDb::Transaction(t) => {
                let mut t = t.write().await;
                self.count_all_via(&mut *t).await
            }
        }
    }

    async fn get_for_update<'a>(&self, db: &Self::Db<'a>, filter: &F) -> Result<Option<T>> {
        match db {
            PgDb::Pool(p) => {
                let mut conn = p.acquire().await?;
                self.get_via(&mut conn, filter, true).await
            }
            PgDb::Transaction(t) => {
                let mut t = t.write().await;
                self.get_via(&mut *t, filter, true).await
            }
        }
    }
}

/// Wrapper around a pool of connections to PostgreSQL
/// or a transaction.
pub enum PgDb<'a> {
    Pool(PgPool),
    Transaction(RwLock<sqlx::Transaction<'a, Postgres>>),
}

impl PgDb<'_> {
    pub fn new(pool: PgPool) -> Self {
        Self::Pool(pool)
    }
}

#[async_trait]
impl Db for PgDb<'_> {
    async fn transaction<A, T>(&self, action: A) -> Result<T>
    where
        A: for<'a> FnOnce(&'a Self) -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>> + Send,
        T: Send,
    {
        let wrapped = RwLock::new(match self {
            Self::Transaction(_) => todo!(),
            Self::Pool(p) => p.begin().await?,
        });

        let tx = Self::Transaction(wrapped);

        match action(&tx).await {
            Ok(res) => {
                if let Self::Transaction(t) = tx {
                    t.into_inner().commit().await?;
                }
                Ok(res)
            }
            Err(err) => {
                if let Self::Transaction(t) = tx {
                    t.into_inner().rollback().await?;
                }
                bail!(err)
            }
        }
    }
}
