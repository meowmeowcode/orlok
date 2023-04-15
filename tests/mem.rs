use tokio;

use orlok::base::Repo;
use orlok::mem::MemoryRepo;
use orlok::query::{Order, Query, F};

mod common;

fn users_repo() -> MemoryRepo<common::User> {
    MemoryRepo::new()
}

#[tokio::test]
async fn get() {
    let repo = users_repo();
    let alice = common::add_alice(&repo).await;
    let bob = common::add_bob(&repo).await;
    let result = repo.get(&F::eq("name", "Alice")).await.unwrap().unwrap();
    assert_eq!(result, alice);
    let result = repo.get(&F::eq("id", 2)).await.unwrap().unwrap();
    assert_eq!(result, bob);
}

#[tokio::test]
async fn get_none() {
    let repo = users_repo();
    common::add_alice(&repo).await;
    let uu = repo.get(&F::eq("name", "test")).await.unwrap();
    assert!(uu.is_none());
}

#[tokio::test]
async fn and() {
    let repo = users_repo();
    common::add_alice(&repo).await;
    let bob = common::add_bob(&repo).await;
    common::add_eve(&repo).await;

    let result = repo
        .get_many(&Query::filter(F::and(vec![F::gt("id", 1), F::lt("id", 3)])))
        .await
        .unwrap();
    assert_eq!(result, vec![bob]);
}

#[tokio::test]
async fn or() {
    let repo = users_repo();
    let alice = common::add_alice(&repo).await;
    common::add_bob(&repo).await;
    let eve = common::add_eve(&repo).await;

    let result = repo
        .get_many(&Query::filter(F::or(vec![F::eq("id", 1), F::eq("id", 3)])))
        .await
        .unwrap();
    assert_eq!(result, vec![alice, eve]);
}

#[tokio::test]
async fn int_ne() {
    let repo = users_repo();
    let alice = common::add_alice(&repo).await;
    common::add_bob(&repo).await;
    let eve = common::add_eve(&repo).await;
    let users = repo.get_many(&Query::filter(F::ne("id", 2))).await.unwrap();
    assert_eq!(users, vec![alice, eve]);
}

#[tokio::test]
async fn int_lt() {
    let repo = users_repo();
    let alice = common::add_alice(&repo).await;
    let bob = common::add_bob(&repo).await;
    common::add_eve(&repo).await;
    let users = repo.get_many(&Query::filter(F::lt("id", 3))).await.unwrap();
    assert_eq!(users, vec![alice, bob]);
}

#[tokio::test]
async fn int_gt() {
    let repo = users_repo();
    common::add_alice(&repo).await;
    let bob = common::add_bob(&repo).await;
    let eve = common::add_eve(&repo).await;
    let users = repo.get_many(&Query::filter(F::gt("id", 1))).await.unwrap();
    assert_eq!(users, vec![bob, eve]);
}

#[tokio::test]
async fn int_lte() {
    let repo = users_repo();
    let alice = common::add_alice(&repo).await;
    let bob = common::add_bob(&repo).await;
    common::add_eve(&repo).await;
    let users = repo
        .get_many(&Query::filter(F::lte("id", 2)))
        .await
        .unwrap();
    assert_eq!(users, vec![alice, bob]);
}

#[tokio::test]
async fn int_gte() {
    let repo = users_repo();
    common::add_alice(&repo).await;
    let bob = common::add_bob(&repo).await;
    let eve = common::add_eve(&repo).await;
    let query = Query::filter(F::gte("id", 2));
    let users = repo.get_many(&query).await.unwrap();
    assert_eq!(users, vec![bob, eve]);
}

#[tokio::test]
async fn int_in() {
    let repo = users_repo();
    let alice = common::add_alice(&repo).await;
    common::add_bob(&repo).await;
    let eve = common::add_eve(&repo).await;
    let users = repo
        .get_many(&Query::filter(F::in_("id", vec![1, 3])))
        .await
        .unwrap();
    assert_eq!(users, vec![alice, eve]);
}

