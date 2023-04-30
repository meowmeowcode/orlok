mod common;

use std::collections::HashMap;

use async_once::AsyncOnce;
use sea_query::SimpleExpr;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

use lazy_static::lazy_static;

use orlok::base::{Repo, TxManager};
use orlok::pg::{PgRepo, PgTxManager};
use orlok::query::{Order, Query, F};

use common::User;

lazy_static! {
    static ref POOL: AsyncOnce<PgPool> = AsyncOnce::new(async {
        PgPool::connect("postgresql://orlok:orlok@localhost/orlok")
            .await
            .unwrap()
    });
}

fn dump_user(entity: &User) -> HashMap<String, SimpleExpr> {
    HashMap::from([
        ("id".to_string(), entity.id.into()),
        ("name".to_string(), entity.name.clone().into()),
        ("age".to_string(), entity.age.into()),
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
        age: row.get("age"),
        is_evil: row.get("is_evil"),
        weight: row.get("weight"),
        money: row.get("money"),
        registered_at: row.get("registered_at"),
    }
}

pub async fn users_repo<'a>() -> PgRepo<'a, User> {
    let pool = POOL.get().await;

    sqlx::query(
        "create table if not exists users (
            id uuid,
            name text,
            age bigint,
            is_evil boolean,
            weight float8,
            money decimal,
            registered_at timestamptz
        )",
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

pub async fn tx_manager<'a>() -> PgTxManager<'a> {
    PgTxManager::new(POOL.get().await)
}

#[tokio::test]
async fn get() {
    let repo = users_repo().await;
    let alice = common::add_alice(&repo).await;
    let bob = common::add_bob(&repo).await;
    let result = repo.get(&F::eq("name", "Alice")).await.unwrap().unwrap();
    assert_eq!(result, alice);
    let result = repo.get(&F::eq("id", bob.id)).await.unwrap().unwrap();
    assert_eq!(result, bob);
}

#[tokio::test]
async fn get_none() {
    let repo = users_repo().await;
    common::add_alice(&repo).await;
    let result = repo.get(&F::eq("name", "test")).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn exists() {
    let repo = users_repo().await;
    assert_eq!(repo.exists(&F::eq("name", "Alice")).await.unwrap(), false);
    common::add_alice(&repo).await;
    assert_eq!(repo.exists(&F::eq("name", "Alice")).await.unwrap(), true);
}

#[tokio::test]
async fn count() {
    let repo = users_repo().await;
    let filter = F::or(&[F::eq("name", "Bob"), F::eq("name", "Alice")]);
    assert_eq!(repo.count(&filter).await.unwrap(), 0);
    common::add_alice(&repo).await;
    common::add_bob(&repo).await;
    common::add_eve(&repo).await;
    assert_eq!(repo.count(&filter).await.unwrap(), 2);
}

#[tokio::test]
async fn count_all() {
    let repo = users_repo().await;
    assert_eq!(repo.count_all().await.unwrap(), 0);
    common::add_alice(&repo).await;
    common::add_bob(&repo).await;
    common::add_eve(&repo).await;
    assert_eq!(repo.count_all().await.unwrap(), 3);
}

#[tokio::test]
async fn delete() {
    let repo = users_repo().await;
    common::add_alice(&repo).await;
    common::add_bob(&repo).await;
    let eve = common::add_eve(&repo).await;
    repo.delete(&F::or(&[F::eq("name", "Bob"), F::eq("name", "Alice")]))
        .await
        .unwrap();
    let users = repo.get_many(&Query::new()).await.unwrap();
    assert_eq!(users, vec![eve]);
}

#[tokio::test]
async fn update() {
    let repo = users_repo().await;
    let alice = common::add_alice(&repo).await;
    let mut bob = common::add_bob(&repo).await;
    let eve = common::add_eve(&repo).await;
    bob.name = "Robert".to_string();
    repo.update(&F::eq("id", bob.id), &bob).await.unwrap();
    let users = repo
        .get_many(&Query::new().order(vec![Order::Asc("name".to_string())]))
        .await
        .unwrap();
    assert_eq!(users, vec![alice, eve, bob]);
}

#[tokio::test]
async fn get_many() {
    let repo = users_repo().await;
    let alice = common::add_alice(&repo).await;
    let bob = common::add_bob(&repo).await;
    let eve = common::add_eve(&repo).await;

    let cases = [
        // and, or:
        (
            Query::filter(F::and(&[F::gt("age", alice.age), F::lt("age", eve.age)])),
            vec![&bob],
        ),
        (
            Query::filter(F::or(&[F::eq("age", alice.age), F::eq("age", eve.age)])),
            vec![&alice, &eve],
        ),
        // i64 filters:
        (Query::filter(F::ne("age", bob.age)), vec![&alice, &eve]),
        (Query::filter(F::lt("age", eve.age)), vec![&alice, &bob]),
        (Query::filter(F::gt("age", alice.age)), vec![&bob, &eve]),
        (Query::filter(F::lte("age", bob.age)), vec![&alice, &bob]),
        (Query::filter(F::gte("age", bob.age)), vec![&bob, &eve]),
        (
            Query::filter(F::in_("age", vec![alice.age, eve.age])),
            vec![&alice, &eve],
        ),
        (
            Query::filter(F::between("age", (alice.age, bob.age))),
            vec![&alice, &bob],
        ),
        // f64 filters
        (
            Query::filter(F::eq("weight", bob.weight.unwrap())),
            vec![&bob],
        ),
        (
            Query::filter(F::ne("weight", bob.weight.unwrap())),
            vec![&alice],
        ),
        (
            Query::filter(F::lt("weight", bob.weight.unwrap())),
            vec![&alice],
        ),
        (
            Query::filter(F::gt("weight", alice.weight.unwrap())),
            vec![&bob],
        ),
        (
            Query::filter(F::lte("weight", bob.weight.unwrap())),
            vec![&alice, &bob],
        ),
        (
            Query::filter(F::gte("weight", alice.weight.unwrap())),
            vec![&alice, &bob],
        ),
        // string filters:
        (Query::filter(F::contains("name", "o")), vec![&bob]),
        (Query::filter(F::starts_with("name", "E")), vec![&eve]),
        (Query::filter(F::ends_with("name", "e")), vec![&alice, &eve]),
        // bool filters:
        (Query::filter(F::eq("is_evil", true)), vec![&eve]),
        (Query::filter(F::eq("is_evil", false)), vec![&alice, &bob]),
        // datetime filters:
        (
            Query::filter(F::eq("registered_at", bob.registered_at)),
            vec![&bob],
        ),
        (
            Query::filter(F::ne("registered_at", bob.registered_at)),
            vec![&alice, &eve],
        ),
        (
            Query::filter(F::lt("registered_at", bob.registered_at)),
            vec![&alice],
        ),
        (
            Query::filter(F::gt("registered_at", bob.registered_at)),
            vec![&eve],
        ),
        (
            Query::filter(F::lte("registered_at", bob.registered_at)),
            vec![&alice, &bob],
        ),
        (
            Query::filter(F::gte("registered_at", bob.registered_at)),
            vec![&bob, &eve],
        ),
        // decimal filters:
        (Query::filter(F::eq("money", bob.money)), vec![&bob]),
        (Query::filter(F::ne("money", bob.money)), vec![&alice, &eve]),
        (Query::filter(F::lt("money", bob.money)), vec![&alice]),
        (Query::filter(F::gt("money", bob.money)), vec![&eve]),
        (
            Query::filter(F::lte("money", bob.money)),
            vec![&alice, &bob],
        ),
        (Query::filter(F::gte("money", bob.money)), vec![&bob, &eve]),
        // uuid filters:
        (Query::filter(F::eq("id", bob.id)), vec![&bob]),
        (Query::filter(F::ne("id", bob.id)), vec![&alice, &eve]),
        (
            Query::filter(F::in_("id", vec![alice.id, eve.id])),
            vec![&alice, &eve],
        ),
        // offset, limit, order:
        (Query::new().limit(2).clone(), vec![&alice, &bob]),
        (Query::new().offset(1).clone(), vec![&bob, &eve]),
        (
            Query::new()
                .order(vec![Order::Asc("name".to_string())])
                .clone(),
            vec![&alice, &bob, &eve],
        ),
        (
            Query::new()
                .order(vec![Order::Desc("name".to_string())])
                .clone(),
            vec![&eve, &bob, &alice],
        ),
    ];

    for (query, expected_result) in cases {
        let users = repo.get_many(&query).await.unwrap();
        let result: Vec<&User> = users.iter().collect();
        assert_eq!(result, expected_result, "filter {:?} doesn't work", query);
    }
}

#[tokio::test]
async fn transaction() {
    let tx_manager = tx_manager().await;
    let repo = users_repo().await;
    common::add_bob(&repo).await;

    tx_manager
        .run(|tx| {
            Box::pin({
                let repo = repo.clone();
                async move {
                    repo.delete_within(tx, &F::eq("name", "Bob")).await?;
                    Ok(())
                }
            })
        })
        .await
        .unwrap();

    let bob = repo.get(&F::eq("name", "Bob")).await.unwrap();
    assert!(bob.is_none());
}

#[tokio::test]
async fn transaction_rollback() {
    let tx_manager = tx_manager().await;
    let repo = users_repo().await;
    common::add_alice(&repo).await;

    let result = tx_manager
        .run::<_, anyhow::Error>(|tx| {
            Box::pin({
                let repo = repo.clone();
                async move {
                    repo.delete_within(tx, &F::eq("name", "Alice")).await?;
                    anyhow::bail!("failed transaction")
                }
            })
        })
        .await;

    assert!(result.is_err());
    let alice = repo.get(&F::eq("name", "Alice")).await.unwrap();
    assert!(alice.is_some());
}

#[tokio::test]
async fn get_for_update() {
    let tx_manager = tx_manager().await;
    let repo = users_repo().await;
    let user = common::add_bob(&repo).await;
    let new_name = "Robert";

    tx_manager
        .run(|tx| {
            Box::pin({
                let repo = repo.clone();
                async move {
                    let mut user = repo
                        .get_for_update(tx, &F::eq("id", user.id))
                        .await?
                        .unwrap();
                    user.name = new_name.to_string();
                    repo.update_within(tx, &F::eq("id", user.id), &user).await?;
                    Ok(())
                }
            })
        })
        .await
        .unwrap();

    let user = repo.get(&F::eq("id", user.id)).await.unwrap().unwrap();
    assert_eq!(user.name, new_name);
}
