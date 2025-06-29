use std::hash::{DefaultHasher, Hash, Hasher};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Index, IndexMut};
use std::error;
use std::fmt;

use crate::check::{Check, IsTrue, is_prime_and_within_limit};

#[derive(Debug, Clone, PartialEq)]
pub struct OutOfCapacityError {pub capacity: usize}

impl fmt::Display for OutOfCapacityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HashMap has reached its capacity of {} entries", self.capacity)
    }
}

impl error::Error for OutOfCapacityError {}

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

trait Entry<K, V, const C: usize> {
    type Type;
    fn key(&self) -> &K;
    fn value(&self) -> &V;
    fn consume_self(self) -> V;
    fn mut_value(&mut self) -> &mut V;
    fn next(&self) -> usize;
    fn mut_next(&mut self) -> &mut usize;
    fn prev(&self) -> usize;
    fn mut_prev(&mut self) -> &mut usize;
    fn new(key: K, value: V) -> Self; 
}

pub struct FixedSizeHashMapImpl<K, V, const C: usize, H, E>
where
    Check<{ is_prime_and_within_limit(C, 25013) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
    E: Entry<K, V, C>
{
    _data: Vec<Slot<E>>,
    _size: usize,
    _head: usize,
    _tail: usize,
    _phantom: PhantomData<(K, V, H)>
}


impl<K, V, const C: usize, H, E> Default for FixedSizeHashMapImpl<K, V, C, H, E>
where
    Check<{ is_prime_and_within_limit(C, 25013) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
    E: Entry<K, V, C>
{
    fn default() -> Self {
        Self::new()
    }
}


// hash_map Internals 
impl<K, V, const C: usize, H, E> FixedSizeHashMapImpl<K, V, C, H, E>
where
    Check<{ is_prime_and_within_limit(C, 25013) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
    E: Entry<K, V, C>
{
    pub(crate) fn _get_index_of(&mut self, key: &K) -> Option<usize> {
        let i = self._find_index(key, true);
        if i != Self::CAPACITY && self._data[i].is_occupied() {Some(i)} else {None}
    }

    pub(crate) fn _get_mut_value_and_index_of(&mut self, key: &K) -> Option<(&mut V, usize)> {
        let i = self._find_index(key, true);
        if i != Self::CAPACITY && let Slot::IsOccupiedBy(ref mut entry) = self._data[i] {
            Some ((entry.mut_value(), i))
        } else {
            None
        }
    }

    pub(crate) fn _get_val_and_index_of(&self, key: &K) -> Option<(&V, usize)> {
        let i = self._find_index(key, true);
        if i != Self::CAPACITY && let Slot::IsOccupiedBy(ref entry) = self._data[i] {
            Some ((entry.value(), i))
        } else {
            None
        }
    }

    pub(crate) fn _get_mut_value_at(&mut self, i: usize) -> Option<&mut V> {
        if i < Self::CAPACITY
            && let Slot::IsOccupiedBy(ref mut entry) = self._data[i]
        {
            Some(entry.mut_value())
        } else {
            None
        }
    }

    pub(crate) fn _insert_get_index(&mut self, key: K, value: V) -> Result<(usize, Option<V>), OutOfCapacityError> {
        let i = self._find_index(&key, false);
        if i == Self::CAPACITY {
            return Result::Err(OutOfCapacityError{capacity: Self::CAPACITY});
        }

        let mut old_val: Option<V> = None;
        match self._data[i] {
            Slot::IsOccupiedBy(ref mut entry) => {
                old_val = Some(mem::replace(entry.mut_value(), value));
                self._remove_from_list(i);
                self._move_to_back_of_list(i);
            }
            Slot::Empty | Slot::WasOccupied => {
            self._data[i] = Slot::IsOccupiedBy({
                    let mut e = E::new(key, value);
                    *e.mut_next() = Self::CAPACITY;
                    *e.mut_next() = Self::CAPACITY;
                    e
                });
                self._move_to_back_of_list(i);
                self._size += 1;
            }
        }

        Result::Ok((i, old_val))
    }

    fn _get_key_val_at(&self, i: usize) -> Option<(&K, &V)> {
        if i < Self::CAPACITY
            && let Slot::IsOccupiedBy(ref entry) = self._data[i]
        {
            Some((entry.key(), entry.value()))
        } else {
            None
        }
    }

    fn _find_index(&self, key: &K, key_must_exist: bool) -> usize {
        let mut hash_state: Self::_Hash = Default::default();
        key.hash(&mut hash_state);
        let already_visited = (hash_state.finish() % Self::CAPACITY as u64) as usize;
        let mut index = already_visited;

        while match self._data[index] {
            Slot::IsOccupiedBy(ref entry) => *entry.key() != *key,
            Slot::WasOccupied => key_must_exist,
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
                *entry.mut_prev() = i;
            }

            if let Slot::IsOccupiedBy(ref mut entry) = self._data[i] {
                *entry.mut_next() = self._head;
                *entry.mut_prev() = Self::CAPACITY;
            }

            self._head = i;
        }
    }

    fn _move_to_back_of_list(&mut self, i: usize) {
        if self._size == 0 {
            debug_assert!(self._head == Self::CAPACITY && self._tail == Self::CAPACITY);
            self._head = i;
            self._tail = i;
        } else {
            if let Slot::IsOccupiedBy(ref mut entry) = self._data[self._tail] {
                *entry.mut_next() = i;
            }

            if let Slot::IsOccupiedBy(ref mut entry) = self._data[i] {
                *entry.mut_prev() = self._tail;
                *entry.mut_next() = Self::CAPACITY;
            }

            self._tail = i;
        }
    }

    fn _remove_from_list(&mut self, i: usize) {
        let mut entry_next = Self::CAPACITY;
        let mut entry_prev = Self::CAPACITY;

        match self._data[i] {
            Slot::IsOccupiedBy(ref entry) => {
                entry_next = entry.next();
                entry_prev = entry.prev();
            }
            _ => {}
        }

        if entry_prev != Self::CAPACITY {
            match self._data[entry_prev] {
                Slot::IsOccupiedBy(ref mut entry) => *entry.mut_next() = entry_next,
                _ => {}
            }
        } else {
            self._head = entry_next;
        }

        if entry_next != Self::CAPACITY {
            match self._data[entry_next] {
                Slot::IsOccupiedBy(ref mut entry) => *entry.mut_prev() = entry_prev,
                _ => {}
            }
        } else {
            self._tail = entry_prev;
        }
    }

    fn _get_entry_at(&self, i: usize) -> Option<&E> {
        if i < Self::CAPACITY
            && let Slot::IsOccupiedBy(ref entry) = self._data[i]
        {
            Some(entry)
        } else {
            None
        }
    }
}

impl<K, V, const C: usize, H, E> FixedSizeHashMapImpl<K, V, C, H, E>
where
    Check<{ is_prime_and_within_limit(C, 25013) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
    E: Entry<K, V, C>
{
    const CAPACITY: usize = C;
    type _Hash = H;

    pub fn new() -> FixedSizeHashMapImpl<K, V, C, H, E> {
        let mut map = FixedSizeHashMapImpl::<K, V, C, H, E> {
            _data: Vec::new(),
            _size: 0,
            _head: Self::CAPACITY,
            _tail: Self::CAPACITY,
            _phantom: Default::default(),
        };
        map._data.resize_with(Self::CAPACITY, Default::default);
        map
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, OutOfCapacityError> {
        let result = self._insert_get_index(key, value);
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
        let i = self._find_index(key, true);
        i != Self::CAPACITY && self._data[i].is_occupied() 
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let i = self._find_index(key, true);

        if i != Self::CAPACITY && let Slot::IsOccupiedBy(ref entry) = self._data[i]
        {
            Some(entry.value())
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let i = self._find_index(key, true);

        if i < Self::CAPACITY && let Slot::IsOccupiedBy(ref mut entry) = self._data[i]
        {
            Some(entry.mut_value())
        } else {
            None
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let i = self._find_index(key, true);
        if i >= Self::CAPACITY || !self._data[i].is_occupied() {
            return None;
        }

        self._size -= 1;
        self._remove_from_list(i);

        self._data[i].take().map(|entry| entry.consume_self())
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

    pub fn iter_head(&self) -> MapIterator<'_, K, V, E, C, impl Fn(&E) -> usize + use<K, V, C, H, E>> {
        MapIterator {
            _remaining: self._size,
            _current: self._head,
            _data: &self._data,
            _fn_next: |entry| entry.next(),
            _phantom: Default::default(),
        }
    }

    pub fn iter_tail(&self) -> MapIterator<'_, K, V, E, C, impl Fn(&E) -> usize + use<K, V, C, H, E>> {
        MapIterator {
            _remaining: self._size,
            _current: self._tail,
            _data: &self._data,
            _fn_next: |entry| entry.prev(),
            _phantom: Default::default(),
        }
    }
}

impl<'a, K: 'a, V: 'a, const C: usize, H, E> Index<&K> for FixedSizeHashMapImpl<K, V, C, H, E>
where
    Check<{ is_prime_and_within_limit(C, 25013) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
    E: Entry<K, V, C>
{
    type Output = V;
    fn index(&self, key: &K) -> &Self::Output {
        self.get(key).expect("Panic! not in map")
    }
}

impl<K, V, const C: usize, H, E> IndexMut<&K> for FixedSizeHashMapImpl<K, V, C, H, E>
where
    Check<{ is_prime_and_within_limit(C, 25013) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
    E: Entry<K, V, C>
{
    fn index_mut(&mut self, key: &K) -> &mut Self::Output {
        self.get_mut(key).expect("Panic! not in map")
    }
}

pub struct MapIterator<'a, K: 'a, V: 'a, E: 'a, const C: usize, Next>
where
    Next: Fn(&E) -> usize,
    E: Entry<K, V, C>
{
    _remaining: usize,
    _current: usize,
    _data: &'a Vec<Slot<E>>,
    _fn_next: Next,
    _phantom: PhantomData<(K, V)>
}

impl<'a, K: 'a, V: 'a, E: 'a, const C: usize, Next> Iterator for MapIterator<'a, K, V, E, C, Next>
where
    E: Entry<K, V, C>,
    Next: Fn(&E) -> usize,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self._current < C
            && let Slot::IsOccupiedBy(ref entry) = self._data[self._current]
        {
            self._remaining -= 1;
            self._current = (self._fn_next)(entry);
            Some((entry.key(), entry.value()))
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


pub struct MapEntry<K, V, const C: usize> {
    key: K,
    value: V,
    next: usize,
    prev: usize,
}

impl<K, V, const C: usize> Entry<K, V, C> for MapEntry<K, V, C> {
    type Type = MapEntry<K, V, C>;
    fn key(&self) -> &K {&self.key}
    fn value(&self) -> &V {&self.value}
    fn consume_self(self) -> V {self.value}
    fn mut_value(&mut self) -> &mut V {&mut self.value}
    fn next(&self) -> usize {self.next}
    fn mut_next(&mut self) -> &mut usize {&mut self.next}
    fn prev(&self) -> usize {self.prev}
    fn mut_prev(&mut self) -> &mut usize {&mut self.prev}
    fn new(key: K, value: V) -> Self {
        Self {
            key: key,
            value: value,
            next: C,
            prev: C,
        }
    }
}

pub type FixedSizeHashMap<K, V, const C: usize, H=DefaultHasher> = FixedSizeHashMapImpl<K, V, C, H, MapEntry<K, V, C>>;

pub type FixedSizeHashSet<K, const C: usize> = FixedSizeHashMapImpl<K, (), C, DefaultHasher, MapEntry<K, (), C>>;