#[tokio::test]
async fn int_between() {
    let repo = users_repo();
    let alice = common::add_alice(&repo).await;
    let bob = common::add_bob(&repo).await;
    common::add_eve(&repo).await;
    let users = repo
        .get_many(&Query::filter(F::between("id", (1, 2))))
        .await
        .unwrap();
    assert_eq!(users, vec![alice, bob]);
}

#[tokio::test]
async fn contains() {
    let repo = users_repo();
    common::add_alice(&repo).await;
    let bob = common::add_bob(&repo).await;
    common::add_eve(&repo).await;
    let users = repo
        .get_many(&Query::filter(F::contains("name", "o")))
        .await
        .unwrap();
    assert_eq!(users, vec![bob]);
}

#[tokio::test]
async fn starts_with() {
    let repo = users_repo();
    common::add_alice(&repo).await;
    common::add_bob(&repo).await;
    let eve = common::add_eve(&repo).await;
    let users = repo
        .get_many(&Query::filter(F::starts_with("name", "E")))
        .await
        .unwrap();
    assert_eq!(users, vec![eve]);
}

#[tokio::test]
async fn ends_with() {
    let repo = users_repo();
    let alice = common::add_alice(&repo).await;
    common::add_bob(&repo).await;
    let eve = common::add_eve(&repo).await;
    let users = repo
        .get_many(&Query::filter(F::ends_with("name", "e")))
        .await
        .unwrap();
    assert_eq!(users, vec![alice, eve]);
}

#[tokio::test]
async fn limit() {
    let repo = users_repo();
    let alice = common::add_alice(&repo).await;
    let bob = common::add_bob(&repo).await;
    common::add_eve(&repo).await;
    let users = repo.get_many(&Query::new().limit(2)).await.unwrap();
    assert_eq!(users, vec![alice, bob]);
}

#[tokio::test]
async fn offset() {
    let repo = users_repo();
    common::add_alice(&repo).await;
    let bob = common::add_bob(&repo).await;
    let eve = common::add_eve(&repo).await;
    let users = repo.get_many(&Query::new().offset(1)).await.unwrap();
    assert_eq!(users, vec![bob, eve]);
}

#[tokio::test]
async fn order() {
    let repo = users_repo();
    let bob = common::add_bob(&repo).await;
    let eve = common::add_eve(&repo).await;
    let alice = common::add_alice(&repo).await;
    let unordered = repo.get_many(&Query::new()).await.unwrap();
    let ordered = repo
        .get_many(&Query::new().order(vec![Order::Asc("name".to_string())]))
        .await
        .unwrap();
    assert_eq!(unordered, vec![bob.clone(), eve.clone(), alice.clone()]);
    assert_eq!(ordered, vec![alice, bob, eve]);
}

#[tokio::test]
async fn order_desc() {
    let repo = users_repo();
    let bob = common::add_bob(&repo).await;
    let eve = common::add_eve(&repo).await;
    let alice = common::add_alice(&repo).await;
    let ordered = repo
        .get_many(&Query::new().order(vec![Order::Desc("name".to_string())]))
        .await
        .unwrap();
    assert_eq!(ordered, vec![eve, bob, alice]);
}

#[tokio::test]
async fn delete() {
    let repo = users_repo();
    common::add_alice(&repo).await;
    common::add_bob(&repo).await;
    let eve = common::add_eve(&repo).await;
    repo.delete(&F::or(vec![F::eq("name", "Bob"), F::eq("name", "Alice")]))
        .await
        .unwrap();
    let users = repo.get_many(&Query::new()).await.unwrap();
    assert_eq!(users, vec![eve]);
}

#[tokio::test]
async fn update() {
    let repo = users_repo();
    let alice = common::add_alice(&repo).await;
    let mut bob = common::add_bob(&repo).await;
    let eve = common::add_eve(&repo).await;
    bob.name = "Robert".to_string();
    repo.update(&F::eq("id", bob.id), &bob).await.unwrap();
    let users = repo.get_many(&Query::new()).await.unwrap();
    assert_eq!(users, vec![alice, bob, eve]);
}
