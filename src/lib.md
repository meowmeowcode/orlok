<div style="text-align: center">
    <img src="https://github.com/meowmeowcode/orlok/raw/main/orlok.png" width="200" alt="Orlok" />
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

Suppose we have a struct representing a character of some story:

```rust
use uuid::Uuid;

#[derive(PartialEq, Clone, Debug)]
pub struct Character {
    pub id: Uuid,
    pub name: String,
    pub location: String,
}

impl Character {
    pub fn new(name: String, location: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            location,
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
    "create table if not exists characters (
        id uuid,
        name text,
        location text
    )"
)
.execute(&pool)
.await?;
#         sqlx::query("delete from characters").execute(&pool).await?;
#         Ok(())
#     })
# }
```

To save the character struct to the database we need
to somehow map its fields to a table row.
We can define a function for this.
This function must return a `HashMap` with keys and values
that correspond to columns and values in the "characters" table:

```rust
#         use uuid::Uuid;
# 
#         #[derive(PartialEq, Clone, Debug)]
#         pub struct Character {
#             pub id: Uuid,
#             pub name: String,
#             pub location: String,
#         }
use std::collections::HashMap;
use sea_query::SimpleExpr;

fn dump_character(u: &Character) -> HashMap<String, SimpleExpr> {
    HashMap::from([
        ("id".to_string(), u.id.into()),
        ("name".to_string(), u.name.clone().into()),
        ("location".to_string(), u.location.clone().into()),
    ])
}
```

After saving the character struct to the database,
we want to be able to load it back,
so we need a function that maps
a database row to the struct:

```rust
#         use uuid::Uuid;
#
#         #[derive(PartialEq, Clone, Debug)]
#         pub struct Character {
#             pub id: Uuid,
#             pub name: String,
#             pub location: String,
#         }
use sqlx::Row;
use sqlx::postgres::PgRow;

fn load_character(row: &PgRow) -> Character {
    Character {
        id: row.get("id"),
        name: row.get("name"),
        location: row.get("location"),
    }
}
```

After this, we can create a repository:

```rust
#         use uuid::Uuid;
# 
#         #[derive(PartialEq, Clone, Debug)]
#         pub struct Character {
#             pub id: Uuid,
#             pub name: String,
#             pub location: String,
#         }
#         use std::collections::HashMap;
#         use sea_query::SimpleExpr;
# 
#         fn dump_character(u: &Character) -> HashMap<String, SimpleExpr> {
#             HashMap::from([
#                 ("id".to_string(), u.id.into()),
#                 ("name".to_string(), u.name.clone().into()),
#                 ("location".to_string(), u.location.clone().into()),
#             ])
#         }
# 
# 
#         use sqlx::Row;
#         use sqlx::postgres::PgRow;
# 
#         fn load_character(row: &PgRow) -> Character {
#             Character {
#                 id: row.get("id"),
#                 name: row.get("name"),
#                 location: row.get("location"),
#             }
#         }
# 
# 
use orlok::Repo;
use orlok::pg::PgRepo;

let characters_repo = PgRepo::new("characters", dump_character, load_character);
```

The first argument for the `new` function is the name of the table where we want to store our characters.

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

Now we can save new characters:

