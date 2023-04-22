use chrono::TimeZone;
use std::collections::HashMap;

use async_once::AsyncOnce;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sea_query::SimpleExpr;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

use orlok::base::Repo;
use orlok::mem::MemoryRepo;
use orlok::pg::PgRepo;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub is_evil: bool,
    pub weight: Option<f64>,
    pub registered_at: DateTime<Utc>,
    pub money: Decimal,
}

impl User {
    pub fn new(id: i64, name: &str) -> Self {
        Self {
            id: id,
            name: name.to_string(),
            is_evil: false,
            weight: None,
            money: dec!(0.0),
            registered_at: Utc::now(),
        }
    }
}

pub async fn add_alice(repo: &Box<dyn Repo<User>>) -> User {
    let mut u = User::new(1, "Alice");
    u.money = dec!(130.50);
    u.registered_at = Utc.with_ymd_and_hms(2018, 3, 1, 0, 0, 0).unwrap();
    u.weight = Some(70.5);
    repo.add(&u).await.unwrap();
    u
}

pub async fn add_bob(repo: &Box<dyn Repo<User>>) -> User {
    let mut u = User::new(2, "Bob");
    u.money = dec!(150.06);
    u.registered_at = Utc.with_ymd_and_hms(2019, 2, 2, 0, 0, 0).unwrap();
    u.weight = Some(83.4);
    repo.add(&u).await.unwrap();
    u
}

pub async fn add_eve(repo: &Box<dyn Repo<User>>) -> User {
    let mut u = User::new(3, "Eve");
    u.money = dec!(230.25);
    u.registered_at = Utc.with_ymd_and_hms(2020, 1, 3, 0, 0, 0).unwrap();
    u.is_evil = true;
    repo.add(&u).await.unwrap();
    u
}

fn dump_user(entity: &User) -> HashMap<String, SimpleExpr> {
    HashMap::from([
        ("id".to_string(), entity.id.into()),
        ("name".to_string(), entity.name.clone().into()),
        ("is_evil".to_string(), entity.is_evil.into()),
        ("weight".to_string(), entity.weight.into()),
        ("money".to_string(), entity.money.into()),
        ("registered_at".to_string(), entity.registered_at.into()),
    ])
}

fn load_user(row: &PgRow) -> User {
    User {
        id: row.get("id"),
        name: row.get("name"),
        is_evil: row.get("is_evil"),
        weight: row.get("weight"),
        money: row.get("money"),
        registered_at: row.get("registered_at"),
    }
}

async fn users_pg_repo<'a>() -> PgRepo<'a, User> {
    let pool = POOL.get().await;

    sqlx::query(
        "create table if not exists users (
            id bigint,
            name text,
            is_evil boolean,
            weight float8,
            money decimal,
            registered_at timestamptz
        )"
    )
        .execute(pool)
        .await
        .unwrap();

    sqlx::query("delete from users")
        .execute(pool)
        .await
        .unwrap();

    PgRepo::new(pool, "users".to_string(), dump_user, load_user)
}

lazy_static! {
    static ref POOL: AsyncOnce<PgPool> = AsyncOnce::new(async {
        PgPool::connect("postgresql://orlok:orlok@localhost/orlok")
            .await
            .unwrap()
    });
}

pub async fn repos() -> Vec<Box<dyn Repo<User>>> {
    vec![Box::new(MemoryRepo::new()), Box::new(users_pg_repo().await)]
}
