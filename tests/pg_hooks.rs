use orlok::pg::PgDb;
use orlok::Repo;
use orlok::F;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

use std::collections::HashMap;

use orlok::pg::Value;

use sqlx::postgres::PgRow;

use orlok::pg::PgRepo;

#[derive(Debug, PartialEq)]
struct User {
    id: Uuid,
    name: String,
    emails: Vec<String>,
}

impl User {
    fn new(name: &str, emails: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            emails,
        }
    }
}

fn dump_user(entity: &User) -> HashMap<String, Value> {
    HashMap::from([
        ("id".to_string(), entity.id.into()),
        ("name".to_string(), entity.name.clone().into()),
    ])
}

fn load_user(row: &PgRow) -> User {
    User {
        id: row.get("id"),
        name: row.get("name"),
        emails: row.get("emails"),
    }
}

fn users_repo() -> PgRepo<User> {
    PgRepo::new("users_with_emails", dump_user, load_user)
        .query(
            "
            select u.id, u.name, array_agg(emails.email) as emails
            from users_with_emails as u
            left join emails
            on emails.user_id = u.id
            group by u.id, u.name
        ",
        )
        .after_add(|u| {
            u.emails
                .iter()
                .map(|e| {
                    sqlx::query("insert into emails (id, user_id, email) values ($1, $2, $3)")
                        .bind(Uuid::new_v4())
                        .bind(u.id)
                        .bind(e)
                })
                .collect()
        })
        .after_update(|u| {
            let mut queries = vec![sqlx::query("delete from emails where user_id = $1").bind(u.id)];
            queries.extend(u.emails.iter().map(|e| {
                sqlx::query("insert into emails (id, user_id, email) values ($1, $2, $3)")
                    .bind(Uuid::new_v4())
                    .bind(u.id)
                    .bind(e)
            }));
            queries
        })
}

pub async fn db<'a>() -> PgDb<'a> {
    let pool = PgPool::connect("postgresql://orlok:orlok@localhost/orlok")
        .await
        .unwrap();

    sqlx::query(
        "create table if not exists users_with_emails (
            id uuid primary key,
            name text
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "create table if not exists emails (
            id uuid primary key,
            user_id uuid references users_with_emails(id) on delete cascade,
            email text
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query("delete from users_with_emails")
        .execute(&pool)
        .await
        .unwrap();

    PgDb::new(pool)
}

#[tokio::test]
async fn test_one_to_many() {
    let db = db().await;
    let repo = users_repo();
    let bob = User::new(
        "Bob",
        vec!["bob@test.com".to_string(), "bob123@test.com".to_string()],
    );
    repo.add(&db, &bob).await.unwrap();
    let alice = User::new("Alice", vec!["alice@test.com".to_string()]);
    repo.add(&db, &alice).await.unwrap();
    let b = repo.get(&db, &F::eq("id", bob.id)).await.unwrap().unwrap();
    assert_eq!(b, bob);
    let a = repo.get(&db, &F::eq("id", alice.id)).await.unwrap().unwrap();
    assert_eq!(a, alice);
}