```rust
# use tokio_test;
# fn main() -> anyhow::Result<()> {
#     tokio_test::block_on(async {
#         use uuid::Uuid;
# 
#         #[derive(PartialEq, Clone, Debug)]
#         pub struct Character {
#             pub id: Uuid,
#             pub name: String,
#             pub location: String,
#         }
# 
#         impl Character {
#             pub fn new(name: String, location: String) -> Self {
#                 Self {
#                     id: Uuid::new_v4(),
#                     name,
#                     location,
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
#             "create table if not exists characters (
#                 id uuid,
#                 name text,
#                 location text
#             )"
#         )
#         .execute(&pool)
#         .await?;
#         sqlx::query("delete from characters").execute(&pool).await?;
# 
# 
#         use std::collections::HashMap;
#         use sea_query::SimpleExpr;
# 
#         fn dump_character(u: &Character) -> HashMap<String, SimpleExpr> {
#             HashMap::from([
#                 ("id".to_string(), u.id.into()),
#                 ("name".to_string(), u.name.clone().into()),
#                 ("location".to_string(), u.location.clone().into()),
#             ])
#         }
# 
# 
#         use sqlx::Row;
#         use sqlx::postgres::PgRow;
# 
#         fn load_character(row: &PgRow) -> Character {
#             Character {
#                 id: row.get("id"),
#                 name: row.get("name"),
#                 location: row.get("location"),
#             }
#         }
# 
# 
#         use orlok::Repo;
#         use orlok::pg::PgRepo;
# 
#         let characters_repo = PgRepo::new("characters", dump_character, load_character);
# 
# 
#         use orlok::Db;
#         use orlok::pg::PgDb;
# 
#         let db = PgDb::new(pool);
# 
# 
let orlok = Character::new("Orlok".to_string(), "Transylvania".to_string());
characters_repo.add(&db, &orlok).await?;
let thomas = Character::new("Thomas".to_string(), "Wisborg".to_string());
characters_repo.add(&db, &thomas).await?;
let ellen = Character::new("Ellen".to_string(), "Wisborg".to_string());
characters_repo.add(&db, &ellen).await?;
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
#         pub struct Character {
#             pub id: Uuid,
#             pub name: String,
#             pub location: String,
#         }
# 
#         impl Character {
#             pub fn new(name: String, location: String) -> Self {
#                 Self {
#                     id: Uuid::new_v4(),
#                     name,
#                     location,
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
#             "create table if not exists characters (
#                 id uuid,
#                 name text,
#                 location text
#             )"
#         )
#         .execute(&pool)
#         .await?;
#         sqlx::query("delete from characters").execute(&pool).await?;
# 
# 
#         use std::collections::HashMap;
#         use sea_query::SimpleExpr;
# 
#         fn dump_character(u: &Character) -> HashMap<String, SimpleExpr> {
#             HashMap::from([
#                 ("id".to_string(), u.id.into()),
#                 ("name".to_string(), u.name.clone().into()),
#                 ("location".to_string(), u.location.clone().into()),
#             ])
#         }
# 
# 
#         use sqlx::Row;
#         use sqlx::postgres::PgRow;
# 
#         fn load_character(row: &PgRow) -> Character {
#             Character {
#                 id: row.get("id"),
#                 name: row.get("name"),
#                 location: row.get("location"),
#             }
#         }
# 
# 
#         use orlok::Repo;
#         use orlok::pg::PgRepo;
# 
#         let characters_repo = PgRepo::new("characters", dump_character, load_character);
# 
# 
#         use orlok::Db;
#         use orlok::pg::PgDb;
# 
#         let db = PgDb::new(pool);
# 
# 
#         let orlok = Character::new("Orlok".to_string(), "Transylvania".to_string());
#         characters_repo.add(&db, &orlok).await?;
#         let thomas = Character::new("Thomas".to_string(), "Wisborg".to_string());
#         characters_repo.add(&db, &thomas).await?;
#         let ellen = Character::new("Ellen".to_string(), "Wisborg".to_string());
#         characters_repo.add(&db, &ellen).await?;
# 
# 
use orlok::F;

let character = characters_repo.get(&db, &F::eq("name", "Orlok")).await?.unwrap();
assert_eq!(character, orlok);
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
#         pub struct Character {
#             pub id: Uuid,
#             pub name: String,
#             pub location: String,
#         }
# 
#         impl Character {
#             pub fn new(name: String, location: String) -> Self {
#                 Self {
#                     id: Uuid::new_v4(),
#                     name,
#                     location,
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
#             "create table if not exists characters (
#                 id uuid,
#                 name text,
#                 location text
#             )"
#         )
#         .execute(&pool)
#         .await?;
#         sqlx::query("delete from characters").execute(&pool).await?;
# 
# 
#         use std::collections::HashMap;
#         use sea_query::SimpleExpr;
# 
#         fn dump_character(u: &Character) -> HashMap<String, SimpleExpr> {
#             HashMap::from([
#                 ("id".to_string(), u.id.into()),
#                 ("name".to_string(), u.name.clone().into()),
#                 ("location".to_string(), u.location.clone().into()),
#             ])
#         }
# 
# 
#         use sqlx::Row;
#         use sqlx::postgres::PgRow;
# 
#         fn load_character(row: &PgRow) -> Character {
#             Character {
#                 id: row.get("id"),
#                 name: row.get("name"),
#                 location: row.get("location"),
#             }
#         }
# 
# 
#         use orlok::Repo;
#         use orlok::pg::PgRepo;
# 
#         let characters_repo = PgRepo::new("characters", dump_character, load_character);
# 
# 
#         use orlok::Db;
#         use orlok::pg::PgDb;
# 
#         let db = PgDb::new(pool);
# 
# 
#         let orlok = Character::new("Orlok".to_string(), "Transylvania".to_string());
#         characters_repo.add(&db, &orlok).await?;
#         let thomas = Character::new("Thomas".to_string(), "Wisborg".to_string());
#         characters_repo.add(&db, &thomas).await?;
#         let ellen = Character::new("Ellen".to_string(), "Wisborg".to_string());
#         characters_repo.add(&db, &ellen).await?;
# 
# 
#         use orlok::F;
let character = characters_repo.get(&db, &F::eq("name", "Knock")).await?;
assert!(character.is_none());
#         Ok(())
#     })
# }
```

