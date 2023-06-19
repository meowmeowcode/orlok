//! In-memory repository implementation.
use std::cmp::Ordering;
use std::collections::HashMap;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::str::FromStr;

use anyhow::{bail, Error, Result};
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::base::{Db, Repo};
use crate::query::{Op, Order, Query, F};

/// Repository that stores entities as an in-memory collection
/// of JSON objects.
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
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            phantom: PhantomData,
        }
    }

    fn load(item: Value) -> Result<T> {
        Ok(serde_json::from_value(item)?)
    }

    fn find_index(items: &Vec<Value>, filter: &F) -> Result<Option<usize>> {
        for (index, item) in items.iter().enumerate() {
            if matches_filter(item, filter)? {
                return Ok(Some(index));
            }
        }
        Ok(None)
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
        for item in items {
            if matches_filter(item, filter)? {
                return Ok(Some(Self::load(item.clone())?));
            }
        }
        Ok(None)
    }

    async fn get_many<'a>(&self, db: &Self::Db<'a>, query: &Query) -> Result<Vec<T>> {
        let mut lock = db.data.write().await;
        let items = lock.entry(self.key.clone()).or_insert(Vec::new());
        let mut sorted: Vec<&Value> = Vec::new();
        let mut result: Box<dyn Iterator<Item = &Value>> = Box::new(items.iter());

        if let Some(filter) = &query.filter {
            result = Box::new(
                result
                    .try_fold(Vec::new(), move |mut acc, x| {
                        if matches_filter(x, &filter.clone())? {
                            acc.push(x);
                        }
                        Ok::<Vec<&Value>, Error>(acc)
                    })?
                    .into_iter(),
            );
        }

        if let Some(order) = &query.order {
            sorted.extend(result);
            sorted.sort_by(|x, y| {
                vals_cmp(&extract_fields(x, order), &extract_fields(y, order), order)
            });
            result = Box::new(sorted.iter().copied());
        }

        if let Some(offset) = query.offset {
            result = Box::new(result.skip(offset));
        }

        if let Some(limit) = query.limit {
            result = Box::new(result.take(limit));
        }

        Ok(result.try_fold(Vec::new(), |mut acc, x| {
            acc.push(Self::load(x.clone())?);
            Ok::<Vec<T>, Error>(acc)
        })?)
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

        while let Some(index) = Self::find_index(items, filter)? {
            items.remove(index);
        }

        Ok(())
    }

    async fn update<'a>(&self, db: &Self::Db<'a>, filter: &F, entity: &T) -> Result<()> {
        let mut lock = db.data.write().await;
        let items = lock.entry(self.key.clone()).or_insert(Vec::new());
        let item = serde_json::to_value(entity)?;

        if let Some(index) = Self::find_index(items, filter)? {
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
        let mut count = 0;

        for item in items {
            if matches_filter(item, filter)? {
                count += 1
            }
        }

        Ok(count)
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

fn matches_filter(v: &Value, f: &F) -> Result<bool> {
    Ok(match f {
        F::And(filters) => matches_all_filters(v, filters)?,
        F::Or(filters) => matches_any_filter(v, filters)?,
        F::Not(filter) => !matches_filter(v, filter)?,
        F::IsNone(field) => v[field].is_null(),
        F::Value { field, op } => {
            if let Some(val) = v.get(field) {
                if val.is_null() {
                    return Ok(false);
                }
                match op {
                    Op::StrEq(arg) => extract_string(val)? == arg,
                    Op::StrNe(arg) => extract_string(val)? != arg,
                    Op::StrContains(arg) => extract_string(val)?.contains(arg),
                    Op::StrStartsWith(arg) => extract_string(val)?.starts_with(arg),
                    Op::StrEndsWith(arg) => extract_string(val)?.ends_with(arg),
                    Op::StrIn(arg) => arg.contains(extract_string(val)?),
                    Op::DateTimeEq(arg) => extract_date_time(val)? == *arg,
                    Op::DateTimeNe(arg) => extract_date_time(val)? != *arg,
                    Op::DateTimeLt(arg) => extract_date_time(val)? < *arg,
                    Op::DateTimeGt(arg) => extract_date_time(val)? > *arg,
                    Op::DateTimeLte(arg) => extract_date_time(val)? <= *arg,
                    Op::DateTimeGte(arg) => extract_date_time(val)? >= *arg,
                    Op::DecimalEq(arg) => extract_decimal(val)? == *arg,
                    Op::DecimalNe(arg) => extract_decimal(val)? != *arg,
                    Op::DecimalLt(arg) => extract_decimal(val)? < *arg,
                    Op::DecimalGt(arg) => extract_decimal(val)? > *arg,
                    Op::DecimalLte(arg) => extract_decimal(val)? <= *arg,
                    Op::DecimalGte(arg) => extract_decimal(val)? >= *arg,
                    Op::UuidEq(arg) => extract_uuid(val)? == *arg,
                    Op::UuidNe(arg) => extract_uuid(val)? != *arg,
                    Op::UuidIn(arg) => arg.contains(&extract_uuid(val)?),
                    Op::IntEq(arg) => extract_int(val)? == *arg,
                    Op::IntNe(arg) => extract_int(val)? != *arg,
                    Op::IntLt(arg) => extract_int(val)? < *arg,
                    Op::IntGt(arg) => extract_int(val)? > *arg,
                    Op::IntLte(arg) => extract_int(val)? <= *arg,
                    Op::IntGte(arg) => extract_int(val)? >= *arg,
                    Op::IntBetween(arg1, arg2) => {
                        let n = extract_int(val)?;
                        *arg1 <= n && n <= *arg2
                    }
                    Op::IntIn(arg) => arg.contains(&extract_int(val)?),
                    Op::FloatEq(arg) => extract_float(val)? == *arg,
                    Op::FloatNe(arg) => extract_float(val)? != *arg,
                    Op::FloatLt(arg) => extract_float(val)? < *arg,
                    Op::FloatGt(arg) => extract_float(val)? > *arg,
                    Op::FloatLte(arg) => extract_float(val)? <= *arg,
                    Op::FloatGte(arg) => extract_float(val)? >= *arg,
                    Op::BoolEq(arg) => extract_bool(val)? == *arg,
                    Op::BoolNe(arg) => extract_bool(val)? != *arg,
                }
            } else {
                bail!("Unknown field {}", field)
            }
        }
    })
}

fn matches_all_filters(v: &Value, filters: &[F]) -> Result<bool> {
    for f in filters {
        if !matches_filter(&v, f)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn matches_any_filter(v: &Value, filters: &[F]) -> Result<bool> {
    for f in filters {
        if matches_filter(&v, f)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn extract_string(v: &Value) -> Result<&String> {
    if let Value::String(s) = v {
        Ok(s)
    } else {
        bail!("{:?} is not a string", v)
    }
}

fn extract_date_time(v: &Value) -> Result<DateTime<FixedOffset>> {
    let s = extract_string(v)?;
    Ok(DateTime::parse_from_rfc3339(&s)?)
}

fn extract_decimal(v: &Value) -> Result<Decimal> {
    let s = extract_string(v)?;
    Ok(Decimal::from_str(s)?)
}

fn extract_uuid(v: &Value) -> Result<Uuid> {
    let s = extract_string(v)?;
    Ok(Uuid::parse_str(s)?)
}

fn extract_int(v: &Value) -> Result<i64> {
    if let Value::Number(n) = v {
        if let Some(n) = n.as_i64() {
            Ok(n)
        } else {
            bail!("Cannot convert {:?} to an f64", n);
        }
    } else {
        bail!("{:?} is not a number", v);
    }
}

fn extract_float(v: &Value) -> Result<f64> {
    if let Value::Number(n) = v {
        if let Some(n) = n.as_f64() {
            Ok(n)
        } else {
            bail!("Cannot convert {:?} to f64", n);
        }
    } else {
        bail!("{:?} is not a number", v);
    }
}

fn extract_bool(v: &Value) -> Result<bool> {
    if let Value::Bool(b) = v {
        Ok(*b)
    } else {
        bail!("{:?} is not a bool", v);
    }
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
        (Value::Null, Value::Null) => Ordering::Equal,
        (Value::Null, _) => Ordering::Less,
        (_, Value::Null) => Ordering::Greater,
        (Value::String(s1), Value::String(s2)) => s1.cmp(&s2),
        (Value::Number(n1), Value::Number(n2)) => {
            n1.as_f64().unwrap().total_cmp(&n2.as_f64().unwrap())
        }
        (Value::Bool(b1), Value::Bool(b2)) => b1.cmp(b2),
        _ => Ordering::Equal,
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

/// Struct that contains a collection of JSON objects.
pub struct JsonDb<'a> {
    data: RwLock<HashMap<String, Vec<Value>>>,
    phantom: PhantomData<&'a ()>,
}

impl<'a> JsonDb<'a> {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
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
