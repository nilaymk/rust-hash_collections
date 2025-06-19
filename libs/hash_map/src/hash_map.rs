use const_primes::is_prime;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Index, IndexMut};

use crate::check::{Check, IsTrue};

pub struct Entry<K, V, const C: usize> {
    key: K,
    value: V,
    next: usize,
    prev: usize,
}

pub enum Slot<T> {
    Empty,
    WasOccupied,
    IsOccupiedBy(T),
}

impl<T> Default for Slot<T> {
    fn default() -> Self {
        Self::Empty
    }
}

impl<T> Slot<T> {
    fn is_occupied(&self) -> bool {
        match self {
            Self::IsOccupiedBy(_) => true,
            _ => false,
        }
    }

    fn take(&mut self) -> Option<T> {
        let old_entry;
        (*self, old_entry) = match std::mem::replace(self, Slot::Empty) {
            Slot::IsOccupiedBy(old_entry) => (Slot::WasOccupied, Some(old_entry)),
            Slot::WasOccupied => (Slot::WasOccupied, None),
            Slot::Empty => (Slot::Empty, None),
        };
        old_entry
    }
}

pub struct FixedSizeHashMapImpl<K, V, const C: usize, H>
where
    Check<{ is_prime_and_within_limit(C, 25013) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    _data: Vec<Slot<Entry<K, V, C>>>,
    _size: usize,
    _head: usize,
    _tail: usize,
    _phantom: PhantomData<H>,
}

pub const fn is_prime_and_within_limit(c: usize, max_cap: usize) -> bool {
    is_prime(c as u64) && c <= max_cap
}