Note that here we use the `F` struct for filtering entities.
It has different methods for different conditions.
For example, if we want to find a character with
the letter "h" in their name, we can do something like this:

```rust
# use tokio_test;
# fn main() -> anyhow::Result<()> {
#     tokio_test::block_on(async {
#         use uuid::Uuid;
# 
#         #[derive(PartialEq, Clone, Debug)]
#         pub struct Character {
#             pub id: Uuid,
#             pub name: String,
#             pub location: String,
#         }
# 
#         impl Character {
#             pub fn new(name: String, location: String) -> Self {
#                 Self {
#                     id: Uuid::new_v4(),
#                     name,
#                     location,
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
#             "create table if not exists characters (
#                 id uuid,
#                 name text,
#                 location text
#             )"
#         )
#         .execute(&pool)
#         .await?;
#         sqlx::query("delete from characters").execute(&pool).await?;
# 
# 
#         use std::collections::HashMap;
#         use sea_query::SimpleExpr;
# 
#         fn dump_character(u: &Character) -> HashMap<String, SimpleExpr> {
#             HashMap::from([
#                 ("id".to_string(), u.id.into()),
#                 ("name".to_string(), u.name.clone().into()),
#                 ("location".to_string(), u.location.clone().into()),
#             ])
#         }
# 
# 
#         use sqlx::Row;
#         use sqlx::postgres::PgRow;
# 
#         fn load_character(row: &PgRow) -> Character {
#             Character {
#                 id: row.get("id"),
#                 name: row.get("name"),
#                 location: row.get("location"),
#             }
#         }
# 
# 
#         use orlok::Repo;
#         use orlok::pg::PgRepo;
# 
#         let characters_repo = PgRepo::new("characters", dump_character, load_character);
# 
# 
#         use orlok::Db;
#         use orlok::pg::PgDb;
# 
#         let db = PgDb::new(pool);
# 
# 
#         let orlok = Character::new("Orlok".to_string(), "Transylvania".to_string());
#         characters_repo.add(&db, &orlok).await?;
#         let thomas = Character::new("Thomas".to_string(), "Wisborg".to_string());
#         characters_repo.add(&db, &thomas).await?;
#         let ellen = Character::new("Ellen".to_string(), "Wisborg".to_string());
#         characters_repo.add(&db, &ellen).await?;
# 
# 
#         use orlok::F;
let character = characters_repo.get(&db, &F::contains("name", "h")).await?.unwrap();
assert_eq!(character, thomas);
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
#         pub struct Character {
#             pub id: Uuid,
#             pub name: String,
#             pub location: String,
#         }
# 
#         impl Character {
#             pub fn new(name: String, location: String) -> Self {
#                 Self {
#                     id: Uuid::new_v4(),
#                     name,
#                     location,
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
#             "create table if not exists characters (
#                 id uuid,
#                 name text,
#                 location text
#             )"
#         )
#         .execute(&pool)
#         .await?;
#         sqlx::query("delete from characters").execute(&pool).await?;
# 
# 
#         use std::collections::HashMap;
#         use sea_query::SimpleExpr;
# 
#         fn dump_character(u: &Character) -> HashMap<String, SimpleExpr> {
#             HashMap::from([
#                 ("id".to_string(), u.id.into()),
#                 ("name".to_string(), u.name.clone().into()),
#                 ("location".to_string(), u.location.clone().into()),
#             ])
#         }
# 
# 
#         use sqlx::Row;
#         use sqlx::postgres::PgRow;
# 
#         fn load_character(row: &PgRow) -> Character {
#             Character {
#                 id: row.get("id"),
#                 name: row.get("name"),
#                 location: row.get("location"),
#             }
#         }
# 
# 
#         use orlok::Repo;
#         use orlok::pg::PgRepo;
# 
#         let characters_repo = PgRepo::new("characters", dump_character, load_character);
# 
# 
#         use orlok::Db;
#         use orlok::pg::PgDb;
# 
#         let db = PgDb::new(pool);
# 
# 
#         let orlok = Character::new("Orlok".to_string(), "Transylvania".to_string());
#         characters_repo.add(&db, &orlok).await?;
#         let thomas = Character::new("Thomas".to_string(), "Wisborg".to_string());
#         characters_repo.add(&db, &thomas).await?;
#         let ellen = Character::new("Ellen".to_string(), "Wisborg".to_string());
#         characters_repo.add(&db, &ellen).await?;
# 
# 
#         use orlok::F;
let character = characters_repo.get(
    &db,
    &F::and(
        vec![
            F::starts_with("name", "E"),
            F::ends_with("name", "n")
        ]
    )
).await?.unwrap();

assert_eq!(character, ellen);
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
#         pub struct Character {
#             pub id: Uuid,
#             pub name: String,
#             pub location: String,
#         }
# 
#         impl Character {
#             pub fn new(name: String, location: String) -> Self {
#                 Self {
#                     id: Uuid::new_v4(),
#                     name,
#                     location,
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
#             "create table if not exists characters (
#                 id uuid,
#                 name text,
#                 location text
#             )"
#         )
#         .execute(&pool)
#         .await?;
#         sqlx::query("delete from characters").execute(&pool).await?;
# 
# 
#         use std::collections::HashMap;
#         use sea_query::SimpleExpr;
# 
#         fn dump_character(u: &Character) -> HashMap<String, SimpleExpr> {
#             HashMap::from([
#                 ("id".to_string(), u.id.into()),
#                 ("name".to_string(), u.name.clone().into()),
#                 ("location".to_string(), u.location.clone().into()),
#             ])
#         }
# 
# 
#         use sqlx::Row;
#         use sqlx::postgres::PgRow;
# 
#         fn load_character(row: &PgRow) -> Character {
#             Character {
#                 id: row.get("id"),
#                 name: row.get("name"),
#                 location: row.get("location"),
#             }
#         }
# 
# 
#         use orlok::Repo;
#         use orlok::pg::PgRepo;
# 
#         let characters_repo = PgRepo::new("characters", dump_character, load_character);
# 
# 
#         use orlok::Db;
#         use orlok::pg::PgDb;
# 
#         let db = PgDb::new(pool);
# 
# 
#         let orlok = Character::new("Orlok".to_string(), "Transylvania".to_string());
#         characters_repo.add(&db, &orlok).await?;
#         let thomas = Character::new("Thomas".to_string(), "Wisborg".to_string());
#         characters_repo.add(&db, &thomas).await?;
#         let ellen = Character::new("Ellen".to_string(), "Wisborg".to_string());
#         characters_repo.add(&db, &ellen).await?;
# 
# 
#         use orlok::F;
use orlok::Query;

let characters = characters_repo.get_many(
    &db,
    &Query::filter(F::contains("name", "l"))
).await?;

assert_eq!(characters, vec![orlok.clone(), ellen.clone()]);
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
#         pub struct Character {
#             pub id: Uuid,
#             pub name: String,
#             pub location: String,
#         }
# 
#         impl Character {
#             pub fn new(name: String, location: String) -> Self {
#                 Self {
#                     id: Uuid::new_v4(),
#                     name,
#                     location,
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
#             "create table if not exists characters (
#                 id uuid,
#                 name text,
#                 location text
#             )"
#         )
#         .execute(&pool)
#         .await?;
#         sqlx::query("delete from characters").execute(&pool).await?;
# 
# 
#         use std::collections::HashMap;
#         use sea_query::SimpleExpr;
# 
#         fn dump_character(u: &Character) -> HashMap<String, SimpleExpr> {
#             HashMap::from([
#                 ("id".to_string(), u.id.into()),
#                 ("name".to_string(), u.name.clone().into()),
#                 ("location".to_string(), u.location.clone().into()),
#             ])
#         }
# 
# 
#         use sqlx::Row;
#         use sqlx::postgres::PgRow;
# 
#         fn load_character(row: &PgRow) -> Character {
#             Character {
#                 id: row.get("id"),
#                 name: row.get("name"),
#                 location: row.get("location"),
#             }
#         }
# 
# 
#         use orlok::Repo;
#         use orlok::pg::PgRepo;
# 
#         let characters_repo = PgRepo::new("characters", dump_character, load_character);
# 
# 
#         use orlok::Db;
#         use orlok::pg::PgDb;
# 
#         let db = PgDb::new(pool);
# 
# 
#         let orlok = Character::new("Orlok".to_string(), "Transylvania".to_string());
#         characters_repo.add(&db, &orlok).await?;
#         let thomas = Character::new("Thomas".to_string(), "Wisborg".to_string());
#         characters_repo.add(&db, &thomas).await?;
#         let ellen = Character::new("Ellen".to_string(), "Wisborg".to_string());
#         characters_repo.add(&db, &ellen).await?;
# 
# 
#         use orlok::F;
#         use orlok::Query;
use orlok::Order;

let characters = characters_repo.get_many(
    &db,
    &Query::new()
        .order(vec![Order::Desc("name".to_string())])
        .limit(2)
        .offset(1)
).await?;

assert_eq!(characters, vec![orlok.clone(), ellen.clone()]);
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
#         pub struct Character {
#             pub id: Uuid,
#             pub name: String,
#             pub location: String,
#         }
# 
#         impl Character {
#             pub fn new(name: String, location: String) -> Self {
#                 Self {
#                     id: Uuid::new_v4(),
#                     name,
#                     location,
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
#             "create table if not exists characters (
#                 id uuid,
#                 name text,
#                 location text
#             )"
#         )
#         .execute(&pool)
#         .await?;
#         sqlx::query("delete from characters").execute(&pool).await?;
# 
# 
#         use std::collections::HashMap;
#         use sea_query::SimpleExpr;
# 
#         fn dump_character(u: &Character) -> HashMap<String, SimpleExpr> {
#             HashMap::from([
#                 ("id".to_string(), u.id.into()),
#                 ("name".to_string(), u.name.clone().into()),
#                 ("location".to_string(), u.location.clone().into()),
#             ])
#         }
# 
# 
#         use sqlx::Row;
#         use sqlx::postgres::PgRow;
# 
#         fn load_character(row: &PgRow) -> Character {
#             Character {
#                 id: row.get("id"),
#                 name: row.get("name"),
#                 location: row.get("location"),
#             }
#         }
# 
# 
#         use orlok::Repo;
#         use orlok::pg::PgRepo;
# 
#         let characters_repo = PgRepo::new("characters", dump_character, load_character);
# 
# 
#         use orlok::Db;
#         use orlok::pg::PgDb;
# 
#         let db = PgDb::new(pool);
# 
# 
#         let orlok = Character::new("Orlok".to_string(), "Transylvania".to_string());
#         characters_repo.add(&db, &orlok).await?;
#         let thomas = Character::new("Thomas".to_string(), "Wisborg".to_string());
#         characters_repo.add(&db, &thomas).await?;
#         let ellen = Character::new("Ellen".to_string(), "Wisborg".to_string());
#         characters_repo.add(&db, &ellen).await?;
# 
# 
#         use orlok::F;
let mut orlok = characters_repo.get(&db, &F::eq("name", "Orlok")).await?.unwrap();
orlok.name = "Count Orlok".to_string();
characters_repo.update(&db, &F::eq("id", orlok.id), &orlok).await?;
#         assert!(!characters_repo.exists(&db, &F::eq("name", "Orlok")).await?);
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
#         pub struct Character {
#             pub id: Uuid,
#             pub name: String,
#             pub location: String,
#         }
# 
#         impl Character {
#             pub fn new(name: String, location: String) -> Self {
#                 Self {
#                     id: Uuid::new_v4(),
#                     name,
#                     location,
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
#             "create table if not exists characters (
#                 id uuid,
#                 name text,
#                 location text
#             )"
#         )
#         .execute(&pool)
#         .await?;
#         sqlx::query("delete from characters").execute(&pool).await?;
# 
# 
#         use std::collections::HashMap;
#         use sea_query::SimpleExpr;
# 
#         fn dump_character(u: &Character) -> HashMap<String, SimpleExpr> {
#             HashMap::from([
#                 ("id".to_string(), u.id.into()),
#                 ("name".to_string(), u.name.clone().into()),
#                 ("location".to_string(), u.location.clone().into()),
#             ])
#         }
# 
# 
#         use sqlx::Row;
#         use sqlx::postgres::PgRow;
# 
#         fn load_character(row: &PgRow) -> Character {
#             Character {
#                 id: row.get("id"),
#                 name: row.get("name"),
#                 location: row.get("location"),
#             }
#         }
# 
# 
#         use orlok::Repo;
#         use orlok::pg::PgRepo;
# 
#         let characters_repo = PgRepo::new("characters", dump_character, load_character);
# 
# 
#         use orlok::Db;
#         use orlok::pg::PgDb;
# 
#         let db = PgDb::new(pool);
# 
# 
#         let orlok = Character::new("Orlok".to_string(), "Transylvania".to_string());
#         characters_repo.add(&db, &orlok).await?;
#         let thomas = Character::new("Thomas".to_string(), "Wisborg".to_string());
#         characters_repo.add(&db, &thomas).await?;
#         let ellen = Character::new("Ellen".to_string(), "Wisborg".to_string());
#         characters_repo.add(&db, &ellen).await?;
# 
# 
#         use orlok::F;
#         let mut orlok = characters_repo.get(&db, &F::eq("name", "Orlok")).await?.unwrap();
#         orlok.name = "Count Orlok".to_string();
#         characters_repo.update(&db, &F::eq("id", orlok.id), &orlok).await?;
db.transaction(|tx| {
    Box::pin({
        let characters_repo = characters_repo.clone();
        async move {
            let mut thomas = characters_repo.get_for_update(
                &tx, &F::eq("name", "Thomas")
            ).await?.unwrap();
            let mut orlok = characters_repo.get_for_update(
                &tx, &F::eq("name", "Count Orlok")
            ).await?.unwrap();
            thomas.location = "Transylvania".to_string();
            orlok.location = "Wisborg".to_string();
            characters_repo.update(&tx, &F::eq("id", thomas.id), &thomas).await?;
            characters_repo.update(&tx, &F::eq("id", orlok.id), &orlok).await?;
            Ok(())
        }
    })
}).await?;
#         Ok(())
#     })
# }
```

