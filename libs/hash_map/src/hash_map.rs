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

pub struct FixedSizeHashMap <K, V, const C: usize>  {
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
impl <K: Hash + std::cmp::Eq, V, const C: usize> FixedSizeHashMap <K, V, C>  {
    const CAPACITY: usize = C;

    fn find_index(&self, key: &K) -> usize {
        let mut s = DefaultHasher::new();
        key.hash(&mut s);
        let already_visited = (s.finish() % Self::CAPACITY as u64) as usize;
        let mut index = already_visited;
        while self._data[index].as_ref().is_some_and(|key_val| {key_val.key != *key}) {
            index = (index + 1) % Self::CAPACITY; // linear probing
            if (index == already_visited) {
                return Self::CAPACITY
            }
        }
        index
    }
    
    pub fn new() -> FixedSizeHashMap<K, V, C> {
        let mut map = FixedSizeHashMap {
            _data: Vec::new(),
            _size: 0,
        };
        map._data.resize_with(Self::CAPACITY, Default::default);
        map
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let i = self.find_index(&key);
        if i == Self::CAPACITY {
            return None
        }

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
        let i = self.find_index(key);
        i != Self::CAPACITY && self._data[i].is_some()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let i = self.find_index(&key);
        if i == Self::CAPACITY {
            return None
        }
        match self._data[i].as_ref() {
            Some(data) => Some(&data.value),
            None => None
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let i = self.find_index(&key);
        if i == Self::CAPACITY {
            return None
        }
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


#[cfg(test)]
mod tests {
    type MyMap = FixedSizeHashMap<String, u64, 13>;
    fn add_some_data(mymap: &mut MyMap, num: i32) {
        let keys = ["foo", "bar", "baz", "bat", "boo", "fat"];
        for (i, key) in keys.iter().enumerate() {
            if i as i32 == num {
                break;
            }
            mymap.insert(String::from(*key), (i as u64 +1)*100);
        }
    }

    use super::*;
    #[test]
    fn insert_and_get_items() {
        let mut fmap = MyMap::new();
        assert!(fmap.capacity() == 13);
        assert!(fmap.size() == 0);
        assert!(fmap.exists(&String::from("foo")) == false);
        assert!(fmap.get(&String::from("foo")) == None);

        add_some_data(&mut fmap, 3);

        assert!(fmap.capacity() == 13);
        assert_eq!(fmap.size(), 3);
        assert!(fmap.exists(&String::from("foo"))
            && fmap.exists(&String::from("bar"))
            && fmap.exists(&String::from("baz"))
        );
        assert_eq!(fmap.get(&String::from("foo")), Some(&100));
        assert_eq!(fmap.get(&String::from("bar")), Some(&200));
        assert_eq!(fmap.get(&String::from("baz")), Some(&300));
    }
    #[test]
    fn update_items() {
        let mut fmap = MyMap::new();
        add_some_data(&mut fmap, 4);
        assert_eq!(fmap.size(), 4);

        let old_val = fmap.insert(String::from("bar"), 2000);
        
        assert_eq!(fmap.size(), 4);
        assert!(fmap.get(&String::from("bar")) == Some(&2000));
        assert_eq!(old_val, Some(200));
    }

    #[test]
    fn remove_items() {
        let mut fmap = MyMap::new();
        add_some_data(&mut fmap, 4);
        assert!(fmap.size() == 4);

        let old_val_of_bar = fmap.remove(&String::from("bar"));
        let old_val_of_zoo = fmap.remove(&String::from("zoo"));

        assert_eq!(fmap.size(), 3);
        assert_eq!(old_val_of_bar, Some(200));
        assert_eq!(old_val_of_zoo, None);
        assert_eq!(fmap.exists(&String::from("bar")), false);
        assert_eq!(fmap.exists(&String::from("zoo")), false);
    }
}
