use chrono::TimeZone;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use orlok::base::Repo;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub age: i64,
    pub is_evil: bool,
    pub weight: Option<f64>,
    pub registered_at: DateTime<Utc>,
    pub money: Decimal,
}

impl User {
    pub fn new(name: &str, age: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            age: age,
            is_evil: false,
            weight: None,
            money: dec!(0.0),
            registered_at: Utc::now(),
        }
    }
}

#[allow(dead_code)]
pub async fn add_alice<X>(repo: &impl Repo<User, Transaction = X>) -> User {
    let mut u = User::new("Alice", 24);
    u.money = dec!(130.50);
    u.registered_at = Utc.with_ymd_and_hms(2018, 3, 1, 0, 0, 0).unwrap();
    u.weight = Some(70.5);
    repo.add(&u).await.unwrap();
    u
}

#[allow(dead_code)]
pub async fn add_bob<X>(repo: &impl Repo<User, Transaction = X>) -> User {
    let mut u = User::new("Bob", 29);
    u.money = dec!(150.06);
    u.registered_at = Utc.with_ymd_and_hms(2019, 2, 2, 0, 0, 0).unwrap();
    u.weight = Some(83.4);
    repo.add(&u).await.unwrap();
    u
}

#[allow(dead_code)]
pub async fn add_eve<X>(repo: &impl Repo<User, Transaction = X>) -> User {
    let mut u = User::new("Eve", 31);
    u.money = dec!(230.25);
    u.registered_at = Utc.with_ymd_and_hms(2020, 1, 3, 0, 0, 0).unwrap();
    u.is_evil = true;
    repo.add(&u).await.unwrap();
    u
}
