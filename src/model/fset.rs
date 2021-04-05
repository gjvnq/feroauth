use crate::model::prelude::*;
use oso::errors::TypeError;
use oso::{FromPolar, PolarValue, ToPolar};
use serde::de::DeserializeOwned;
use serde::{Deserializer, Serializer};
use std::borrow::Borrow;
use std::collections::hash_map::RandomState;
use std::collections::hash_set::{
    Difference, Drain, Intersection, IntoIter, Iter, SymmetricDifference, Union,
};
use std::collections::HashSet;
use std::hash::BuildHasher;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct FSet<T, S = RandomState>(HashSet<T, S>)
where
    T: Eq + Hash;

impl<T> FSet<T>
where
    T: Eq + Hash,
{
    pub fn new() -> Self {
        FSet(HashSet::new())
    }
}

impl<T, S> FSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn drain(&mut self) -> Drain<'_, T> {
        self.0.drain()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    pub fn with_hasher(hasher: S) -> FSet<T, S> {
        FSet(HashSet::with_hasher(hasher))
    }

    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> FSet<T, S> {
        FSet(HashSet::with_capacity_and_hasher(capacity, hasher))
    }

    pub fn hasher(&self) -> &S {
        self.0.hasher()
    }

    pub fn difference<'a>(&'a self, other: &'a HashSet<T, S>) -> Difference<'a, T, S> {
        self.0.difference(other)
    }

    pub fn symmetric_difference<'a>(
        &'a self,
        other: &'a HashSet<T, S>,
    ) -> SymmetricDifference<'a, T, S> {
        self.0.symmetric_difference(other)
    }

    pub fn intersection<'a>(&'a self, other: &'a HashSet<T, S>) -> Intersection<'a, T, S> {
        self.0.intersection(other)
    }

    pub fn union<'a>(&'a self, other: &'a HashSet<T, S>) -> Union<'a, T, S> {
        self.0.union(other)
    }

    pub fn contains<Q: ?Sized>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.0.contains(value)
    }

    pub fn get<Q: ?Sized>(&self, value: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.0.get(value)
    }

    pub fn is_disjoint(&self, other: &HashSet<T, S>) -> bool {
        self.0.is_disjoint(other)
    }

    pub fn is_subset(&self, other: &HashSet<T, S>) -> bool {
        self.0.is_subset(other)
    }

    pub fn is_superset(&self, other: &HashSet<T, S>) -> bool {
        self.0.is_superset(other)
    }

    pub fn insert(&mut self, value: T) -> bool {
        self.0.insert(value)
    }

    pub fn replace(&mut self, value: T) -> Option<T> {
        self.0.replace(value)
    }

    pub fn remove<Q: ?Sized>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.0.remove(value)
    }

    pub fn take<Q: ?Sized>(&mut self, value: &Q) -> Option<T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.0.take(value)
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.0.retain(f)
    }
}

impl<T, H> Serialize for FSet<T, H>
where
    T: Serialize + Eq + Hash,
    H: BuildHasher,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for FSet<T>
where
    T: DeserializeOwned + Eq + Hash,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(FSet(HashSet::<T>::deserialize(deserializer)?))
    }
}

impl<'a, T, S> IntoIterator for &'a FSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Iter<'a, T> {
        self.0.iter()
    }
}

impl<T, S> IntoIterator for FSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> IntoIter<T> {
        self.0.into_iter()
    }
}

impl<T, S> ToPolar for FSet<T, S>
where
    T: Eq + Hash + ToPolar,
    S: BuildHasher,
{
    fn to_polar(self) -> PolarValue {
        PolarValue::List(self.into_iter().map(|v| v.to_polar()).collect())
    }
}

impl<T> FromPolar for FSet<T>
where
    T: Eq + Hash + Clone + FromPolar,
{
    fn from_polar(val: PolarValue) -> Result<Self, OsoErrorReal> {
        if let PolarValue::List(l) = val {
            let mut result = HashSet::new();
            for v in l {
                result.insert(T::from_polar(v)?);
            }
            Ok(FSet(result))
        } else {
            Err(TypeError::expected("List").user())
        }
    }
}

impl<T,S> std::convert::From<HashSet<T,S>> for FSet<T,S> where T: Eq + Hash, S: BuildHasher {
    fn from(val: HashSet<T,S>) -> Self {
        FSet(val)
    }
}

impl<T,S> std::convert::Into<HashSet<T,S>> for FSet<T,S> where T: Eq + Hash, S: BuildHasher {
    fn into(self) -> HashSet<T,S> {
        self.0
    }
}