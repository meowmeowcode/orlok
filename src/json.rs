use std::cmp::Ordering;
use std::collections::HashMap;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::str::FromStr;

use anyhow::{bail, Result};
use async_trait::async_trait;
use chrono::DateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::base::{Db, Repo};
use crate::query::{Op, Order, Query, F};

pub type JsonData = RwLock<HashMap<String, Vec<Value>>>;

#[derive(Clone)]
pub struct JsonRepo<T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de>,
{
    key: String,
    phantom: PhantomData<T>,
}

impl<T> JsonRepo<T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de>,
{
    pub fn new(key: String) -> Self {
        Self {
            key,
            phantom: PhantomData,
        }
    }

    fn load(item: Value) -> T {
        serde_json::from_value(item).unwrap()
    }
}

#[async_trait]
impl<T> Repo<T> for JsonRepo<T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + Sync + Send,
{
    type Db<'a> = JsonDb<'a>;

    async fn get<'a>(&self, db: &Self::Db<'a>, filter: &F) -> Result<Option<T>> {
        let mut lock = db.data.write().await;
        let items = lock.entry(self.key.clone()).or_insert(Vec::new());
        let item = items.iter().find(|x| matches_filter(x, filter));

        match item {
            None => Ok(None),
            Some(i) => Ok(Some(Self::load(i.clone()))),
        }
    }

    async fn get_many<'a>(&self, db: &Self::Db<'a>, query: &Query) -> Result<Vec<T>> {
        let mut lock = db.data.write().await;
        let items = lock.entry(self.key.clone()).or_insert(Vec::new());
        let mut sorted: Vec<&Value> = Vec::new();
        let mut filtered: Box<dyn Iterator<Item = &Value>> = Box::new(items.iter());

        if let Some(filter) = &query.filter {
            filtered = Box::new(filtered.filter(move |x| matches_filter(x, &filter.clone())))
        }

        if let Some(order) = &query.order {
            sorted.extend(filtered);
            sorted.sort_by(|x, y| {
                vals_cmp(&extract_fields(x, order), &extract_fields(y, order), order)
            });
            filtered = Box::new(sorted.iter().copied());
        }

        if let Some(offset) = query.offset {
            filtered = Box::new(filtered.skip(offset));
        }

        if let Some(limit) = query.limit {
            filtered = Box::new(filtered.take(limit));
        }

        Ok(filtered.map(|x| Self::load(x.clone())).collect())
    }

    async fn add<'a>(&self, db: &Self::Db<'a>, entity: &T) -> Result<()> {
        let item = serde_json::to_value(entity)?;
        let mut lock = db.data.write().await;
        let items = lock.entry(self.key.clone()).or_insert(Vec::new());
        items.push(item);
        Ok(())
    }

    async fn delete<'a>(&self, db: &Self::Db<'a>, filter: &F) -> Result<()> {
        let mut lock = db.data.write().await;
        let items = lock.entry(self.key.clone()).or_insert(Vec::new());

        while let Some((index, _)) = items
            .iter()
            .enumerate()
            .find(|(_, x)| matches_filter(x, filter))
        {
            items.remove(index);
        }

        Ok(())
    }

    async fn update<'a>(&self, db: &Self::Db<'a>, filter: &F, entity: &T) -> Result<()> {
        let mut lock = db.data.write().await;
        let items = lock.entry(self.key.clone()).or_insert(Vec::new());
        let item = serde_json::to_value(entity)?;

        if let Some((index, _)) = items
            .iter()
            .enumerate()
            .find(|(_, x)| matches_filter(x, filter))
        {
            items[index] = item;
        }

        Ok(())
    }

    async fn exists<'a>(&self, db: &Self::Db<'a>, filter: &F) -> Result<bool> {
        let entity = self.get(db, filter).await?;
        Ok(entity.is_some())
    }

    async fn count<'a>(&self, db: &Self::Db<'a>, filter: &F) -> Result<i64> {
        let mut lock = db.data.write().await;
        let items = lock.entry(self.key.clone()).or_insert(Vec::new());
        Ok(items.iter().filter(|x| matches_filter(x, filter)).count() as i64)
    }

    async fn count_all<'a>(&self, db: &Self::Db<'a>) -> Result<i64> {
        let mut lock = db.data.write().await;
        let items = lock.entry(self.key.clone()).or_insert(Vec::new());
        Ok(items.len() as i64)
    }

    async fn get_for_update<'a>(&self, db: &Self::Db<'a>, filter: &F) -> Result<Option<T>> {
        self.get(db, filter).await
    }
}

