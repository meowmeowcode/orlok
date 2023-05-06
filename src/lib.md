<div style="text-align: center">
    <img src="https://github.com/meowmeowcode/orlok/raw/docs/orlok.png" width="200" alt="Orlok" />
</div>

# Orlok

Orlok is a database toolkit that contains reusable generic
implementations of the [Repository](https://martinfowler.com/eaaCatalog/repository.html) pattern.
It can help you to separate business logic from data storage details
and save you from writing some amount of boilerplate code.

At the moment only PostgreSQL is supported with the help of
[sqlx](https://crates.io/crates/sqlx) and [sea_query](https://crates.io/crates/sea-query).


<div style="background-color: beige; padding: 14px">
    This crate was written in the process of learning Rust,
    so if you're an experienced rustacean, don't be surprised
    if some of its parts will look unnatural to you.
</div>


## Guide

### Installation

```toml
## Cargo.toml
[dependencies]
orlok = "0"
```

### Creating a repository

Suppose we have a struct representing a user of some application:

```rust
use uuid::Uuid;

#[derive(PartialEq, Clone, Debug)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub is_active: bool,
}

impl User {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name,
            is_active: true,
        }
    }
}
```

We're going to store this struct in a database, so
let's create a table for this purpose:

```rust
# use tokio_test;
# fn main() -> anyhow::Result<()> {
#     tokio_test::block_on(async {
use sqlx::PgPool;

let pool = PgPool::connect("postgresql://orlok:orlok@localhost/orlok")
    .await?;

sqlx::query(
    "create table if not exists users (
        id uuid,
        name text,
        is_active boolean
    )"
)
.execute(&pool)
.await?;
#         sqlx::query("delete from users").execute(&pool).await?;
#         Ok(())
#     })
# }
```

To save the user struct to the database we need
to somehow map its fields to a table row.
We can define a function for this.
This function must return a `HashMap` with keys and values
that correspond to columns and values in the "users" table:

```rust
#         use uuid::Uuid;
# 
#         #[derive(PartialEq, Clone, Debug)]
#         pub struct User {
#             pub id: Uuid,
#             pub name: String,
#             pub is_active: bool,
#         }
use std::collections::HashMap;
use sea_query::SimpleExpr;

fn dump_user(u: &User) -> HashMap<String, SimpleExpr> {
    HashMap::from([
        ("id".to_string(), u.id.into()),
        ("name".to_string(), u.name.clone().into()),
        ("is_active".to_string(), u.is_active.into()),
    ])
}
```

After saving the user struct to the database,
we want to be able to load it back,
so we need a function that maps
a database row to the struct:

```rust
#         use uuid::Uuid;
#
#         #[derive(PartialEq, Clone, Debug)]
#         pub struct User {
#             pub id: Uuid,
#             pub name: String,
#             pub is_active: bool,
#         }
use sqlx::Row;
use sqlx::postgres::PgRow;

fn load_user(row: &PgRow) -> User {
    User {
        id: row.get("id"),
        name: row.get("name"),
        is_active: row.get("is_active"),
    }
}
```

After this, we can create a repository:

```rust
#         use uuid::Uuid;
# 
#         #[derive(PartialEq, Clone, Debug)]
#         pub struct User {
#             pub id: Uuid,
#             pub name: String,
#             pub is_active: bool,
#         }
#         use std::collections::HashMap;
#         use sea_query::SimpleExpr;
# 
#         fn dump_user(u: &User) -> HashMap<String, SimpleExpr> {
#             HashMap::from([
#                 ("id".to_string(), u.id.into()),
#                 ("name".to_string(), u.name.clone().into()),
#                 ("is_active".to_string(), u.is_active.into()),
#             ])
#         }
# 
# 
#         use sqlx::Row;
#         use sqlx::postgres::PgRow;
# 
#         fn load_user(row: &PgRow) -> User {
#             User {
#                 id: row.get("id"),
#                 name: row.get("name"),
#                 is_active: row.get("is_active"),
#             }
#         }
# 
# 
use orlok::Repo;
use orlok::pg::PgRepo;

let users_repo = PgRepo::new("users".to_string(), dump_user, load_user);
```

The first argument for the `new` function is the name of the table where we want to store our users.

### Saving entities

Repositories use a special wrapper around a connection to a database. Let's create one:

```rust
# use tokio_test;
# fn main() -> anyhow::Result<()> {
#     tokio_test::block_on(async {
#         use sqlx::PgPool;
# 
#         let pool = PgPool::connect("postgresql://orlok:orlok@localhost/orlok")
#             .await?;
use orlok::Db;
use orlok::pg::PgDb;

let db = PgDb::new(pool);
# 
#         Ok(())
#     })
# }
```

Now we can save new users:

```rust
# use tokio_test;
# fn main() -> anyhow::Result<()> {
#     tokio_test::block_on(async {
#         use uuid::Uuid;
# 
#         #[derive(PartialEq, Clone, Debug)]
#         pub struct User {
#             pub id: Uuid,
#             pub name: String,
#             pub is_active: bool,
#         }
# 
#         impl User {
#             pub fn new(name: String) -> Self {
#                 Self {
#                     id: Uuid::new_v4(),
#                     name: name,
#                     is_active: true,
#                 }
#             }
#         }
# 
# 
#         use sqlx::PgPool;
# 
#         let pool = PgPool::connect("postgresql://orlok:orlok@localhost/orlok")
#             .await?;
# 
#         sqlx::query(
#             "create table if not exists users (
#                 id uuid,
#                 name text,
#                 is_active boolean
#             )"
#         )
#         .execute(&pool)
#         .await?;
#         sqlx::query("delete from users").execute(&pool).await?;
# 
# 
#         use std::collections::HashMap;
#         use sea_query::SimpleExpr;
# 
#         fn dump_user(u: &User) -> HashMap<String, SimpleExpr> {
#             HashMap::from([
#                 ("id".to_string(), u.id.into()),
#                 ("name".to_string(), u.name.clone().into()),
#                 ("is_active".to_string(), u.is_active.into()),
#             ])
#         }
# 
# 
#         use sqlx::Row;
#         use sqlx::postgres::PgRow;
# 
#         fn load_user(row: &PgRow) -> User {
#             User {
#                 id: row.get("id"),
#                 name: row.get("name"),
#                 is_active: row.get("is_active"),
#             }
#         }
# 
# 
#         use orlok::Repo;
#         use orlok::pg::PgRepo;
# 
#         let users_repo = PgRepo::new("users".to_string(), dump_user, load_user);
# 
# 
#         use orlok::Db;
#         use orlok::pg::PgDb;
# 
#         let db = PgDb::new(pool);
# 
# 
let alice = User::new("Alice".to_string());
users_repo.add(&db, &alice).await?;
let bob = User::new("Bob".to_string());
users_repo.add(&db, &bob).await?;
let eve = User::new("Eve".to_string());
users_repo.add(&db, &eve).await?;
#         Ok(())
#     })
# }
```

### Loading one entity

Use the `get` method if you want to load only one entity from the database:

```rust
# use tokio_test;
# fn main() -> anyhow::Result<()> {
#     tokio_test::block_on(async {
#         use uuid::Uuid;
# 
#         #[derive(PartialEq, Clone, Debug)]
#         pub struct User {
#             pub id: Uuid,
#             pub name: String,
#             pub is_active: bool,
#         }
# 
#         impl User {
#             pub fn new(name: String) -> Self {
#                 Self {
#                     id: Uuid::new_v4(),
#                     name: name,
#                     is_active: true,
#                 }
#             }
#         }
# 
# 
#         use sqlx::PgPool;
# 
#         let pool = PgPool::connect("postgresql://orlok:orlok@localhost/orlok")
#             .await?;
# 
#         sqlx::query(
#             "create table if not exists users (
#                 id uuid,
#                 name text,
#                 is_active boolean
#             )"
#         )
#         .execute(&pool)
#         .await?;
#         sqlx::query("delete from users").execute(&pool).await?;
# 
# 
#         use std::collections::HashMap;
#         use sea_query::SimpleExpr;
# 
#         fn dump_user(u: &User) -> HashMap<String, SimpleExpr> {
#             HashMap::from([
#                 ("id".to_string(), u.id.into()),
#                 ("name".to_string(), u.name.clone().into()),
#                 ("is_active".to_string(), u.is_active.into()),
#             ])
#         }
# 
# 
#         use sqlx::Row;
#         use sqlx::postgres::PgRow;
# 
#         fn load_user(row: &PgRow) -> User {
#             User {
#                 id: row.get("id"),
#                 name: row.get("name"),
#                 is_active: row.get("is_active"),
#             }
#         }
# 
# 
#         use orlok::Repo;
#         use orlok::pg::PgRepo;
# 
#         let users_repo = PgRepo::new("users".to_string(), dump_user, load_user);
# 
# 
#         use orlok::Db;
#         use orlok::pg::PgDb;
# 
#         let db = PgDb::new(pool);
# 
# 
#         let alice = User::new("Alice".to_string());
#         users_repo.add(&db, &alice).await?;
#         let bob = User::new("Bob".to_string());
#         users_repo.add(&db, &bob).await?;
#         let eve = User::new("Eve".to_string());
#         users_repo.add(&db, &eve).await?;
# 
# 
use orlok::F;

let user = users_repo.get(&db, &F::eq("name", "Alice")).await?.unwrap();
assert_eq!(user, alice);
#         Ok(())
#     })
# }
```

The result of this method contains an `Option` which is `None` if no record was found:

```rust
# use tokio_test;
# fn main() -> anyhow::Result<()> {
#     tokio_test::block_on(async {
#         use uuid::Uuid;
# 
#         #[derive(PartialEq, Clone, Debug)]
#         pub struct User {
#             pub id: Uuid,
#             pub name: String,
#             pub is_active: bool,
#         }
# 
#         impl User {
#             pub fn new(name: String) -> Self {
#                 Self {
#                     id: Uuid::new_v4(),
#                     name: name,
#                     is_active: true,
#                 }
#             }
#         }
# 
# 
#         use sqlx::PgPool;
# 
#         let pool = PgPool::connect("postgresql://orlok:orlok@localhost/orlok")
#             .await?;
# 
#         sqlx::query(
#             "create table if not exists users (
#                 id uuid,
#                 name text,
#                 is_active boolean
#             )"
#         )
#         .execute(&pool)
#         .await?;
#         sqlx::query("delete from users").execute(&pool).await?;
# 
# 
#         use std::collections::HashMap;
#         use sea_query::SimpleExpr;
# 
#         fn dump_user(u: &User) -> HashMap<String, SimpleExpr> {
#             HashMap::from([
#                 ("id".to_string(), u.id.into()),
#                 ("name".to_string(), u.name.clone().into()),
#                 ("is_active".to_string(), u.is_active.into()),
#             ])
#         }
# 
# 
#         use sqlx::Row;
#         use sqlx::postgres::PgRow;
# 
#         fn load_user(row: &PgRow) -> User {
#             User {
#                 id: row.get("id"),
#                 name: row.get("name"),
#                 is_active: row.get("is_active"),
#             }
#         }
# 
# 
#         use orlok::Repo;
#         use orlok::pg::PgRepo;
# 
#         let users_repo = PgRepo::new("users".to_string(), dump_user, load_user);
# 
# 
#         use orlok::Db;
#         use orlok::pg::PgDb;
# 
#         let db = PgDb::new(pool);
# 
# 
#         let alice = User::new("Alice".to_string());
#         users_repo.add(&db, &alice).await?;
#         let bob = User::new("Bob".to_string());
#         users_repo.add(&db, &bob).await?;
#         let eve = User::new("Eve".to_string());
#         users_repo.add(&db, &eve).await?;
# 
# 
#         use orlok::F;
let user = users_repo.get(&db, &F::eq("name", "Mikhail")).await?;
assert!(user.is_none());
#         Ok(())
#     })
# }
```

Note that here we use the `F` struct for filtering entities.
It has different methods for different conditions.
For example, if we want to find a user with
the letter "o" in their name, we can do something like this:

```rust
# use tokio_test;
# fn main() -> anyhow::Result<()> {
#     tokio_test::block_on(async {
#         use uuid::Uuid;
# 
#         #[derive(PartialEq, Clone, Debug)]
#         pub struct User {
#             pub id: Uuid,
#             pub name: String,
#             pub is_active: bool,
#         }
# 
#         impl User {
#             pub fn new(name: String) -> Self {
#                 Self {
#                     id: Uuid::new_v4(),
#                     name: name,
#                     is_active: true,
#                 }
#             }
#         }
# 
# 
#         use sqlx::PgPool;
# 
#         let pool = PgPool::connect("postgresql://orlok:orlok@localhost/orlok")
#             .await?;
# 
#         sqlx::query(
#             "create table if not exists users (
#                 id uuid,
#                 name text,
#                 is_active boolean
#             )"
#         )
#         .execute(&pool)
#         .await?;
#         sqlx::query("delete from users").execute(&pool).await?;
# 
# 
#         use std::collections::HashMap;
#         use sea_query::SimpleExpr;
# 
#         fn dump_user(u: &User) -> HashMap<String, SimpleExpr> {
#             HashMap::from([
#                 ("id".to_string(), u.id.into()),
#                 ("name".to_string(), u.name.clone().into()),
#                 ("is_active".to_string(), u.is_active.into()),
#             ])
#         }
# 
# 
#         use sqlx::Row;
#         use sqlx::postgres::PgRow;
# 
#         fn load_user(row: &PgRow) -> User {
#             User {
#                 id: row.get("id"),
#                 name: row.get("name"),
#                 is_active: row.get("is_active"),
#             }
#         }
# 
# 
#         use orlok::Repo;
#         use orlok::pg::PgRepo;
# 
#         let users_repo = PgRepo::new("users".to_string(), dump_user, load_user);
# 
# 
#         use orlok::Db;
#         use orlok::pg::PgDb;
# 
#         let db = PgDb::new(pool);
# 
# 
#         let alice = User::new("Alice".to_string());
#         users_repo.add(&db, &alice).await?;
#         let bob = User::new("Bob".to_string());
#         users_repo.add(&db, &bob).await?;
#         let eve = User::new("Eve".to_string());
#         users_repo.add(&db, &eve).await?;
# 
# 
#         use orlok::F;
let user = users_repo.get(&db, &F::contains("name", "o")).await?.unwrap();
assert_eq!(user, bob);
#         Ok(())
#     })
# }
```

Multiple filters can be combined this way:

```rust
# use tokio_test;
# fn main() -> anyhow::Result<()> {
#     tokio_test::block_on(async {
#         use uuid::Uuid;
# 
#         #[derive(PartialEq, Clone, Debug)]
#         pub struct User {
#             pub id: Uuid,
#             pub name: String,
#             pub is_active: bool,
#         }
# 
#         impl User {
#             pub fn new(name: String) -> Self {
#                 Self {
#                     id: Uuid::new_v4(),
#                     name: name,
#                     is_active: true,
#                 }
#             }
#         }
# 
# 
#         use sqlx::PgPool;
# 
#         let pool = PgPool::connect("postgresql://orlok:orlok@localhost/orlok")
#             .await?;
# 
#         sqlx::query(
#             "create table if not exists users (
#                 id uuid,
#                 name text,
#                 is_active boolean
#             )"
#         )
#         .execute(&pool)
#         .await?;
#         sqlx::query("delete from users").execute(&pool).await?;
# 
# 
#         use std::collections::HashMap;
#         use sea_query::SimpleExpr;
# 
#         fn dump_user(u: &User) -> HashMap<String, SimpleExpr> {
#             HashMap::from([
#                 ("id".to_string(), u.id.into()),
#                 ("name".to_string(), u.name.clone().into()),
#                 ("is_active".to_string(), u.is_active.into()),
#             ])
#         }
# 
# 
#         use sqlx::Row;
#         use sqlx::postgres::PgRow;
# 
#         fn load_user(row: &PgRow) -> User {
#             User {
#                 id: row.get("id"),
#                 name: row.get("name"),
#                 is_active: row.get("is_active"),
#             }
#         }
# 
# 
#         use orlok::Repo;
#         use orlok::pg::PgRepo;
# 
#         let users_repo = PgRepo::new("users".to_string(), dump_user, load_user);
# 
# 
#         use orlok::Db;
#         use orlok::pg::PgDb;
# 
#         let db = PgDb::new(pool);
# 
# 
#         let alice = User::new("Alice".to_string());
#         users_repo.add(&db, &alice).await?;
#         let bob = User::new("Bob".to_string());
#         users_repo.add(&db, &bob).await?;
#         let eve = User::new("Eve".to_string());
#         users_repo.add(&db, &eve).await?;
# 
# 
#         use orlok::F;
let user = users_repo.get(
    &db,
    &F::and(
        &[
            F::starts_with("name", "E"),
            F::ends_with("name", "e")
        ]
    )
).await?.unwrap();

assert_eq!(user, eve);
#         Ok(())
#     })
# }
```

### Loading several entities

If you need to load several entities, use the `get_many` method:

```rust
# use tokio_test;
# fn main() -> anyhow::Result<()> {
#     tokio_test::block_on(async {
#         use uuid::Uuid;
# 
#         #[derive(PartialEq, Clone, Debug)]
#         pub struct User {
#             pub id: Uuid,
#             pub name: String,
#             pub is_active: bool,
#         }
# 
#         impl User {
#             pub fn new(name: String) -> Self {
#                 Self {
#                     id: Uuid::new_v4(),
#                     name: name,
#                     is_active: true,
#                 }
#             }
#         }
# 
# 
#         use sqlx::PgPool;
# 
#         let pool = PgPool::connect("postgresql://orlok:orlok@localhost/orlok")
#             .await?;
# 
#         sqlx::query(
#             "create table if not exists users (
#                 id uuid,
#                 name text,
#                 is_active boolean
#             )"
#         )
#         .execute(&pool)
#         .await?;
#         sqlx::query("delete from users").execute(&pool).await?;
# 
# 
#         use std::collections::HashMap;
#         use sea_query::SimpleExpr;
# 
#         fn dump_user(u: &User) -> HashMap<String, SimpleExpr> {
#             HashMap::from([
#                 ("id".to_string(), u.id.into()),
#                 ("name".to_string(), u.name.clone().into()),
#                 ("is_active".to_string(), u.is_active.into()),
#             ])
#         }
# 
# 
#         use sqlx::Row;
#         use sqlx::postgres::PgRow;
# 
#         fn load_user(row: &PgRow) -> User {
#             User {
#                 id: row.get("id"),
#                 name: row.get("name"),
#                 is_active: row.get("is_active"),
#             }
#         }
# 
# 
#         use orlok::Repo;
#         use orlok::pg::PgRepo;
# 
#         let users_repo = PgRepo::new("users".to_string(), dump_user, load_user);
# 
# 
#         use orlok::Db;
#         use orlok::pg::PgDb;
# 
#         let db = PgDb::new(pool);
# 
# 
#         let alice = User::new("Alice".to_string());
#         users_repo.add(&db, &alice).await?;
#         let bob = User::new("Bob".to_string());
#         users_repo.add(&db, &bob).await?;
#         let eve = User::new("Eve".to_string());
#         users_repo.add(&db, &eve).await?;
# 
# 
#         use orlok::F;
use orlok::Query;

let users = users_repo.get_many(&db, &Query::filter(F::ends_with("name", "e"))).await?;
assert_eq!(users, vec![alice.clone(), eve.clone()]);
#         Ok(())
#     })
# }
```

In addition to the `F` struct, we use the `Query` struct here because it provides
options for the limit, offset, and order of entities that we want to retrieve:

```rust
# use tokio_test;
# fn main() -> anyhow::Result<()> {
#     tokio_test::block_on(async {
#         use uuid::Uuid;
# 
#         #[derive(PartialEq, Clone, Debug)]
#         pub struct User {
#             pub id: Uuid,
#             pub name: String,
#             pub is_active: bool,
#         }
# 
#         impl User {
#             pub fn new(name: String) -> Self {
#                 Self {
#                     id: Uuid::new_v4(),
#                     name: name,
#                     is_active: true,
#                 }
#             }
#         }
# 
# 
#         use sqlx::PgPool;
# 
#         let pool = PgPool::connect("postgresql://orlok:orlok@localhost/orlok")
#             .await?;
# 
#         sqlx::query(
#             "create table if not exists users (
#                 id uuid,
#                 name text,
#                 is_active boolean
#             )"
#         )
#         .execute(&pool)
#         .await?;
#         sqlx::query("delete from users").execute(&pool).await?;
# 
# 
#         use std::collections::HashMap;
#         use sea_query::SimpleExpr;
# 
#         fn dump_user(u: &User) -> HashMap<String, SimpleExpr> {
#             HashMap::from([
#                 ("id".to_string(), u.id.into()),
#                 ("name".to_string(), u.name.clone().into()),
#                 ("is_active".to_string(), u.is_active.into()),
#             ])
#         }
# 
# 
#         use sqlx::Row;
#         use sqlx::postgres::PgRow;
# 
#         fn load_user(row: &PgRow) -> User {
#             User {
#                 id: row.get("id"),
#                 name: row.get("name"),
#                 is_active: row.get("is_active"),
#             }
#         }
# 
# 
#         use orlok::Repo;
#         use orlok::pg::PgRepo;
# 
#         let users_repo = PgRepo::new("users".to_string(), dump_user, load_user);
# 
# 
#         use orlok::Db;
#         use orlok::pg::PgDb;
# 
#         let db = PgDb::new(pool);
# 
# 
#         let alice = User::new("Alice".to_string());
#         users_repo.add(&db, &alice).await?;
#         let bob = User::new("Bob".to_string());
#         users_repo.add(&db, &bob).await?;
#         let eve = User::new("Eve".to_string());
#         users_repo.add(&db, &eve).await?;
# 
# 
#         use orlok::F;
#         use orlok::Query;
use orlok::Order;

let users = users_repo.get_many(
    &db,
    &Query::new()
        .order(vec![Order::Desc("name".to_string())])
        .limit(2)
        .offset(1)
).await?;

assert_eq!(users, vec![bob.clone(), alice.clone()]);
#         Ok(())
#     })
# }
```

### Updating an entity

To update an entity we need to modify it and pass its reference
to the `update` method together with a filter that matches
an appropriate record in the database:

```rust
# use tokio_test;
# fn main() -> anyhow::Result<()> {
#     tokio_test::block_on(async {
#         use uuid::Uuid;
# 
#         #[derive(PartialEq, Clone, Debug)]
#         pub struct User {
#             pub id: Uuid,
#             pub name: String,
#             pub is_active: bool,
#         }
# 
#         impl User {
#             pub fn new(name: String) -> Self {
#                 Self {
#                     id: Uuid::new_v4(),
#                     name: name,
#                     is_active: true,
#                 }
#             }
#         }
# 
# 
#         use sqlx::PgPool;
# 
#         let pool = PgPool::connect("postgresql://orlok:orlok@localhost/orlok")
#             .await?;
# 
#         sqlx::query(
#             "create table if not exists users (
#                 id uuid,
#                 name text,
#                 is_active boolean
#             )"
#         )
#         .execute(&pool)
#         .await?;
#         sqlx::query("delete from users").execute(&pool).await?;
# 
# 
#         use std::collections::HashMap;
#         use sea_query::SimpleExpr;
# 
#         fn dump_user(u: &User) -> HashMap<String, SimpleExpr> {
#             HashMap::from([
#                 ("id".to_string(), u.id.into()),
#                 ("name".to_string(), u.name.clone().into()),
#                 ("is_active".to_string(), u.is_active.into()),
#             ])
#         }
# 
# 
#         use sqlx::Row;
#         use sqlx::postgres::PgRow;
# 
#         fn load_user(row: &PgRow) -> User {
#             User {
#                 id: row.get("id"),
#                 name: row.get("name"),
#                 is_active: row.get("is_active"),
#             }
#         }
# 
# 
#         use orlok::Repo;
#         use orlok::pg::PgRepo;
# 
#         let users_repo = PgRepo::new("users".to_string(), dump_user, load_user);
# 
# 
#         use orlok::Db;
#         use orlok::pg::PgDb;
# 
#         let db = PgDb::new(pool);
# 
# 
#         let alice = User::new("Alice".to_string());
#         users_repo.add(&db, &alice).await?;
#         let bob = User::new("Bob".to_string());
#         users_repo.add(&db, &bob).await?;
#         let eve = User::new("Eve".to_string());
#         users_repo.add(&db, &eve).await?;
# 
# 
#         use orlok::F;
let mut eve = users_repo.get(&db, &F::eq("name", "Eve")).await?.unwrap();
eve.is_active = false;
users_repo.update(&db, &F::eq("id", eve.id), &eve).await?;
#         Ok(())
#     })
# }
```

### Removing an entity

For this we also need a filter:

```rust
# use tokio_test;
# fn main() -> anyhow::Result<()> {
#     tokio_test::block_on(async {
#         use uuid::Uuid;
# 
#         #[derive(PartialEq, Clone, Debug)]
#         pub struct User {
#             pub id: Uuid,
#             pub name: String,
#             pub is_active: bool,
#         }
# 
#         impl User {
#             pub fn new(name: String) -> Self {
#                 Self {
#                     id: Uuid::new_v4(),
#                     name: name,
#                     is_active: true,
#                 }
#             }
#         }
# 
# 
#         use sqlx::PgPool;
# 
#         let pool = PgPool::connect("postgresql://orlok:orlok@localhost/orlok")
#             .await?;
# 
#         sqlx::query(
#             "create table if not exists users (
#                 id uuid,
#                 name text,
#                 is_active boolean
#             )"
#         )
#         .execute(&pool)
#         .await?;
#         sqlx::query("delete from users").execute(&pool).await?;
# 
# 
#         use std::collections::HashMap;
#         use sea_query::SimpleExpr;
# 
#         fn dump_user(u: &User) -> HashMap<String, SimpleExpr> {
#             HashMap::from([
#                 ("id".to_string(), u.id.into()),
#                 ("name".to_string(), u.name.clone().into()),
#                 ("is_active".to_string(), u.is_active.into()),
#             ])
#         }
# 
# 
#         use sqlx::Row;
#         use sqlx::postgres::PgRow;
# 
#         fn load_user(row: &PgRow) -> User {
#             User {
#                 id: row.get("id"),
#                 name: row.get("name"),
#                 is_active: row.get("is_active"),
#             }
#         }
# 
# 
#         use orlok::Repo;
#         use orlok::pg::PgRepo;
# 
#         let users_repo = PgRepo::new("users".to_string(), dump_user, load_user);
# 
# 
#         use orlok::Db;
#         use orlok::pg::PgDb;
# 
#         let db = PgDb::new(pool);
# 
# 
#         let alice = User::new("Alice".to_string());
#         users_repo.add(&db, &alice).await?;
#         let bob = User::new("Bob".to_string());
#         users_repo.add(&db, &bob).await?;
#         let eve = User::new("Eve".to_string());
#         users_repo.add(&db, &eve).await?;
# 
# 
#         use orlok::F;
#         let mut eve = users_repo.get(&db, &F::eq("name", "Eve")).await?.unwrap();
#         eve.is_active = false;
#         users_repo.update(&db, &F::eq("id", eve.id), &eve).await?;
users_repo.delete(&db, &F::eq("is_active", false)).await?;
#         Ok(())
#     })
# }
```

### Transactions

Use a closure to execute code in a transaction and return `Ok` to complete the transaction or an error to abort it:

```rust
# use tokio_test;
# fn main() -> anyhow::Result<()> {
#     tokio_test::block_on(async {
#         use uuid::Uuid;
# 
#         #[derive(PartialEq, Clone, Debug)]
#         pub struct User {
#             pub id: Uuid,
#             pub name: String,
#             pub is_active: bool,
#         }
# 
#         impl User {
#             pub fn new(name: String) -> Self {
#                 Self {
#                     id: Uuid::new_v4(),
#                     name: name,
#                     is_active: true,
#                 }
#             }
#         }
# 
# 
#         use sqlx::PgPool;
# 
#         let pool = PgPool::connect("postgresql://orlok:orlok@localhost/orlok")
#             .await?;
# 
#         sqlx::query(
#             "create table if not exists users (
#                 id uuid,
#                 name text,
#                 is_active boolean
#             )"
#         )
#         .execute(&pool)
#         .await?;
#         sqlx::query("delete from users").execute(&pool).await?;
# 
# 
#         use std::collections::HashMap;
#         use sea_query::SimpleExpr;
# 
#         fn dump_user(u: &User) -> HashMap<String, SimpleExpr> {
#             HashMap::from([
#                 ("id".to_string(), u.id.into()),
#                 ("name".to_string(), u.name.clone().into()),
#                 ("is_active".to_string(), u.is_active.into()),
#             ])
#         }
# 
# 
#         use sqlx::Row;
#         use sqlx::postgres::PgRow;
# 
#         fn load_user(row: &PgRow) -> User {
#             User {
#                 id: row.get("id"),
#                 name: row.get("name"),
#                 is_active: row.get("is_active"),
#             }
#         }
# 
# 
#         use orlok::Repo;
#         use orlok::pg::PgRepo;
# 
#         let users_repo = PgRepo::new("users".to_string(), dump_user, load_user);
# 
# 
#         use orlok::Db;
#         use orlok::pg::PgDb;
# 
#         let db = PgDb::new(pool);
# 
# 
#         let alice = User::new("Alice".to_string());
#         users_repo.add(&db, &alice).await?;
#         let bob = User::new("Bob".to_string());
#         users_repo.add(&db, &bob).await?;
#         let eve = User::new("Eve".to_string());
#         users_repo.add(&db, &eve).await?;
# 
# 
#         use orlok::F;
db.transaction(|tx| {
    Box::pin({
        let users_repo = users_repo.clone();
        async move {
            let mut user1 = users_repo.get_for_update(&tx, &F::eq("name", "Alice")).await?.unwrap();
            let mut user2 = users_repo.get_for_update(&tx, &F::eq("name", "Bob")).await?.unwrap();
            user1.name = "Bob".to_string();
            user2.name = "Alice".to_string();
            users_repo.update(&tx, &F::eq("id", user1.id), &user1).await?;
            users_repo.update(&tx, &F::eq("id", user2.id), &user2).await?;
            Ok(())
        }
    })
});
#         Ok(())
#     })
# }
```

Be aware that nested transactions are not supported at the moment.


### Fast prototyping

If you don't have time to think about a database schema
but want to try some ideas
you can use an alternative repository implementation
that stores records in memory as a collection of JSON objects.

```rust
# use tokio_test;
# fn main() -> anyhow::Result<()> {
    # tokio_test::block_on(async {
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tokio::sync::RwLock;
use orlok::{Repo, Db};
use orlok::json::{JsonRepo, JsonDb};

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub is_active: bool,
}

impl User {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name,
            is_active: true,
        }
    }
}

let users_repo = JsonRepo::new("users".to_string());

let data = RwLock::new(HashMap::new());
let db = JsonDb::new(data);

let user = User::new("Alice".to_string());
users_repo.add(&db, &user).await?;
Ok(())
    # })
# }
```
