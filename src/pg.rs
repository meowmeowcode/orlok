use std::fmt::Write;

use async_trait::async_trait;
use sea_query::backend::PostgresQueryBuilder;
use sea_query::{
    Condition, IntoCondition, Order as SeaOrder, Query as SeaQuery, SelectStatement, SimpleExpr,
};
use sea_query::{Expr, Iden};
use sqlx::postgres::PgRow;
use sqlx::PgPool;
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::base::Repo;
use crate::query::{Op, Order, Query, F};

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
                    _ => todo!(),
                }
                .into_condition()
            }
            _ => todo!(),
        }
    }
}

#[async_trait]
impl<'pool, T> Repo<T> for PgRepo<'pool, T>
where
    T: Sync + Send,
{
    async fn get(&self, filter: &F) -> Option<T> {
        let query = {
            let mut query = (self.query)(&self.table);
            self.apply_filter(&mut query, filter);
            query.to_string(PostgresQueryBuilder)
        };

        let result = sqlx::query(&query).fetch_one(self.pool).await;

        match result {
            Ok(row) => Some((self.load)(&row)),
            Err(sqlx::Error::RowNotFound) => None,
            Err(err) => panic!("problem executing a query {:?}", err),
        }
    }

    async fn get_many(&self, query: &Query) -> Vec<T> {
        let sql = {
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

            sql.to_string(PostgresQueryBuilder)
        };

        let result = sqlx::query(&sql).fetch_all(self.pool).await;

        match result {
            Ok(rows) => rows.iter().map(self.load).collect(),
            Err(err) => panic!("problem executing a query {:?}", err),
        }
    }

    async fn update(&self, filter: &F, entity: &T) {
        let sql = {
            let data = (self.dump)(entity);

            SeaQuery::update()
                .table(Name(self.table.clone()))
                .values(
                    data.into_iter()
                        .map(|(k, v)| (Name(k.clone()), SimpleExpr::from(v))),
                )
                .cond_where(Self::filter_to_cond(filter))
                .to_owned()
                .to_string(PostgresQueryBuilder)
        };

        sqlx::query(&sql).execute(self.pool).await.unwrap();
    }

    async fn delete(&self, filter: &F) {
        let sql = {
            let mut sql = SeaQuery::delete()
                .from_table(Name(self.table.clone()))
                .to_owned();
            sql.cond_where(Self::filter_to_cond(filter));
            sql.to_string(PostgresQueryBuilder)
        };

        sqlx::query(&sql).execute(self.pool).await.unwrap();
    }

    async fn add(&self, entity: &T) {
        let query = {
            let data = (self.dump)(entity);
            let keys: Vec<Name> = data.keys().map(|k| Name(k.clone())).collect();

            SeaQuery::insert()
                .into_table(Name(self.table.clone()))
                .columns(keys)
                .values_panic(data.into_values())
                .to_owned()
                .to_string(PostgresQueryBuilder)
        };

        sqlx::query(&query).execute(self.pool).await.unwrap();
    }
}
