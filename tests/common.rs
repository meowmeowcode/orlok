use std::collections::HashMap;

use async_once::AsyncOnce;
use lazy_static::lazy_static;
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
}

impl User {
    pub fn new(id: i64, name: &str) -> Self {
        Self {
            id: id,
            name: name.to_string(),
        }
    }
}

pub async fn add_alice(repo: &Box<dyn Repo<User>>) -> User {
    let u = User::new(1, "Alice");
    repo.add(&u).await.unwrap();
    u
}

pub async fn add_bob(repo: &Box<dyn Repo<User>>) -> User {
    let u = User::new(2, "Bob");
    repo.add(&u).await.unwrap();
    u
}

pub async fn add_eve(repo: &Box<dyn Repo<User>>) -> User {
    let u = User::new(3, "Eve");
    repo.add(&u).await.unwrap();
    u
}

fn dump_user(entity: &User) -> HashMap<String, SimpleExpr> {
    HashMap::from([
        ("id".to_string(), entity.id.into()),
        ("name".to_string(), entity.name.clone().into()),
    ])
}

fn load_user(row: &PgRow) -> User {
    User {
        id: row.get("id"),
        name: row.get("name"),
    }
}

async fn users_pg_repo<'a>() -> PgRepo<'a, User> {
    let pool = POOL.get().await;

    sqlx::query("create table if not exists users (id bigint, name text)")
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
