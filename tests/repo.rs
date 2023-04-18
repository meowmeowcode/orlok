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
        let result = repo.get(&F::eq("id", 2)).await.unwrap().unwrap();
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
    for repo in common::repos().await {
        let alice = common::add_alice(&repo).await;
        let mut bob = common::add_bob(&repo).await;
        let eve = common::add_eve(&repo).await;
        bob.name = "Robert".to_string();
        repo.update(&F::eq("id", bob.id), &bob).await.unwrap();
        let users = repo
            .get_many(&Query::new().order(vec![Order::Asc("id".to_string())]))
            .await
            .unwrap();
        assert_eq!(users, vec![alice, bob, eve]);
    }
}

#[tokio::test]
async fn get_many() {
    for repo in common::repos().await {
        let alice = common::add_alice(&repo).await;
        let bob = common::add_bob(&repo).await;
        let eve = common::add_eve(&repo).await;

        let cases = [
            (
                Query::filter(F::and(&[F::gt("id", 1), F::lt("id", 3)])),
                vec![&bob],
            ),
            (
                Query::filter(F::or(&[F::eq("id", 1), F::eq("id", 3)])),
                vec![&alice, &eve],
            ),
            (Query::filter(F::ne("id", 2)), vec![&alice, &eve]),
            (Query::filter(F::lt("id", 3)), vec![&alice, &bob]),
            (Query::filter(F::gt("id", 1)), vec![&bob, &eve]),
            (Query::filter(F::lte("id", 2)), vec![&alice, &bob]),
            (Query::filter(F::gte("id", 2)), vec![&bob, &eve]),
            (Query::filter(F::in_("id", vec![1, 3])), vec![&alice, &eve]),
            (Query::filter(F::between("id", (1, 2))), vec![&alice, &bob]),
            (Query::filter(F::contains("name", "o")), vec![&bob]),
            (Query::filter(F::starts_with("name", "E")), vec![&eve]),
            (Query::filter(F::ends_with("name", "e")), vec![&alice, &eve]),
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
            assert_eq!(result, expected_result);
        }
    }
}
