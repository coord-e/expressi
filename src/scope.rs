use error::TranslationError;

use std::collections::HashMap;
use std::iter::{DoubleEndedIterator, FromIterator};
use std::ops::Deref;

use failure::Error;

pub trait Scope {
    type V: Clone;

    fn data(&self) -> HashMap<&String, &Self::V>;
    fn insert(&mut self, key: &str, val: Self::V);
    fn get(&self, key: &str) -> Option<Self::V>;
}

#[derive(Debug, Clone)]
pub struct Env<T>(HashMap<String, T>);

impl<T> Env<T> {
    pub fn new() -> Self {
        Env(HashMap::new())
    }
}

impl<T> Deref for Env<T> {
    type Target = HashMap<String, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> FromIterator<(String, T)> for Env<T> {
    fn from_iter<I: IntoIterator<Item = (String, T)>>(iter: I) -> Self {
        Env(iter.into_iter().collect())
    }
}

impl<T> Scope for Env<T>
where
    T: Clone,
{
    type V = T;

    fn data(&self) -> HashMap<&String, &Self::V> {
        self.0.iter().map(|(k, v)| (k, v)).collect()
    }

    fn insert(&mut self, key: &str, val: Self::V) {
        self.0.insert(key.to_string(), val);
    }

    fn get(&self, key: &str) -> Option<Self::V> {
        self.0.get(&key.to_string()).cloned()
    }
}

#[derive(Debug, Clone)]
pub struct ScopedEnv<T>(Vec<Env<T>>);

impl<T> ScopedEnv<T> {
    pub fn new() -> Self {
        ScopedEnv(vec![Env::new()])
    }

    pub fn new_scope(&self) -> Env<T> {
        Env::new()
    }

    pub fn push(&mut self, sc: Env<T>) {
        self.0.push(sc)
    }

    pub fn pop(&mut self) -> Result<Env<T>, Error> {
        if self.0.len() == 1 {
            return Err(TranslationError::UnexpectedScopePop.into());
        }
        self.0
            .pop()
            .ok_or(TranslationError::UnexpectedScopePop.into())
    }

    pub fn unique_name(&self, s: &str) -> String {
        let num_vars = self.0.len();
        format!("{}.{}", s, num_vars)
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Env<T>> {
        self.0.iter()
    }
}

impl<T> FromIterator<Env<T>> for ScopedEnv<T> {
    fn from_iter<I: IntoIterator<Item = Env<T>>>(iter: I) -> Self {
        ScopedEnv(iter.into_iter().collect())
    }
}

impl<T> Scope for ScopedEnv<T>
where
    T: Clone,
{
    type V = T;

    fn data(&self) -> HashMap<&String, &Self::V> {
        self.0.iter().flat_map(|env| env.0.iter()).collect()
    }

    fn insert(&mut self, key: &str, val: Self::V) {
        self.0.last_mut().unwrap().0.insert(key.to_string(), val);
    }

    fn get(&self, key: &str) -> Option<Self::V> {
        self.data().get(&key.to_string()).cloned().cloned()
    }
}
