use std::collections::HashMap;
use std::fmt::Write;
use std::future::Future;
use std::pin::Pin;

use anyhow::{bail, Result};
use async_trait::async_trait;
use sea_query::backend::PostgresQueryBuilder;
use sea_query::{Alias, Expr, Iden, LockType};
use sea_query::{
    Condition, IntoCondition, Order as SeaOrder, Query as SeaQuery, SelectStatement, SimpleExpr,
};
use sea_query_binder::SqlxBinder;
use sqlx::postgres::PgRow;
use sqlx::{PgExecutor, PgPool, Postgres, Row};

use crate::base::{Repo, TxManager};
use crate::query::{Op, Order, Query, F};

#[derive(Clone)]
pub struct PgRepo<'pool, T> {
    pool: &'pool PgPool,
    table: String,
    query: fn(table: &String) -> SelectStatement,
    dump: fn(entity: &T) -> HashMap<String, SimpleExpr>,
    load: fn(row: &PgRow) -> T,
}

pub struct Name(String);

impl Iden for Name {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(s, "{}", self.0).unwrap();
    }
}

fn default_query(table: &String) -> SelectStatement {
    SeaQuery::select()
        .expr(Expr::asterisk())
        .from(Name(table.to_string()))
        .to_owned()
}

impl<'pool, T> PgRepo<'pool, T> {
    pub fn new(
        pool: &'pool PgPool,
        table: String,
        dump: fn(entity: &T) -> HashMap<String, SimpleExpr>,
        load: fn(row: &PgRow) -> T,
    ) -> Self {
        Self {
            pool,
            table,
            dump,
            load,
            query: default_query,
        }
    }

    fn apply_filter(&self, query: &mut sea_query::SelectStatement, filter: &F) {
        let cond = Self::filter_to_cond(filter);
        query.cond_where(cond);
    }

    fn filter_to_cond(filter: &F) -> Condition {
        match filter {
            F::And(filters) => {
                let mut cond = Condition::all();
                for filter in filters {
                    cond = cond.add(Self::filter_to_cond(&filter));
                }
                cond
            }
            F::Or(filters) => {
                let mut cond = Condition::any();
                for filter in filters {
                    cond = cond.add(Self::filter_to_cond(&filter));
                }
                cond
            }
            F::Value { field, op } => {
                let col = Expr::col(Name(field.to_string()));
                match op {
                    Op::StrEq(val) => col.eq(val),
                    Op::StrContains(val) => col.like(format!("%{}%", val)),
                    Op::StrStartsWith(val) => col.like(format!("{}%", val)),
                    Op::StrEndsWith(val) => col.like(format!("%{}", val)),
                    Op::IntEq(val) => col.eq(*val),
                    Op::IntNe(val) => col.ne(*val),
                    Op::IntLt(val) => col.lt(*val),
                    Op::IntGt(val) => col.gt(*val),
                    Op::IntLte(val) => col.lte(*val),
                    Op::IntGte(val) => col.gte(*val),
                    Op::IntBetween(x, y) => col.between(*x, *y),
                    Op::IntIn(values) => col.is_in(values.iter().map(|x| *x)),
                    Op::BoolEq(val) => col.eq(*val),
                    Op::BoolNe(val) => col.ne(*val),
                    Op::FloatEq(val) => col.eq(*val),
                    Op::FloatNe(val) => col.ne(*val),
                    Op::FloatLt(val) => col.lt(*val),
                    Op::FloatGt(val) => col.gt(*val),
                    Op::FloatLte(val) => col.lte(*val),
                    Op::FloatGte(val) => col.gte(*val),
                    Op::DateTimeEq(val) => col.eq(*val),
                    Op::DateTimeNe(val) => col.ne(*val),
                    Op::DateTimeLt(val) => col.lt(*val),
                    Op::DateTimeGt(val) => col.gt(*val),
                    Op::DateTimeLte(val) => col.lte(*val),
                    Op::DateTimeGte(val) => col.gte(*val),
                    Op::DecimalEq(val) => col.eq(*val),
                    Op::DecimalNe(val) => col.ne(*val),
                    Op::DecimalLt(val) => col.lt(*val),
                    Op::DecimalGt(val) => col.gt(*val),
                    Op::DecimalLte(val) => col.lte(*val),
                    Op::DecimalGte(val) => col.gte(*val),
                    Op::UuidEq(val) => col.eq(*val),
                    Op::UuidNe(val) => col.ne(*val),
                    Op::UuidIn(values) => col.is_in(values.iter().map(|x| *x)),
                    _ => todo!(),
                }
                .into_condition()
            }
            _ => todo!(),
        }
    }

