use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::{Index, IndexMut};


use crate::hash_map_internal::{FixedSizeHashMapImpl, Entry};
use crate::check::{Check, IsTrue, is_prime_and_within_limit};

pub use crate::hash_map_internal::{MapIteratorImpl as MapIterator, OutOfCapacityError};

pub struct MapEntry<K, V, const C: usize> {
    _key: K,
    _value: V,
    _next: usize,
    _prev: usize,
}

impl<K, V, const C: usize> Entry<K, V, C> for MapEntry<K, V, C> {
    type Type = MapEntry<K, V, C>;
    fn key(&self) -> &K {&self._key}
    fn value(&self) -> &V {&self._value}
    fn consume_self(self) -> V {self._value}
    fn mut_value(&mut self) -> &mut V {&mut self._value}
    fn next(&self) -> usize {self._next}
    fn mut_next(&mut self) -> &mut usize {&mut self._next}
    fn prev(&self) -> usize {self._prev}
    fn mut_prev(&mut self) -> &mut usize {&mut self._prev}
    fn new(key: K, value: V) -> Self {
        Self {
            _key: key,
            _value: value,
            _next: C,
            _prev: C,
        }
    }
}

pub struct FixedSizeHashMap<K, V, const C: usize, H=DefaultHasher>
where
    Check<{ is_prime_and_within_limit(C, 25013) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    _hash_map_internal: FixedSizeHashMapImpl<K, V, C, H, MapEntry<K, V, C>>
}

impl<K, V, const C: usize, H> FixedSizeHashMap<K, V, C, H>
where
    Check<{ is_prime_and_within_limit(C, 25013) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    const CAPACITY: usize = C;
    type _Hash = H;

    pub fn new() -> FixedSizeHashMap<K, V, C, H> {
        FixedSizeHashMap::<K, V, C, H> {
            _hash_map_internal: FixedSizeHashMapImpl::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, OutOfCapacityError> {
        let result = self._hash_map_internal._insert_get_index(key, value);
        result.map(|rv| rv.1)
    }

    pub fn insert_or<F: FnOnce(&mut V)>(&mut self, key: K, value: V, op: F) -> Result<(), OutOfCapacityError> {
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
        self._hash_map_internal._get_val_and_index_of(key).map(|kv| kv.0)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
         self._hash_map_internal._get_mut_value_and_index_of(key).map(|kv| kv.0)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self._hash_map_internal.remove(key)
    }

    pub const fn capacity(&self) -> usize {
        Self::CAPACITY
    }

    pub fn size(&self) -> usize {
        self._hash_map_internal.size()
    }

    pub fn head(&self) -> Option<(&K, &V)> {
        self._hash_map_internal.head()
    }

    pub fn tail(&self) -> Option<(&K, &V)> {
        self._hash_map_internal.tail()
    }

    pub fn iter_head(&self) -> MapIterator<'_, K, V, MapEntry<K, V, C>, C, impl Fn(&MapEntry<K, V, C>) -> usize + use<K, V, C, H>> {
        self._hash_map_internal.iter_head()
    }

    pub fn iter_tail(&self) -> MapIterator<'_, K, V, MapEntry<K, V, C>, C, impl Fn(&MapEntry<K, V, C>) -> usize + use<K, V, C, H>> {
        self._hash_map_internal.iter_tail()
    }
}

impl<'a, K: 'a, V: 'a, const C: usize, H> Index<&K> for FixedSizeHashMap<K, V, C, H>
where
    Check<{ is_prime_and_within_limit(C, 25013) }>: IsTrue,
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
    Check<{ is_prime_and_within_limit(C, 25013) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    fn index_mut(&mut self, key: &K) -> &mut Self::Output {
        self.get_mut(key).expect("Panic! not in map")
    }
}

pub type FixedSizeHashSet<K, const C: usize> = FixedSizeHashMap<K, (), C>;
