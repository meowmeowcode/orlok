mod common;

use orlok::query::{Order, Query, F};

use common::User;

#[tokio::test]
async fn get() {
    for repo in common::repos().await {
        let alice = common::add_alice(&repo).await;
        let bob = common::add_bob(&repo).await;
        let result = repo.get(&F::eq("name", "Alice")).await.unwrap().unwrap();
        assert_eq!(result, alice);
        let result = repo.get(&F::eq("id", bob.id)).await.unwrap().unwrap();
        assert_eq!(result, bob);
    }
}

#[tokio::test]
async fn get_none() {
    for repo in common::repos().await {
        common::add_alice(&repo).await;
        let result = repo.get(&F::eq("name", "test")).await.unwrap();
        assert!(result.is_none());
    }
}

#[tokio::test]
async fn delete() {
    for repo in common::repos().await {
        common::add_alice(&repo).await;
        common::add_bob(&repo).await;
        let eve = common::add_eve(&repo).await;
        repo.delete(&F::or(&[F::eq("name", "Bob"), F::eq("name", "Alice")]))
            .await
            .unwrap();
        let users = repo.get_many(&Query::new()).await.unwrap();
        assert_eq!(users, vec![eve]);
    }
}

#[tokio::test]
async fn update() {
    for (repo_id, repo) in common::repos().await.iter().enumerate() {
        let alice = common::add_alice(&repo).await;
        let mut bob = common::add_bob(&repo).await;
        let eve = common::add_eve(&repo).await;
        bob.name = "Robert".to_string();
        repo.update(&F::eq("id", bob.id), &bob).await.unwrap();
        let users = repo
            .get_many(&Query::new().order(vec![Order::Asc("name".to_string())]))
            .await
            .unwrap();
        assert_eq!(
            users,
            vec![alice, eve, bob],
            "update one user for repo {}",
            repo_id
        );
    }
}

#[tokio::test]
async fn get_many() {
    for (repo_id, repo) in common::repos().await.iter().enumerate() {
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
            assert_eq!(
                result, expected_result,
                "filter {:?} doesn't work in repo {}",
                query, repo_id
            );
        }
    }
}