    async fn get_via(
        &self,
        executor: impl PgExecutor<'_>,
        filter: &F,
        for_update: bool,
    ) -> Result<Option<T>> {
        let (sql, values) = {
            let mut query = (self.query)(&self.table);
            self.apply_filter(&mut query, filter);
            if for_update {
                query.lock(LockType::Update);
            }
            query.build_sqlx(PostgresQueryBuilder)
        };

        let result = sqlx::query_with(&sql, values).fetch_one(executor).await;

        match result {
            Ok(row) => Ok(Some((self.load)(&row))),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(err) => bail!(err),
        }
    }

    async fn get_many_via(&self, executor: impl PgExecutor<'_>, query: &Query) -> Result<Vec<T>> {
        let (sql, values) = {
            let mut sql = (self.query)(&self.table);

            if let Some(filter) = &query.filter {
                self.apply_filter(&mut sql, filter);
            }

            if let Some(limit) = query.limit {
                sql.limit(limit as u64);
            }

            if let Some(offset) = query.offset {
                sql.offset(offset as u64);
            }

            if let Some(order) = &query.order {
                sql.order_by_columns(
                    order
                        .iter()
                        .map(|x| match x {
                            Order::Asc(field) => (Name(field.to_string()), SeaOrder::Asc),
                            Order::Desc(field) => (Name(field.to_string()), SeaOrder::Desc),
                        })
                        .collect::<Vec<(Name, SeaOrder)>>(),
                );
            }

            sql.build_sqlx(PostgresQueryBuilder)
        };

        let result = sqlx::query_with(&sql, values).fetch_all(executor).await;

        match result {
            Ok(rows) => Ok(rows.iter().map(self.load).collect()),
            Err(err) => bail!(err),
        }
    }

    async fn update_via(
        &self,
        executor: impl PgExecutor<'_>,
        filter: &F,
        entity: &T,
    ) -> Result<()> {
        let (sql, values) = {
            let data = (self.dump)(entity);

            SeaQuery::update()
                .table(Name(self.table.clone()))
                .values(
                    data.into_iter()
                        .map(|(k, v)| (Name(k.clone()), SimpleExpr::from(v))),
                )
                .cond_where(Self::filter_to_cond(filter))
                .to_owned()
                .build_sqlx(PostgresQueryBuilder)
        };

        sqlx::query_with(&sql, values).execute(executor).await?;
        Ok(())
    }

    async fn delete_via(&self, executor: impl PgExecutor<'_>, filter: &F) -> Result<()> {
        let (sql, values) = {
            let mut sql = SeaQuery::delete()
                .from_table(Name(self.table.clone()))
                .to_owned();
            sql.cond_where(Self::filter_to_cond(filter));
            sql.build_sqlx(PostgresQueryBuilder)
        };

        sqlx::query_with(&sql, values).execute(executor).await?;
        Ok(())
    }

    async fn add_via(&self, executor: impl PgExecutor<'_>, entity: &T) -> Result<()> {
        let (sql, values) = {
            let data = (self.dump)(entity);
            let keys: Vec<Name> = data.keys().map(|k| Name(k.clone())).collect();

            SeaQuery::insert()
                .into_table(Name(self.table.clone()))
                .columns(keys)
                .values_panic(data.into_values())
                .to_owned()
                .build_sqlx(PostgresQueryBuilder)
        };

        sqlx::query_with(&sql, values).execute(executor).await?;
        Ok(())
    }