Be aware that nested transactions are not supported at the moment.

### Removing an entity

For this we also need a filter:

```rust
# use tokio_test;
# fn main() -> anyhow::Result<()> {
#     tokio_test::block_on(async {
#         use uuid::Uuid;
# 
#         #[derive(PartialEq, Clone, Debug)]
#         pub struct Character {
#             pub id: Uuid,
#             pub name: String,
#             pub location: String,
#         }
# 
#         impl Character {
#             pub fn new(name: String, location: String) -> Self {
#                 Self {
#                     id: Uuid::new_v4(),
#                     name,
#                     location,
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
#             "create table if not exists characters (
#                 id uuid,
#                 name text,
#                 location text
#             )"
#         )
#         .execute(&pool)
#         .await?;
#         sqlx::query("delete from characters").execute(&pool).await?;
# 
# 
#         use std::collections::HashMap;
#         use sea_query::SimpleExpr;
# 
#         fn dump_character(u: &Character) -> HashMap<String, SimpleExpr> {
#             HashMap::from([
#                 ("id".to_string(), u.id.into()),
#                 ("name".to_string(), u.name.clone().into()),
#                 ("location".to_string(), u.location.clone().into()),
#             ])
#         }
# 
# 
#         use sqlx::Row;
#         use sqlx::postgres::PgRow;
# 
#         fn load_character(row: &PgRow) -> Character {
#             Character {
#                 id: row.get("id"),
#                 name: row.get("name"),
#                 location: row.get("location"),
#             }
#         }
# 
# 
#         use orlok::Repo;
#         use orlok::pg::PgRepo;
# 
#         let characters_repo = PgRepo::new("characters", dump_character, load_character);
# 
# 
#         use orlok::Db;
#         use orlok::pg::PgDb;
# 
#         let db = PgDb::new(pool);
# 
# 
#         let orlok = Character::new("Orlok".to_string(), "Transylvania".to_string());
#         characters_repo.add(&db, &orlok).await?;
#         let thomas = Character::new("Thomas".to_string(), "Wisborg".to_string());
#         characters_repo.add(&db, &thomas).await?;
#         let ellen = Character::new("Ellen".to_string(), "Wisborg".to_string());
#         characters_repo.add(&db, &ellen).await?;
# 
# 
#         use orlok::F;
#         let mut orlok = characters_repo.get(&db, &F::eq("name", "Orlok")).await?.unwrap();
#         orlok.name = "Count Orlok".to_string();
#         characters_repo.update(&db, &F::eq("id", orlok.id), &orlok).await?;
#         assert!(characters_repo.exists(&db, &F::eq("name", "Count Orlok")).await?);
characters_repo.delete(&db, &F::eq("name", "Count Orlok")).await?;
#         assert!(!characters_repo.exists(&db, &F::eq("name", "Count Orlok")).await?);
#         Ok(())
#     })
# }
```

### Fast prototyping

If you don't have time to think about a database schema
but want to try some ideas
you can use an alternative repository implementation
that stores records in memory as a collection of JSON objects.

```rust
# use tokio_test;
# fn main() -> anyhow::Result<()> {
    # tokio_test::block_on(async {
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use orlok::{Repo, Db};
use orlok::json::{JsonRepo, JsonDb};

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Character {
    pub id: Uuid,
    pub name: String,
    pub location: String,
}

impl Character {
    pub fn new(name: String, location: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            location,
        }
    }
}

let characters_repo = JsonRepo::new("characters");
let db = JsonDb::new();
let character = Character::new("Orlok".to_string(), "Transylvania".to_string());
characters_repo.add(&db, &character).await?;
Ok(())
    # })
# }
```
