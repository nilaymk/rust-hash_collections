#![allow(dead_code)]

use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::{Index, IndexMut};

use crate::{
    check::{Check, IsTrue, is_prime_and_within_limit},
    hash_map_internal::{Entry, FixedSizeHashMapImpl, MapIteratorImpl},
    OutOfCapacityError
};

pub struct MapEntry<K, V, const C: usize> {
    _key: K,
    _value: V,
    _next: usize,
    _prev: usize,
}

impl<K, V, const C: usize> Entry<K, V, C> for MapEntry<K, V, C> {
    fn key(&self) -> &K {
        &self._key
    }
    fn value(&self) -> &V {
        &self._value
    }
    fn consume_self(self) -> V {
        self._value
    }
    fn mut_value(&mut self) -> &mut V {
        &mut self._value
    }
    fn next(&self) -> usize {
        self._next
    }
    fn mut_next(&mut self) -> &mut usize {
        &mut self._next
    }
    fn prev(&self) -> usize {
        self._prev
    }
    fn mut_prev(&mut self) -> &mut usize {
        &mut self._prev
    }
    fn new(key: K, value: V) -> Self {
        Self {
            _key: key,
            _value: value,
            _next: C,
            _prev: C,
        }
    }
}

pub struct FixedSizeHashMap<K, V, const C: usize, H = DefaultHasher>
where
    Check<{ is_prime_and_within_limit(C, crate::MAX_CAPACITY) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    _hash_map_internal: FixedSizeHashMapImpl<K, V, C, H, MapEntry<K, V, C>>,
}

impl<K, V, const C: usize, H> Default for FixedSizeHashMap<K, V, C, H>
where
    Check<{ is_prime_and_within_limit(C, crate::MAX_CAPACITY) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V, const C: usize, H> FixedSizeHashMap<K, V, C, H>
where
    Check<{ is_prime_and_within_limit(C, crate::MAX_CAPACITY) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    const CAPACITY: usize = C;
    type _Hash = H;
    
    pub fn placeholder() -> FixedSizeHashMap<K, V, C, H> {
        FixedSizeHashMap::<K, V, C, H> {
            _hash_map_internal: FixedSizeHashMapImpl::placeholder(),
        }
    }

    pub fn new() -> FixedSizeHashMap<K, V, C, H> {
        FixedSizeHashMap::<K, V, C, H> {
            _hash_map_internal: FixedSizeHashMapImpl::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, OutOfCapacityError> {
        let result = self._hash_map_internal.insert_get_index(key, value);
        result.map(|rv| rv.1)
    }

    pub fn insert_or<F: FnOnce(&mut V)>(
        &mut self,
        key: K,
        value: V,
        op: F,
    ) -> Result<(), OutOfCapacityError> {
        if let Some(value) = self.get_mut(&key) {
            op(value);
            Ok(())
        } else {
            self.insert(key, value).map(|_| ())
        }
    }

    pub fn exists(&self, key: &K) -> bool {
        self._hash_map_internal.exists(key)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self._hash_map_internal
            .get_entry_and_index_of(key)
            .map(|e| e.0.value())
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self._hash_map_internal
            .get_mut_entry_and_index_of(key)
            .map(|e| e.0.mut_value())
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self._hash_map_internal
            .remove(key)
            .map(|e| e.consume_self())
    }

    pub const fn capacity(&self) -> usize {
        Self::CAPACITY
    }

    pub fn size(&self) -> usize {
        self._hash_map_internal.size()
    }

    pub fn head(&self) -> Option<(&K, &V)> {
        self._hash_map_internal.head().map(|e| (e.key(), e.value()))
    }

    pub fn tail(&self) -> Option<(&K, &V)> {
        self._hash_map_internal.tail().map(|e| (e.key(), e.value()))
    }

    pub fn iter_head(&self) -> MapIter<'_, K, V, C> {
        MapIter {
            _inner_iter: self._hash_map_internal.iter_head(),
        }
    }

    pub fn iter_tail(&self) -> MapIter<'_, K, V, C> {
        MapIter {
            _inner_iter: self._hash_map_internal.iter_tail(),
        }
    }
}

impl<'a, K: 'a, V: 'a, const C: usize, H> Index<&K> for FixedSizeHashMap<K, V, C, H>
where
    Check<{ is_prime_and_within_limit(C, crate::MAX_CAPACITY) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    type Output = V;
    fn index(&self, key: &K) -> &Self::Output {
        self.get(key).expect("Panic! not in map")
    }
}

impl<K, V, const C: usize, H> IndexMut<&K> for FixedSizeHashMap<K, V, C, H>
where
    Check<{ is_prime_and_within_limit(C, crate::MAX_CAPACITY) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    fn index_mut(&mut self, key: &K) -> &mut Self::Output {
        self.get_mut(key).expect("Panic! not in map")
    }
}

pub struct MapIter<'a, K: 'a, V: 'a, const C: usize> {
    _inner_iter: MapIteratorImpl<'a, K, V, MapEntry<K, V, C>, C>,
}

impl<'a, K: 'a, V: 'a, const C: usize> Iterator for MapIter<'a, K, V, C> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self._inner_iter.next().map(|e| (e.key(), e.value()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self._inner_iter.size_hint()
    }

    fn count(self) -> usize {
        self._inner_iter.count()
    }
}
