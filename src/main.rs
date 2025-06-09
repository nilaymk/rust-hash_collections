#![feature(adt_const_params, )] 
#![feature(const_type_id)]
#![feature(generic_const_exprs)]
#![feature(core_intrinsics)]
#![feature(inherent_associated_types)]


use const_primes::{is_prime};
use const_guards::guard;
use std::any::{Any};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::mem;

struct Entry <K, V> {
    key: K,
    value: V,
    next: i64,
    prev: i64
}

impl <K, V> Entry <K, V> {
    fn new(key: K, value: V) -> Entry<K, V> {
        Entry {
            key: key,
            value: value,
            next: -1,
            prev: -1,
        }
    }
}

struct FixedSizeHashMap <K, V, const C: usize>  {
    _data: Vec<Option<Entry<K, V>>>,
    _size: usize,
}

const fn is_integral_or_string<T: ?Sized + Any>() -> bool {
    core::intrinsics::type_id::<T>() == core::intrinsics::type_id::<String>()
    || core::intrinsics::type_id::<T>() == core::intrinsics::type_id::<i32>()
    || core::intrinsics::type_id::<T>() == core::intrinsics::type_id::<i64>()
    || core::intrinsics::type_id::<T>() == core::intrinsics::type_id::<i128>()
    || core::intrinsics::type_id::<T>() == core::intrinsics::type_id::<u32>()
    || core::intrinsics::type_id::<T>() == core::intrinsics::type_id::<u64>()
    || core::intrinsics::type_id::<T>() == core::intrinsics::type_id::<u128>()
}


#[guard(is_prime(C as u64))]
#[guard(C <= 25013)]
impl <K: Hash, V, const C: usize> FixedSizeHashMap <K, V, C>  {
    const CAPACITY: usize = C;

    fn calculate_index(key: &K) -> u64 {
        let mut s = DefaultHasher::new();
        key.hash(&mut s);
        s.finish() % Self::CAPACITY as u64
    }

    pub fn new() -> FixedSizeHashMap<K, V, C> {
        FixedSizeHashMap {
            _data: Vec::with_capacity(Self::CAPACITY as usize),
            _size: 0
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let i = Self::calculate_index(&key) as usize;
        match self._data[i].as_mut() {
            Some(entry) => {
                let old_val = mem::replace(&mut entry.value, value);
                Some(old_val)
            }
            None => {
                self._data[i] = Some(Entry::new(key, value));
                self._size += 1;
                None
            }
        }
    }

    pub fn exists(&self, key: &K) -> bool {
        let i = Self::calculate_index(key) as usize;
        self._data[i].is_some()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let i = Self::calculate_index(&key) as usize;
        match self._data[i].as_ref() {
            Some(data) => Some(&data.value),
            None => None
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let i = Self::calculate_index(&key) as usize;
        let old_entry = mem::replace(&mut self._data[i], None);
        match old_entry {
            None => None,
            Some(key_val) => {
                self._size -= 1;
                Some(key_val.value)
            } 
        }
    } 

    pub const fn capacity(&self) -> usize {
        Self::CAPACITY
    }
    
    pub fn size(&self) -> usize {
        self._size
    }
}

type MyMap = FixedSizeHashMap<String, String, 97>;

fn main() {
    let fmap: MyMap = MyMap::new();
    println!("Hello, world!");
}
