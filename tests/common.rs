use serde::{Deserialize, Serialize};

use orlok::base::Repo;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub name: String,
}

impl User {
    pub fn new(id: i64, name: &str) -> Self {
        Self {
            id: id,
            name: name.to_string(),
        }
    }
}

pub fn add_alice(repo: &impl Repo<User>) -> User {
    let u = User::new(1, "Alice");
    repo.add(&u);
    u
}

pub fn add_bob(repo: &impl Repo<User>) -> User {
    let u = User::new(2, "Bob");
    repo.add(&u);
    u
}

pub fn add_eve(repo: &impl Repo<User>) -> User {
    let u = User::new(3, "Eve");
    repo.add(&u);
    u
}
