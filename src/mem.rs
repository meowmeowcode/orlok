use std::cmp::Ordering;
use std::marker::PhantomData;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;

use crate::base::Repo;
use crate::query::{Op, Order, Query, F};

pub struct MemoryRepo<T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de>,
{
    items: RwLock<Vec<Value>>,
    phantom: PhantomData<T>,
}

impl<T> MemoryRepo<T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de>,
{
    pub fn new() -> Self {
        Self {
            items: RwLock::new(vec![]),
            phantom: PhantomData,
        }
    }

    fn load(item: Value) -> T {
        serde_json::from_value(item).unwrap()
    }
}

#[async_trait]
impl<T> Repo<T> for MemoryRepo<T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + std::marker::Sync + std::marker::Send,
{
    async fn get(&self, filter: &F) -> Option<T> {
        let items = self.items.read().await;
        let item = items.iter().find(|x| matches_filter(x, filter));

        match item {
            None => None,
            Some(i) => Some(Self::load(i.clone())),
        }
    }

    async fn get_many(&self, query: &Query) -> Vec<T> {
        let items = self.items.read().await;
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

        filtered.map(|x| Self::load(x.clone())).collect()
    }

    async fn add(&self, entity: &T) {
        self.items
            .write()
            .await
            .push(serde_json::to_value(entity).unwrap());
    }

    async fn delete(&self, filter: &F) {
        let mut items = self.items.write().await;

        while let Some((index, _)) = items
            .iter()
            .enumerate()
            .find(|(_, x)| matches_filter(x, filter))
        {
            items.remove(index);
        }
    }

    async fn update(&self, filter: &F, entity: &T) {
        let mut items = self.items.write().await;
        if let Some((index, _)) = items
            .iter()
            .enumerate()
            .find(|(_, x)| matches_filter(x, filter))
        {
            items[index] = serde_json::to_value(entity).unwrap();
        }
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
                _ => false,
            },
            Value::Number(n) => {
                let n = n.as_i64().unwrap();
                match op {
                    Op::IntEq(val) => n == *val,
                    Op::IntNe(val) => n != *val,
                    Op::IntLt(val) => n < *val,
                    Op::IntGt(val) => n > *val,
                    Op::IntLte(val) => n <= *val,
                    Op::IntGte(val) => n >= *val,
                    Op::IntBetween(val1, val2) => *val1 <= n && n <= *val2,
                    Op::IntIn(val) => val.contains(&n),
                    _ => false,
                }
            }
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
