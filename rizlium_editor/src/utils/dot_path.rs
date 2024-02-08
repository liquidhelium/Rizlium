//! Data structure for a path seprated by dots (".").

use std::{convert::Infallible, fmt::Debug, str::FromStr};

use bevy::prelude::Deref;
use smallvec::SmallVec;

#[derive(Deref, Hash, PartialEq, Eq, Clone)]
pub struct DotPath {
    inner: SmallVec<[String; 6]>
}

impl Debug for DotPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.join(".").fmt(f)
    }
}

impl FromIterator<String> for DotPath {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self { inner: iter.into_iter().map(|s| {assert!(!s.contains(".")); s}).collect()}
    }
}


impl FromStr for DotPath {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.into())
    }
}

impl From<&str> for DotPath {
    fn from(s: &str) -> Self {
        Self{inner: s.split(".").map(|s| s.to_owned()).collect()}
    }
}

impl ToString for DotPath {
    fn to_string(&self) -> String {
        self.inner.join(".")
    }
}

impl DotPath {
    pub fn into_inner(self) -> SmallVec<[String; 6]> {
        self.inner
    }
    pub fn push(&mut self, name: String) {
        assert!(!name.contains("."));
        self.inner.push(name);
    }
    pub fn push_dotted(&mut self, names: &str) {
        self.inner.append(&mut names.split(".").map(|s| s.to_owned()).collect::<SmallVec<[String;6]>>())
    }
    pub fn pop(&mut self) -> Option<String> {
        self.inner.pop()
    }
}