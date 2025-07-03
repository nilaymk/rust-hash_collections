#![allow(dead_code)]

use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{
    check::{Check, IsTrue, is_prime_and_within_limit},
    hash_map_internal::{Entry, FixedSizeHashMapImpl, MapIteratorImpl},
    hash_map::MapEntry,
    OutOfCapacityError
};

pub struct FixedSizeHashSet<T, const C: usize, H = DefaultHasher>
where
    Check<{ is_prime_and_within_limit(C, crate::MAX_CAPACITY) }>: IsTrue,
    T: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    _hash_map_internal: FixedSizeHashMapImpl<T, (), C, H, MapEntry<T, (), C>>,
}

impl<T, const C: usize, H> Default for FixedSizeHashSet<T, C, H>
where
    Check<{ is_prime_and_within_limit(C, crate::MAX_CAPACITY) }>: IsTrue,
    T: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const C: usize, H> FixedSizeHashSet<T, C, H>
where
    Check<{ is_prime_and_within_limit(C, crate::MAX_CAPACITY) }>: IsTrue,
    T: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    const CAPACITY: usize = C;
    type _Hash = H;
    
    pub fn placeholder() -> FixedSizeHashSet<T, C, H> {
        FixedSizeHashSet::<T, C, H> {
            _hash_map_internal: FixedSizeHashMapImpl::placeholder(),
        }
    }

    pub fn new() -> FixedSizeHashSet<T, C, H> {
        FixedSizeHashSet::<T, C, H> {
            _hash_map_internal: FixedSizeHashMapImpl::new(),
        }
    }

    pub fn insert(&mut self, item: T) -> Result<bool, OutOfCapacityError> {
        match self._hash_map_internal.exists(&item) {
            true => Ok(false),
            false => {
                let result = self._hash_map_internal.insert_get_index(item, ());
                result.map(|_| true)
            }
        }
    }

    pub fn exists(&self, item: &T) -> bool {
        self._hash_map_internal.exists(item)
    }

    pub fn remove(&mut self, item: &T) -> bool {
        match self._hash_map_internal.exists(item) {
            false => false,
            true => {
                self._hash_map_internal.remove(item);
                true
            }
        }
    }

    pub const fn capacity(&self) -> usize {
        Self::CAPACITY
    }

    pub fn size(&self) -> usize {
        self._hash_map_internal.size()
    }

    pub fn head(&self) -> Option<&T> {
        self._hash_map_internal.head().map(|e| e.key())
    }

    pub fn tail(&self) -> Option<&T> {
        self._hash_map_internal.tail().map(|e| e.key())
    }

    pub fn iter_head(&self) -> SetIter<'_, T, C> {
        SetIter {
            _inner_iter: self._hash_map_internal.iter_head(),
        }
    }

    pub fn iter_tail(&self) -> SetIter<'_, T, C> {
        SetIter {
            _inner_iter: self._hash_map_internal.iter_tail(),
        }
    }
}

pub struct SetIter<'a, T: 'a, const C: usize> {
    _inner_iter: MapIteratorImpl<'a, T, (), MapEntry<T, (), C>, C>,
}

impl<'a, T: 'a, const C: usize> Iterator for SetIter<'a, T, C> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self._inner_iter.next().map(|e| e.key())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self._inner_iter.size_hint()
    }

    fn count(self) -> usize {
        self._inner_iter.count()
    }
}
