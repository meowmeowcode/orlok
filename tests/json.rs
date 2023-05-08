mod common;

use orlok::base::{Db, Repo};
use orlok::json::{JsonDb, JsonRepo};
use orlok::query::{Order, F, Q};

use common::User;

async fn users_repo() -> JsonRepo<User> {
    JsonRepo::new("users".to_string())
}

async fn db<'a>() -> JsonDb<'a> {
    JsonDb::new()
}

#[tokio::test]
async fn get() {
    let db = db().await;
    let repo = users_repo().await;
    let alice = common::add_alice(&db, &repo).await;
    let bob = common::add_bob(&db, &repo).await;
    let result = repo
        .get(&db, &F::eq("name".to_string(), "Alice".to_string()))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result, alice);
    let result = repo
        .get(&db, &F::eq("id".to_string(), bob.id))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result, bob);
}

#[tokio::test]
async fn get_none() {
    let db = db().await;
    let repo = users_repo().await;
    common::add_alice(&db, &repo).await;
    let result = repo
        .get(&db, &F::eq("name".to_string(), "test".to_string()))
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn exists() {
    let db = db().await;
    let repo = users_repo().await;
    assert_eq!(
        repo.exists(&db, &F::eq("name".to_string(), "Alice".to_string()))
            .await
            .unwrap(),
        false
    );
    common::add_alice(&db, &repo).await;
    assert_eq!(
        repo.exists(&db, &F::eq("name".to_string(), "Alice".to_string()))
            .await
            .unwrap(),
        true
    );
}

#[tokio::test]
async fn count() {
    let db = db().await;
    let repo = users_repo().await;
    let filter = F::or(vec![
        F::eq("name".to_string(), "Bob".to_string()),
        F::eq("name".to_string(), "Alice".to_string()),
    ]);
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
        &F::or(vec![
            F::eq("name".to_string(), "Bob".to_string()),
            F::eq("name".to_string(), "Alice".to_string()),
        ]),
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
    repo.update(&db, &F::eq("id".to_string(), bob.id), &bob)
        .await
        .unwrap();
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
            Q::filter(F::and(vec![
                F::gt("age".to_string(), alice.age),
                F::lt("age".to_string(), eve.age),
            ])),
            vec![&bob],
        ),
        (
            Q::filter(F::or(vec![
                F::eq("age".to_string(), alice.age),
                F::eq("age".to_string(), eve.age),
            ])),
            vec![&alice, &eve],
        ),
        // i64 filters:
        (
            Q::filter(F::ne("age".to_string(), bob.age)),
            vec![&alice, &eve],
        ),
        (
            Q::filter(F::lt("age".to_string(), eve.age)),
            vec![&alice, &bob],
        ),
        (
            Q::filter(F::gt("age".to_string(), alice.age)),
            vec![&bob, &eve],
        ),
        (
            Q::filter(F::lte("age".to_string(), bob.age)),
            vec![&alice, &bob],
        ),
        (
            Q::filter(F::gte("age".to_string(), bob.age)),
            vec![&bob, &eve],
        ),
        (
            Q::filter(F::in_("age".to_string(), vec![alice.age, eve.age])),
            vec![&alice, &eve],
        ),
        (
            Q::filter(F::between("age".to_string(), (alice.age, bob.age))),
            vec![&alice, &bob],
        ),
        // f64 filters
        (
            Q::filter(F::eq("weight".to_string(), bob.weight.unwrap())),
            vec![&bob],
        ),
        (
            Q::filter(F::ne("weight".to_string(), bob.weight.unwrap())),
            vec![&alice],
        ),
        (
            Q::filter(F::lt("weight".to_string(), bob.weight.unwrap())),
            vec![&alice],
        ),
        (
            Q::filter(F::gt("weight".to_string(), alice.weight.unwrap())),
            vec![&bob],
        ),
        (
            Q::filter(F::lte("weight".to_string(), bob.weight.unwrap())),
            vec![&alice, &bob],
        ),
        (
            Q::filter(F::gte("weight".to_string(), alice.weight.unwrap())),
            vec![&alice, &bob],
        ),
        // string filters:
        (
            Q::filter(F::contains("name".to_string(), "o".to_string())),
            vec![&bob],
        ),
        (
            Q::filter(F::starts_with("name".to_string(), "E".to_string())),
            vec![&eve],
        ),
        (
            Q::filter(F::ends_with("name".to_string(), "e".to_string())),
            vec![&alice, &eve],
        ),
        // bool filters:
        (Q::filter(F::eq("is_evil".to_string(), true)), vec![&eve]),
        (
            Q::filter(F::eq("is_evil".to_string(), false)),
            vec![&alice, &bob],
        ),
        // datetime filters:
        (
            Q::filter(F::eq("registered_at".to_string(), bob.registered_at)),
            vec![&bob],
        ),
        (
            Q::filter(F::ne("registered_at".to_string(), bob.registered_at)),
            vec![&alice, &eve],
        ),
        (
            Q::filter(F::lt("registered_at".to_string(), bob.registered_at)),
            vec![&alice],
        ),
        (
            Q::filter(F::gt("registered_at".to_string(), bob.registered_at)),
            vec![&eve],
        ),
        (
            Q::filter(F::lte("registered_at".to_string(), bob.registered_at)),
            vec![&alice, &bob],
        ),
        (
            Q::filter(F::gte("registered_at".to_string(), bob.registered_at)),
            vec![&bob, &eve],
        ),
        // decimal filters:
        (Q::filter(F::eq("money".to_string(), bob.money)), vec![&bob]),
        (
            Q::filter(F::ne("money".to_string(), bob.money)),
            vec![&alice, &eve],
        ),
        (
            Q::filter(F::lt("money".to_string(), bob.money)),
            vec![&alice],
        ),
        (Q::filter(F::gt("money".to_string(), bob.money)), vec![&eve]),
        (
            Q::filter(F::lte("money".to_string(), bob.money)),
            vec![&alice, &bob],
        ),
        (
            Q::filter(F::gte("money".to_string(), bob.money)),
            vec![&bob, &eve],
        ),
        // uuid filters:
        (Q::filter(F::eq("id".to_string(), bob.id)), vec![&bob]),
        (
            Q::filter(F::ne("id".to_string(), bob.id)),
            vec![&alice, &eve],
        ),
        (
            Q::filter(F::in_("id".to_string(), vec![alice.id, eve.id])),
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
                repo.delete(tx, &F::eq("name".to_string(), "Bob".to_string()))
                    .await?;
                Ok(())
            }
        })
    })
    .await
    .unwrap();

    let bob = repo
        .get(&db, &F::eq("name".to_string(), "Bob".to_string()))
        .await
        .unwrap();
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
                    repo.delete(tx, &F::eq("name".to_string(), "Alice".to_string()))
                        .await?;
                    anyhow::bail!("failed transaction")
                }
            })
        })
        .await;

    assert!(result.is_err());
    let alice = repo
        .get(&db, &F::eq("name".to_string(), "Alice".to_string()))
        .await
        .unwrap();
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
                    .get_for_update(tx, &F::eq("id".to_string(), user.id))
                    .await?
                    .unwrap();
                user.name = new_name.to_string();
                repo.update(tx, &F::eq("id".to_string(), user.id), &user)
                    .await?;
                Ok(())
            }
        })
    })
    .await
    .unwrap();

    let user = repo
        .get(&db, &F::eq("id".to_string(), user.id))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user.name, new_name);
}