    async fn exists_via(&self, executor: impl PgExecutor<'_>, filter: &F) -> Result<bool> {
        let (sql, values) = {
            let mut inner_query = (self.query)(&self.table);
            self.apply_filter(&mut inner_query, filter);
            let query = SeaQuery::select()
                .expr_as(Expr::exists(inner_query), Alias::new("result"))
                .to_owned();
            query.build_sqlx(PostgresQueryBuilder)
        };

        let result = sqlx::query_with(&sql, values).fetch_one(executor).await;

        match result {
            Ok(row) => Ok(row.get("result")),
            Err(err) => bail!(err),
        }
    }
}

#[async_trait]
impl<'pool, T> Repo<T> for PgRepo<'pool, T>
where
    T: Sync + Send,
{
    type Transaction = PgTransaction<'pool>;

    async fn get(&self, filter: &F) -> Result<Option<T>> {
        self.get_via(self.pool, filter, false).await
    }

    async fn get_many(&self, query: &Query) -> Result<Vec<T>> {
        self.get_many_via(self.pool, query).await
    }

    async fn update(&self, filter: &F, entity: &T) -> Result<()> {
        self.update_via(self.pool, filter, entity).await
    }

    async fn delete(&self, filter: &F) -> Result<()> {
        self.delete_via(self.pool, filter).await
    }

    async fn add(&self, entity: &T) -> Result<()> {
        self.add_via(self.pool, entity).await
    }

    async fn exists(&self, filter: &F) -> Result<bool> {
        self.exists_via(self.pool, filter).await
    }

    async fn get_for_update(
        &self,
        transaction: &mut Self::Transaction,
        filter: &F,
    ) -> Result<Option<T>> {
        self.get_via(&mut transaction.wrapped, filter, true).await
    }

    async fn get_within(
        &self,
        transaction: &mut Self::Transaction,
        filter: &F,
    ) -> Result<Option<T>> {
        self.get_via(&mut transaction.wrapped, filter, false).await
    }

    async fn get_many_within(
        &self,
        transaction: &mut Self::Transaction,
        query: &Query,
    ) -> Result<Vec<T>> {
        self.get_many_via(&mut transaction.wrapped, query).await
    }

    async fn update_within(
        &self,
        transaction: &mut Self::Transaction,
        filter: &F,
        entity: &T,
    ) -> Result<()> {
        self.update_via(&mut transaction.wrapped, filter, entity)
            .await
    }

    async fn delete_within(&self, transaction: &mut Self::Transaction, filter: &F) -> Result<()> {
        self.delete_via(&mut transaction.wrapped, filter).await
    }

    async fn add_within(&self, transaction: &mut Self::Transaction, entity: &T) -> Result<()> {
        self.add_via(&mut transaction.wrapped, entity).await
    }

    async fn exists_within(&self, transaction: &mut Self::Transaction, filter: &F) -> Result<bool> {
        self.exists_via(&mut transaction.wrapped, filter).await
    }
}

pub struct PgTransaction<'a> {
    wrapped: sqlx::Transaction<'a, Postgres>,
}

pub struct PgTxManager<'pool> {
    pool: &'pool PgPool,
}

impl<'pool> PgTxManager<'pool> {
    pub fn new(pool: &'pool PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'pool> TxManager for PgTxManager<'pool> {
    type Transaction = PgTransaction<'pool>;

    async fn run<A, T>(&self, action: A) -> Result<T>
    where
        A: for<'a> FnOnce(
                &'a mut Self::Transaction,
            ) -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>
            + Send,
        T: Send,
    {
        let mut tx = Self::Transaction {
            wrapped: self.pool.begin().await?,
        };
        match action(&mut tx).await {
            Ok(res) => {
                tx.wrapped.commit().await?;
                Ok(res)
            }
            Err(err) => {
                tx.wrapped.rollback().await?;
                bail!(err)
            }
        }
    }
}
