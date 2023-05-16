mod common;

use std::collections::HashMap;

use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

use orlok::base::{Db, Repo};
use orlok::pg::{PgDb, PgRepo, Value};
use orlok::query::{Order, F, Q};

use common::User;

fn dump_user(entity: &User) -> HashMap<String, Value> {
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

pub async fn db<'a>() -> PgDb<'a> {
    let pool = PgPool::connect("postgresql://orlok:orlok@localhost/orlok")
        .await
        .unwrap();

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
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query("delete from test_users")
        .execute(&pool)
        .await
        .unwrap();

    PgDb::new(pool)
}

pub async fn users_repo() -> PgRepo<User> {
    PgRepo::new("test_users", dump_user, load_user)
}

#[tokio::test]
async fn get() {
    let db = db().await;
    let repo = users_repo().await;
    let alice = common::add_alice(&db, &repo).await;
    let bob = common::add_bob(&db, &repo).await;
    let result = repo
        .get(&db, &F::eq("name", "Alice"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result, alice);
    let result = repo.get(&db, &F::eq("id", bob.id)).await.unwrap().unwrap();
    assert_eq!(result, bob);
}

#[tokio::test]
async fn get_none() {
    let db = db().await;
    let repo = users_repo().await;
    common::add_alice(&db, &repo).await;
    let result = repo.get(&db, &F::eq("name", "test")).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn exists() {
    let db = db().await;
    let repo = users_repo().await;
    assert_eq!(
        repo.exists(&db, &F::eq("name", "Alice")).await.unwrap(),
        false
    );
    common::add_alice(&db, &repo).await;
    assert_eq!(
        repo.exists(&db, &F::eq("name", "Alice")).await.unwrap(),
        true
    );
}

#[tokio::test]
async fn count() {
    let db = db().await;
    let repo = users_repo().await;
    let filter = F::or(vec![F::eq("name", "Bob"), F::eq("name", "Alice")]);
    assert_eq!(repo.count(&db, &filter).await.unwrap(), 0);
    common::add_alice(&db, &repo).await;
    common::add_bob(&db, &repo).await;
    common::add_eve(&db, &repo).await;
    assert_eq!(repo.count(&db, &filter).await.unwrap(), 2);
}

#[tokio::test]
async fn count_all() {
    let db = db().await;
    let repo = users_repo().await;
    assert_eq!(repo.count_all(&db).await.unwrap(), 0);
    common::add_alice(&db, &repo).await;
    common::add_bob(&db, &repo).await;
    common::add_eve(&db, &repo).await;
    assert_eq!(repo.count_all(&db).await.unwrap(), 3);
}

#[tokio::test]
async fn delete() {
    let db = db().await;
    let repo = users_repo().await;
    common::add_alice(&db, &repo).await;
    common::add_bob(&db, &repo).await;
    let eve = common::add_eve(&db, &repo).await;
    repo.delete(
        &db,
        &F::or(vec![F::eq("name", "Bob"), F::eq("name", "Alice")]),
    )
    .await
    .unwrap();
    let users = repo.get_many(&db, &Q::new()).await.unwrap();
    assert_eq!(users, vec![eve]);
}

#[tokio::test]
async fn update() {
    let db = db().await;
    let repo = users_repo().await;
    let alice = common::add_alice(&db, &repo).await;
    let mut bob = common::add_bob(&db, &repo).await;
    let eve = common::add_eve(&db, &repo).await;
    bob.name = "Robert".to_string();
    repo.update(&db, &F::eq("id", bob.id), &bob).await.unwrap();
    let users = repo
        .get_many(&db, &Q::new().order(vec![Order::Asc("name".to_string())]))
        .await
        .unwrap();
    assert_eq!(users, vec![alice, eve, bob]);
}

#[tokio::test]
async fn get_many() {
    let db = db().await;
    let repo = users_repo().await;
    let alice = common::add_alice(&db, &repo).await;
    let bob = common::add_bob(&db, &repo).await;
    let eve = common::add_eve(&db, &repo).await;

    let cases = [
        // and, or:
        (
            Q::filter(F::and(vec![F::gt("age", alice.age), F::lt("age", eve.age)])),
            vec![&bob],
        ),
        (
            Q::filter(F::or(vec![F::eq("age", alice.age), F::eq("age", eve.age)])),
            vec![&alice, &eve],
        ),
        // none filter:
        (Q::filter(F::is_none("weight")), vec![&eve]),
        // i64 filters:
        (Q::filter(F::ne("age", bob.age)), vec![&alice, &eve]),
        (Q::filter(F::lt("age", eve.age)), vec![&alice, &bob]),
        (Q::filter(F::gt("age", alice.age)), vec![&bob, &eve]),
        (Q::filter(F::lte("age", bob.age)), vec![&alice, &bob]),
        (Q::filter(F::gte("age", bob.age)), vec![&bob, &eve]),
        (
            Q::filter(F::in_("age", vec![alice.age, eve.age])),
            vec![&alice, &eve],
        ),
        (
            Q::filter(F::between("age", (alice.age, bob.age))),
            vec![&alice, &bob],
        ),
        // f64 filters
        (Q::filter(F::eq("weight", bob.weight.unwrap())), vec![&bob]),
        (
            Q::filter(F::ne("weight", bob.weight.unwrap())),
            vec![&alice],
        ),
        (
            Q::filter(F::lt("weight", bob.weight.unwrap())),
            vec![&alice],
        ),
        (
            Q::filter(F::gt("weight", alice.weight.unwrap())),
            vec![&bob],
        ),
        (
            Q::filter(F::lte("weight", bob.weight.unwrap())),
            vec![&alice, &bob],
        ),
        (
            Q::filter(F::gte("weight", alice.weight.unwrap())),
            vec![&alice, &bob],
        ),
        // string filters:
        (Q::filter(F::ne("name", "Alice")), vec![&bob, &eve]),
        (
            Q::filter(F::in_("name", vec!["Alice", "Eve"])),
            vec![&alice, &eve],
        ),
        (Q::filter(F::contains("name", "o")), vec![&bob]),
        (Q::filter(F::starts_with("name", "E")), vec![&eve]),
        (Q::filter(F::ends_with("name", "e")), vec![&alice, &eve]),
        // bool filters:
        (Q::filter(F::eq("is_evil", true)), vec![&eve]),
        (Q::filter(F::eq("is_evil", false)), vec![&alice, &bob]),
        // datetime filters:
        (
            Q::filter(F::eq("registered_at", bob.registered_at)),
            vec![&bob],
        ),
        (
            Q::filter(F::ne("registered_at", bob.registered_at)),
            vec![&alice, &eve],
        ),
        (
            Q::filter(F::lt("registered_at", bob.registered_at)),
            vec![&alice],
        ),
        (
            Q::filter(F::gt("registered_at", bob.registered_at)),
            vec![&eve],
        ),
        (
            Q::filter(F::lte("registered_at", bob.registered_at)),
            vec![&alice, &bob],
        ),
        (
            Q::filter(F::gte("registered_at", bob.registered_at)),
            vec![&bob, &eve],
        ),
        // decimal filters:
        (Q::filter(F::eq("money", bob.money)), vec![&bob]),
        (Q::filter(F::ne("money", bob.money)), vec![&alice, &eve]),
        (Q::filter(F::lt("money", bob.money)), vec![&alice]),
        (Q::filter(F::gt("money", bob.money)), vec![&eve]),
        (Q::filter(F::lte("money", bob.money)), vec![&alice, &bob]),
        (Q::filter(F::gte("money", bob.money)), vec![&bob, &eve]),
        // uuid filters:
        (Q::filter(F::eq("id", bob.id)), vec![&bob]),
        (Q::filter(F::ne("id", bob.id)), vec![&alice, &eve]),
        (
            Q::filter(F::in_("id", vec![alice.id, eve.id])),
            vec![&alice, &eve],
        ),
        // offset, limit, order:
        (Q::new().limit(2), vec![&alice, &bob]),
        (Q::new().offset(1), vec![&bob, &eve]),
        (
            Q::new().order(vec![Order::Asc("name".to_string())]),
            vec![&alice, &bob, &eve],
        ),
        (
            Q::new().order(vec![Order::Desc("name".to_string())]),
            vec![&eve, &bob, &alice],
        ),
    ];

    for (query, expected_result) in cases {
        let users = repo.get_many(&db, &query).await.unwrap();
        let result: Vec<&User> = users.iter().collect();
        assert_eq!(result, expected_result, "filter {:?} doesn't work", query);
    }
}

#[tokio::test]
async fn transaction() {
    let db = db().await;
    let repo = users_repo().await;
    common::add_bob(&db, &repo).await;

    db.transaction(|tx| {
        Box::pin({
            let repo = repo.clone();
            async move {
                repo.delete(tx, &F::eq("name", "Bob")).await?;
                Ok(())
            }
        })
    })
    .await
    .unwrap();

    let bob = repo.get(&db, &F::eq("name", "Bob")).await.unwrap();
    assert!(bob.is_none());
}

#[tokio::test]
async fn transaction_rollback() {
    let db = db().await;
    let repo = users_repo().await;
    common::add_alice(&db, &repo).await;

    let result = db
        .transaction::<_, anyhow::Error>(|tx| {
            Box::pin({
                let repo = repo.clone();
                async move {
                    repo.delete(tx, &F::eq("name", "Alice")).await?;
                    anyhow::bail!("failed transaction")
                }
            })
        })
        .await;

    assert!(result.is_err());
    let alice = repo.get(&db, &F::eq("name", "Alice")).await.unwrap();
    assert!(alice.is_some());
}

#[tokio::test]
async fn get_for_update() {
    let db = db().await;
    let repo = users_repo().await;
    let user = common::add_bob(&db, &repo).await;
    let new_name = "Robert";

    db.transaction(|tx| {
        Box::pin({
            let repo = repo.clone();
            async move {
                let mut user = repo
                    .get_for_update(tx, &F::eq("id", user.id))
                    .await?
                    .unwrap();
                user.name = new_name.to_string();
                repo.update(tx, &F::eq("id", user.id), &user).await?;
                Ok(())
            }
        })
    })
    .await
    .unwrap();

    let user = repo.get(&db, &F::eq("id", user.id)).await.unwrap().unwrap();
    assert_eq!(user.name, new_name);
}