fn matches_filter(v: &Value, f: &F) -> bool {
    match f {
        F::And(filters) => matches_all_filters(v, filters),
        F::Or(filters) => matches_any_filter(v, filters),
        F::IsNone(field) => v[field].is_null(),
        F::Value { field, op } => match &v[field] {
            Value::String(s) => match op {
                Op::StrEq(val) => *s == *val,
                Op::StrNe(val) => *s != *val,
                Op::StrContains(val) => s.contains(val),
                Op::StrStartsWith(val) => s.starts_with(val),
                Op::StrEndsWith(val) => s.ends_with(val),
                Op::StrIn(val) => val.contains(s),
                Op::DateTimeEq(val) => DateTime::parse_from_rfc3339(&s).unwrap() == *val,
                Op::DateTimeNe(val) => DateTime::parse_from_rfc3339(&s).unwrap() != *val,
                Op::DateTimeLt(val) => DateTime::parse_from_rfc3339(&s).unwrap() < *val,
                Op::DateTimeGt(val) => DateTime::parse_from_rfc3339(&s).unwrap() > *val,
                Op::DateTimeLte(val) => DateTime::parse_from_rfc3339(&s).unwrap() <= *val,
                Op::DateTimeGte(val) => DateTime::parse_from_rfc3339(&s).unwrap() >= *val,
                Op::DecimalEq(val) => Decimal::from_str(s).unwrap() == *val,
                Op::DecimalNe(val) => Decimal::from_str(s).unwrap() != *val,
                Op::DecimalLt(val) => Decimal::from_str(s).unwrap() < *val,
                Op::DecimalGt(val) => Decimal::from_str(s).unwrap() > *val,
                Op::DecimalLte(val) => Decimal::from_str(s).unwrap() <= *val,
                Op::DecimalGte(val) => Decimal::from_str(s).unwrap() >= *val,
                Op::UuidEq(val) => Uuid::parse_str(s).unwrap() == *val,
                Op::UuidNe(val) => Uuid::parse_str(s).unwrap() != *val,
                Op::UuidIn(val) => val.contains(&Uuid::parse_str(s).unwrap()),
                _ => false,
            },
            Value::Number(n) => match op {
                Op::IntEq(val) => {
                    if let Some(n) = n.as_i64() {
                        n == *val
                    } else {
                        false
                    }
                }
                Op::IntNe(val) => {
                    if let Some(n) = n.as_i64() {
                        n != *val
                    } else {
                        false
                    }
                }
                Op::IntLt(val) => {
                    if let Some(n) = n.as_i64() {
                        n < *val
                    } else {
                        false
                    }
                }
                Op::IntGt(val) => {
                    if let Some(n) = n.as_i64() {
                        n > *val
                    } else {
                        false
                    }
                }
                Op::IntLte(val) => {
                    if let Some(n) = n.as_i64() {
                        n <= *val
                    } else {
                        false
                    }
                }
                Op::IntGte(val) => {
                    if let Some(n) = n.as_i64() {
                        n >= *val
                    } else {
                        false
                    }
                }
                Op::IntBetween(val1, val2) => {
                    if let Some(n) = n.as_i64() {
                        *val1 <= n && n <= *val2
                    } else {
                        false
                    }
                }
                Op::IntIn(val) => {
                    if let Some(n) = n.as_i64() {
                        val.contains(&n)
                    } else {
                        false
                    }
                }
                Op::FloatEq(val) => {
                    if let Some(n) = n.as_f64() {
                        n == *val
                    } else {
                        false
                    }
                }
                Op::FloatNe(val) => {
                    if let Some(n) = n.as_f64() {
                        n != *val
                    } else {
                        false
                    }
                }
                Op::FloatLt(val) => {
                    if let Some(n) = n.as_f64() {
                        n < *val
                    } else {
                        false
                    }
                }
                Op::FloatGt(val) => {
                    if let Some(n) = n.as_f64() {
                        n > *val
                    } else {
                        false
                    }
                }
                Op::FloatLte(val) => {
                    if let Some(n) = n.as_f64() {
                        n <= *val
                    } else {
                        false
                    }
                }
                Op::FloatGte(val) => {
                    if let Some(n) = n.as_f64() {
                        n >= *val
                    } else {
                        false
                    }
                }
                _ => false,
            },
            Value::Bool(b) => match op {
                Op::BoolEq(val) => b == val,
                Op::BoolNe(val) => b != val,
                _ => false,
            },
            _ => false,
        },
    }
}

fn matches_all_filters(v: &Value, filters: &[F]) -> bool {
    filters.iter().all(|f| matches_filter(&v, f))
}

fn matches_any_filter(v: &Value, filters: &[F]) -> bool {
    filters.iter().any(|f| matches_filter(&v, f))
}

fn extract_fields<'a>(v: &'a Value, order: &Vec<Order>) -> Vec<&'a Value> {
    order
        .iter()
        .map(|f| match f {
            Order::Asc(field) => field,
            Order::Desc(field) => field,
        })
        .map(|f| &v[f])
        .collect()
}

fn val_cmp(x: &Value, y: &Value, order: &Order) -> Ordering {
    let result = match (x, y) {
        (Value::String(s1), Value::String(s2)) => s1.as_str().cmp(&s2.as_str()),
        (Value::Number(n1), Value::Number(n2)) => n1.as_i64().unwrap().cmp(&n2.as_i64().unwrap()),
        _ => todo!(),
    };

    match order {
        Order::Asc(_) => result,
        Order::Desc(_) => match result {
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
            _ => result,
        },
    }
}

fn vals_cmp(xs: &Vec<&Value>, ys: &Vec<&Value>, fields: &Vec<Order>) -> Ordering {
    for ((x, y), order) in xs.iter().zip(ys.iter()).zip(fields.iter()) {
        match val_cmp(x, y, order) {
            Ordering::Greater => return Ordering::Greater,
            Ordering::Less => return Ordering::Less,
            _ => continue,
        }
    }

    Ordering::Equal
}

pub struct JsonTransaction {}

pub struct JsonDb<'a> {
    data: JsonData,
    phantom: PhantomData<&'a ()>,
}

impl<'a> JsonDb<'a> {
    pub fn new(data: JsonData) -> Self {
        Self {
            data,
            phantom: PhantomData,
        }
    }
}

#[async_trait]
impl Db for JsonDb<'_> {
    async fn transaction<A, T>(&self, action: A) -> Result<T>
    where
        A: for<'a> FnOnce(&'a Self) -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>> + Send,
        T: Send,
    {
        let initial_state = self.data.read().await.clone();

        match action(&self).await {
            Ok(res) => Ok(res),
            Err(err) => {
                let mut data = self.data.write().await;
                data.clear();
                data.extend(initial_state);
                bail!(err)
            }
        }
    }
}