impl<K, V, const C: usize, H> Default for FixedSizeHashMapImpl<K, V, C, H>
where
    Check<{ is_prime_and_within_limit(C, 25013) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V, const C: usize, H> FixedSizeHashMapImpl<K, V, C, H>
where
    Check<{ is_prime_and_within_limit(C, 25013) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    const CAPACITY: usize = C;
    type _Hash = H;

    pub fn new() -> FixedSizeHashMapImpl<K, V, C, H> {
        let mut map = FixedSizeHashMapImpl::<K, V, C, H> {
            _data: Vec::new(),
            _size: 0,
            _head: Self::CAPACITY,
            _tail: Self::CAPACITY,
            _phantom: Default::default(),
        };
        map._data.resize_with(Self::CAPACITY, Default::default);
        map
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let i = self._find_index(&key, false);
        if i == Self::CAPACITY {
            return None;
        }

        let mut old_val: Option<V> = None;
        match self._data[i] {
            Slot::IsOccupiedBy(ref mut entry) => {
                old_val = Some(mem::replace(&mut entry.value, value));
                self._remove_from_list(i);
                self._move_to_front_of_list(i);
            }
            Slot::Empty | Slot::WasOccupied => {
                self._data[i] = Slot::IsOccupiedBy(Entry {
                    key,
                    value,
                    next: Self::CAPACITY,
                    prev: Self::CAPACITY,
                });
                self._move_to_front_of_list(i);
                self._size += 1;
            }
        }
        old_val
    }

    pub fn exists(&self, key: &K) -> bool {
        let i = self._find_index(key, true);
        i != Self::CAPACITY && self._data[i].is_occupied()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let i = self._find_index(key, true);
        match self._get_key_val_at(i) {
            Some(key_val_pair) => Some(key_val_pair.1),
            None => None,
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let i = self._find_index(key, true);

        if i < Self::CAPACITY
            && let Slot::IsOccupiedBy(ref mut entry) = self._data[i]
        {
            Some(&mut entry.value)
        } else {
            None
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let i = self._find_index(key, true);
        if i == Self::CAPACITY {
            return None;
        }

        match self._data[i] {
            Slot::IsOccupiedBy(_) => {
                self._size -= 1;
                self._remove_from_list(i);
            }
            _ => {}
        }

        match self._data[i].take() {
            Some(entry) => Some(entry.value),
            None => None,
        }
    }

    pub const fn capacity(&self) -> usize {
        Self::CAPACITY
    }

    pub fn size(&self) -> usize {
        self._size
    }

    pub fn head(&self) -> Option<(&K, &V)> {
        self._get_key_val_at(self._head)
    }

    pub fn tail(&self) -> Option<(&K, &V)> {
        self._get_key_val_at(self._tail)
    }

    pub fn iter_head(&self) -> MapIterator<'_, K, V, C, impl Fn(&Entry<K, V, C>) -> usize> {
        MapIterator {
            _remaining: self._size,
            _current: self._head,
            _data: &self._data,
            _fn_next: |entry| entry.next,
        }
    }

    pub fn iter_tail(&self) -> MapIterator<'_, K, V, C, impl Fn(&Entry<K, V, C>) -> usize> {
        MapIterator {
            _remaining: self._size,
            _current: self._tail,
            _data: &self._data,
            _fn_next: |entry| entry.prev,
        }
    }

    fn _get_key_val_at(&self, i: usize) -> Option<(&K, &V)> {
        if i < Self::CAPACITY
            && let Slot::IsOccupiedBy(ref entry) = self._data[i]
        {
            Some((&entry.key, &entry.value))
        } else {
            None
        }
    }

    fn _find_index(&self, key: &K, skip_previously_occupied: bool) -> usize {
        let mut hash_state: Self::_Hash = Default::default();
        key.hash(&mut hash_state);
        let already_visited = (hash_state.finish() % Self::CAPACITY as u64) as usize;
        let mut index = already_visited;
        while match self._data[index] {
            Slot::IsOccupiedBy(ref entry) => entry.key != *key,
            Slot::WasOccupied => skip_previously_occupied,
            Slot::Empty => false,
        } {
            index = (index + 1) % Self::CAPACITY; // linear probing
            if index == already_visited {
                return Self::CAPACITY;
            }
        }
        index
    }

    fn _move_to_front_of_list(&mut self, i: usize) {
        if self._size == 0 {
            debug_assert!(self._head == Self::CAPACITY && self._tail == Self::CAPACITY);
            self._head = i;
            self._tail = i;
        } else {
            if let Slot::IsOccupiedBy(ref mut entry) = self._data[self._head] {
                entry.prev = i;
            }

            if let Slot::IsOccupiedBy(ref mut entry) = self._data[i] {
                entry.next = self._head;
                entry.prev = Self::CAPACITY;
            }

            self._head = i;
        }
    }

    fn _remove_from_list(&mut self, i: usize) {
        let mut entry_next = Self::CAPACITY;
        let mut entry_prev = Self::CAPACITY;

        match self._data[i] {
            Slot::IsOccupiedBy(ref entry) => {
                entry_next = entry.next;
                entry_prev = entry.prev;
            }
            _ => {}
        }

        if entry_prev != Self::CAPACITY {
            match self._data[entry_prev] {
                Slot::IsOccupiedBy(ref mut entry) => entry.next = entry_next,
                _ => {}
            }
        } else {
            self._head = entry_next;
        }

        if entry_next != Self::CAPACITY {
            match self._data[entry_next] {
                Slot::IsOccupiedBy(ref mut entry) => entry.prev = entry_prev,
                _ => {}
            }
        } else {
            self._tail = entry_prev;
        }
    }

    fn _get_entry_at(&self, i: usize) -> Option<&Entry<K, V, C>> {
        if i < Self::CAPACITY
            && let Slot::IsOccupiedBy(ref entry) = self._data[i]
        {
            Some(entry)
        } else {
            None
        }
    }
}

impl<'a, K: 'a, V: 'a, const C: usize, H> Index<&K> for FixedSizeHashMapImpl<K, V, C, H>
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

impl<K, V, const C: usize, H> IndexMut<&K> for FixedSizeHashMapImpl<K, V, C, H>
where
    Check<{ is_prime_and_within_limit(C, 25013) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    fn index_mut(&mut self, key: &K) -> &mut Self::Output {
        self.get_mut(key).expect("Panic! not in map")
    }
}

pub struct MapIterator<'a, K: 'a, V: 'a, const C: usize, Next>
where
    Next: Fn(&Entry<K, V, C>) -> usize,
{
    _remaining: usize,
    _current: usize,
    _data: &'a Vec<Slot<Entry<K, V, C>>>,
    _fn_next: Next,
}

impl<'a, K: 'a, V: 'a, const C: usize, Next> Iterator for MapIterator<'a, K, V, C, Next>
where
    Next: Fn(&Entry<K, V, C>) -> usize,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self._current < C
            && let Slot::IsOccupiedBy(ref entry) = self._data[self._current]
        {
            self._remaining -= 1;
            self._current = (self._fn_next)(entry);
            Some((&entry.key, &entry.value))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self._remaining, Some(self._remaining))
    }

    fn count(self) -> usize {
        self._remaining
    }
}

pub type FixedSizeHashMap<K, V, const C: usize> = FixedSizeHashMapImpl<K, V, C, DefaultHasher>;
