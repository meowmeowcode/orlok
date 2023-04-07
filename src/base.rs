use crate::query::{Query, F};

pub trait Repo<T> {
    fn get(&self, filter: &F) -> Option<T>;
    fn get_many(&self, query: &Query) -> Vec<T>;
    fn add(&self, entity: &T);
    fn update(&self, filter: &F, entity: &T);
    fn delete(&self, filter: &F);
}
