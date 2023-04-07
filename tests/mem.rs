use orlok::base::Repo;
use orlok::mem::MemoryRepo;
use orlok::query::{Order, Query, F};

mod common;

fn users_repo() -> MemoryRepo<common::User> {
    MemoryRepo::new()
}

#[test]
fn get() {
    let repo = users_repo();
    let alice = common::add_alice(&repo);
    let bob = common::add_bob(&repo);
    let result = repo.get(&F::eq("name", "Alice")).unwrap();
    assert_eq!(result, alice);
    let result = repo.get(&F::eq("id", 2)).unwrap();
    assert_eq!(result, bob);
}

#[test]
fn get_none() {
    let repo = users_repo();
    common::add_alice(&repo);
    let uu = repo.get(&F::eq("name", "test"));
    assert!(uu.is_none());
}

#[test]
fn and() {
    let repo = users_repo();
    common::add_alice(&repo);
    let bob = common::add_bob(&repo);
    common::add_eve(&repo);

    let result = repo.get_many(&Query::filter(F::and(vec![F::gt("id", 1), F::lt("id", 3)])));
    assert_eq!(result, vec![bob]);
}

#[test]
fn or() {
    let repo = users_repo();
    let alice = common::add_alice(&repo);
    common::add_bob(&repo);
    let eve = common::add_eve(&repo);

    let result = repo.get_many(&Query::filter(F::or(vec![F::eq("id", 1), F::eq("id", 3)])));
    assert_eq!(result, vec![alice, eve]);
}

#[test]
fn int_ne() {
    let repo = users_repo();
    let alice = common::add_alice(&repo);
    common::add_bob(&repo);
    let eve = common::add_eve(&repo);
    let users = repo.get_many(&Query::filter(F::ne("id", 2)));
    assert_eq!(users, vec![alice, eve]);
}

#[test]
fn int_lt() {
    let repo = users_repo();
    let alice = common::add_alice(&repo);
    let bob = common::add_bob(&repo);
    common::add_eve(&repo);
    let users = repo.get_many(&Query::filter(F::lt("id", 3)));
    assert_eq!(users, vec![alice, bob]);
}

#[test]
fn int_gt() {
    let repo = users_repo();
    common::add_alice(&repo);
    let bob = common::add_bob(&repo);
    let eve = common::add_eve(&repo);
    let users = repo.get_many(&Query::filter(F::gt("id", 1)));
    assert_eq!(users, vec![bob, eve]);
}

#[test]
fn int_lte() {
    let repo = users_repo();
    let alice = common::add_alice(&repo);
    let bob = common::add_bob(&repo);
    common::add_eve(&repo);
    let users = repo.get_many(&Query::filter(F::lte("id", 2)));
    assert_eq!(users, vec![alice, bob]);
}

#[test]
fn int_gte() {
    let repo = users_repo();
    common::add_alice(&repo);
    let bob = common::add_bob(&repo);
    let eve = common::add_eve(&repo);
    let query = Query::filter(F::gte("id", 2));
    let users = repo.get_many(&query);
    assert_eq!(users, vec![bob, eve]);
}

#[test]
fn int_in() {
    let repo = users_repo();
    let alice = common::add_alice(&repo);
    common::add_bob(&repo);
    let eve = common::add_eve(&repo);
    let users = repo.get_many(&Query::filter(F::in_("id", vec![1, 3])));
    assert_eq!(users, vec![alice, eve]);
}

#[test]
fn int_between() {
    let repo = users_repo();
    let alice = common::add_alice(&repo);
    let bob = common::add_bob(&repo);
    common::add_eve(&repo);
    let users = repo.get_many(&Query::filter(F::between("id", (1, 2))));
    assert_eq!(users, vec![alice, bob]);
}

#[test]
fn contains() {
    let repo = users_repo();
    common::add_alice(&repo);
    let bob = common::add_bob(&repo);
    common::add_eve(&repo);
    let users = repo.get_many(&Query::filter(F::contains("name", "o")));
    assert_eq!(users, vec![bob]);
}

#[test]
fn starts_with() {
    let repo = users_repo();
    common::add_alice(&repo);
    common::add_bob(&repo);
    let eve = common::add_eve(&repo);
    let users = repo.get_many(&Query::filter(F::starts_with("name", "E")));
    assert_eq!(users, vec![eve]);
}

#[test]
fn ends_with() {
    let repo = users_repo();
    let alice = common::add_alice(&repo);
    common::add_bob(&repo);
    let eve = common::add_eve(&repo);
    let users = repo.get_many(&Query::filter(F::ends_with("name", "e")));
    assert_eq!(users, vec![alice, eve]);
}

#[test]
fn limit() {
    let repo = users_repo();
    let alice = common::add_alice(&repo);
    let bob = common::add_bob(&repo);
    common::add_eve(&repo);
    let users = repo.get_many(&Query::new().limit(2));
    assert_eq!(users, vec![alice, bob]);
}

#[test]
fn offset() {
    let repo = users_repo();
    common::add_alice(&repo);
    let bob = common::add_bob(&repo);
    let eve = common::add_eve(&repo);
    let users = repo.get_many(&Query::new().offset(1));
    assert_eq!(users, vec![bob, eve]);
}

#[test]
fn order() {
    let repo = users_repo();
    let bob = common::add_bob(&repo);
    let eve = common::add_eve(&repo);
    let alice = common::add_alice(&repo);
    let unordered = repo.get_many(&Query::new());
    let ordered = repo.get_many(&Query::new().order(vec![Order::Asc("name".to_string())]));
    assert_eq!(unordered, vec![bob.clone(), eve.clone(), alice.clone()]);
    assert_eq!(ordered, vec![alice, bob, eve]);
}

#[test]
fn order_desc() {
    let repo = users_repo();
    let bob = common::add_bob(&repo);
    let eve = common::add_eve(&repo);
    let alice = common::add_alice(&repo);
    let ordered = repo.get_many(&Query::new().order(vec![Order::Desc("name".to_string())]));
    assert_eq!(ordered, vec![eve, bob, alice]);
}

#[test]
fn delete() {
    let repo = users_repo();
    common::add_alice(&repo);
    common::add_bob(&repo);
    let eve = common::add_eve(&repo);
    repo.delete(&F::or(vec![F::eq("name", "Bob"), F::eq("name", "Alice")]));
    let users = repo.get_many(&Query::new());
    assert_eq!(users, vec![eve]);
}

#[test]
fn update() {
    let repo = users_repo();
    let alice = common::add_alice(&repo);
    let mut bob = common::add_bob(&repo);
    let eve = common::add_eve(&repo);
    bob.name = "Robert".to_string();
    repo.update(&F::eq("id", bob.id), &bob);
    let users = repo.get_many(&Query::new());
    assert_eq!(users, vec![alice, bob, eve]);
}
