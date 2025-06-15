use const_primes::{is_prime};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::mem;
use std::ops::{Index, IndexMut};

struct Entry <K, V, const C: usize> {
    key: K,
    value: V,
    next: usize,
    prev: usize
}

impl <K, V, const C: usize> Entry <K, V, C> {
    fn new(key: K, value: V) -> Entry<K, V, C> {
        Entry {
            key: key,
            value: value,
            next: C,
            prev: C,
        }
    }
}

pub struct FixedSizeHashMap <K, V, const C: usize>  {
    _data: Vec<Option<Entry<K, V, C>>>,
    _size: usize,
    _head: usize,
    _tail: usize
}

struct Check<const U: bool>;
trait IsTrue {}
impl IsTrue for Check<true> {}

pub const fn is_prime_and_within_limit(c: usize, max_cap: usize) -> bool {
    is_prime(c as u64) && c <= max_cap
}

impl <K: Hash + std::cmp::Eq, V, const C: usize> FixedSizeHashMap <K, V, C>
where
    Check<{is_prime_and_within_limit(C, 25013)}>: IsTrue,
{
    const CAPACITY: usize = C;

    pub fn new() -> FixedSizeHashMap<K, V, C> {
        let mut map = FixedSizeHashMap {
            _data: Vec::new(),
            _size: 0,
            _head: Self::CAPACITY,
            _tail: Self::CAPACITY,
        };
        map._data.resize_with(Self::CAPACITY, Default::default);
        map
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let i = self._find_index(&key);
        if i == Self::CAPACITY {
            return None
        }

        let mut old_val: Option<V> = None;
        match self._data[i].as_mut() {
            Some(entry) => {
                old_val = Some(mem::replace(&mut entry.value, value));
                self._remove_from_list(i);
                self._move_to_front_of_list(i);
            }
            None => {
                let entry: Entry<K, V, C> = Entry::new(key, value);
                self._data[i] = Some(entry);
                self._move_to_front_of_list(i);
                self._size += 1;
            }
        }
        old_val
    }

    pub fn exists(&self, key: &K) -> bool {
        let i = self._find_index(key);
        i != Self::CAPACITY && self._data[i].is_some()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let i = self._find_index(&key);
        match self._get_key_val_at(i) {
            Some(key_val_pair) => Some(key_val_pair.1),
            None => None
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let i = self._find_index(&key);

        if i == Self::CAPACITY {
            return None
        }

        match self._data[i].as_mut() {
            Some(data) => Some(&mut data.value),
            None => None
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let i = self._find_index(&key);
        if i == Self::CAPACITY {
            return None
        }
   
        match self._data[i].as_mut() {
            None => {},
            Some(_) => {
                self._size -= 1;
                self._remove_from_list(i);
            } 
        }
        
        match mem::replace(&mut self._data[i], None) {
            None => None,
            Some(key_val) => Some(key_val.value)
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
            _fn_next: |entry| {entry.next}
         }
    }

    pub fn iter_tail(&self) -> MapIterator<'_, K, V, C, impl Fn(&Entry<K, V, C>) -> usize> {
        MapIterator {
            _remaining: self._size,
            _current: self._tail,
            _data: &self._data,
            _fn_next: |entry| {entry.prev}
         }
    }

    fn _get_key_val_at(&self, i: usize) -> Option<(&K, &V)> {
        if i == Self::CAPACITY {
            return None
        }
        match self._data[i].as_ref() {
            Some(data) => Some((&data.key, &data.value)),
            None => None
        }
    }

    fn _find_index(&self, key: &K) -> usize {
        let mut s = DefaultHasher::new();
        key.hash(&mut s);
        let already_visited = (s.finish() % Self::CAPACITY as u64) as usize;
        let mut index = already_visited;
        while self._data[index].as_ref().is_some_and(|key_val| {key_val.key != *key}) {
            index = (index + 1) % Self::CAPACITY; // linear probing
            if index == already_visited {
                return Self::CAPACITY
            }
        }
        index
    }

    fn _move_to_front_of_list(
        &mut self,
        i: usize
    ) {
        if self._size == 0 {
            debug_assert!(self._head == Self::CAPACITY && self._tail == Self::CAPACITY);
            self._head = i;
            self._tail = i;
        } else {
            unsafe {
                self._data.get_unchecked_mut(self._head).as_mut().unwrap().prev = i;
                let entry = self._data.get_unchecked_mut(i).as_mut().unwrap();
                entry.next = self._head;
                entry.prev = Self::CAPACITY;
            }

            self._head = i;
        }
    }
    
    fn _remove_from_list(&mut self, i: usize) {
        let entry_next: usize;
        let entry_prev: usize;
        unsafe {
            let entry = self._data.get_unchecked(i).as_ref().unwrap();
            entry_next = entry.next;
            entry_prev = entry.prev;
        }

        if entry_prev != Self::CAPACITY {
            unsafe {
                self._data.get_unchecked_mut(entry_prev).as_mut().unwrap().next = entry_next;
            }
        } else {
            self._head = entry_next;
        }
        if entry_next != Self::CAPACITY {
            unsafe {
                let _ = self._data.get_unchecked_mut(entry_next).as_mut().is_some_and(|next_entry| {
                    next_entry.prev = entry_prev;
                    true
                });
            }
        } else {
            self._tail = entry_prev;
        }
    }

    fn _get_entry_at(&self, i: usize) -> Option<&Entry<K, V, C>> {
        if i == Self::CAPACITY {
            return None
        }
        self._data[i].as_ref()
    }

}

impl <K: Hash + std::cmp::Eq, V, const C: usize> Index<&K> for FixedSizeHashMap <K, V, C>
where
    Check<{is_prime_and_within_limit(C, 25013)}>: IsTrue,
{
    type Output = V;
    fn index(&self, key: &K) -> &Self::Output {
        self.get(key).expect("Panic! not in map")
    }
}

impl <K: Hash + std::cmp::Eq, V, const C: usize> IndexMut<&K> for FixedSizeHashMap <K, V, C>
where
    Check<{is_prime_and_within_limit(C, 25013)}>: IsTrue,
{
    fn index_mut(&mut self, key: &K) -> &mut Self::Output {
        self.get_mut(key).expect("Panic! not in map")
    }
}

pub struct MapIterator<'a, K: 'a, V: 'a, const C: usize, Next>
where Next: Fn(&Entry<K, V, C>) -> usize
{
    _remaining: usize, 
    _current: usize,
    _data: &'a Vec<Option<Entry<K, V, C>>>,
    _fn_next: Next,
}

impl <'a, K: 'a, V: 'a, const C: usize, Next> Iterator
for MapIterator<'a, K, V, C, Next>
where Next: Fn(&Entry<K, V, C>) -> usize  {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self._current >= C {
            return None
        }

        match self._data[self._current].as_ref() {
            Some(entry) => {
                self._remaining -= 1;
                self._current = (self._fn_next)(&entry);
                Some((&entry.key, &entry.value))
            },
            None => None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self._remaining, Some(self._remaining))
    }

    fn count(self) -> usize {
        self._remaining
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
        assert!(fmap.head() == None);
        assert!(fmap.tail() == None);

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
        assert_eq!(fmap.head(), Some( (&String::from("baz"), &300) ));
        assert_eq!(fmap.tail(), Some( (&String::from("foo"), &100) ));

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
    fn remove_items_from_middle() {
        let mut fmap = MyMap::new();
        add_some_data(&mut fmap, 4);
        assert!(fmap.size() == 4);

        let old_val_of_bar = fmap.remove(&String::from("bar"));
        let old_val_of_baz = fmap.remove(&String::from("baz"));

        assert_eq!(fmap.size(), 2);
        assert_eq!(old_val_of_bar, Some(200));
        assert_eq!(old_val_of_baz, Some(300));
        assert_eq!(fmap.exists(&String::from("bar")), false);
        assert_eq!(fmap.exists(&String::from("zoo")), false);
        assert_eq!(fmap.head(), Some( (&String::from("bat"), &400) ));
        assert_eq!(fmap.tail(), Some( (&String::from("foo"), &100) ));
    }

    #[test]
    fn remove_head_and_tail_item() {
        let mut fmap = MyMap::new();
        add_some_data(&mut fmap, 4);
        assert!(fmap.size() == 4);
        
        let _ = fmap.remove(&String::from("bat"));
        let _ = fmap.remove(&String::from("foo"));

        assert_eq!(fmap.size(), 2);
        assert_eq!(fmap.head(), Some( (&String::from("baz"), &300) ));
        assert_eq!(fmap.tail(), Some( (&String::from("bar"), &200) ));
    }

       #[test]
    fn remove_non_existent_item() {
        let mut fmap = MyMap::new();
        add_some_data(&mut fmap, 4);
        assert!(fmap.size() == 4);

        let old_val_of_zoo = fmap.remove(&String::from("zoo"));

        assert_eq!(fmap.size(), 4);
        assert_eq!(old_val_of_zoo, None);
        assert_eq!(fmap.head(), Some( (&String::from("bat"), &400) ));
        assert_eq!(fmap.tail(), Some( (&String::from("foo"), &100) ));
    }

    #[test]
    fn in_place_update() {
        let mut fmap = MyMap::new();
        add_some_data(&mut fmap, 4);
        assert!(fmap.size() == 4);

        fmap.get_mut(&String::from("bar")).and_then(|v| {*v += 1000; Some(true)});
        assert_eq!(fmap.get(&String::from("bar")), Some(&1200));
    }

    #[test]
    fn indexed_read_and_mutate() {
        let mut fmap = MyMap::new();
        add_some_data(&mut fmap, 4);
        assert!(fmap.size() == 4);

        assert_eq!(fmap[&String::from("bar")], 200);
        fmap[&String::from("bar")] += 1000;

        assert_eq!(fmap.get(&String::from("bar")), Some(&1200));
    }

    #[test]
    fn forward_iteration() {
        let mut fmap = MyMap::new();
        add_some_data(&mut fmap, 4);
        assert!(fmap.size() == 4);

        let mut iter = fmap.iter_head();

        assert_eq!(iter.size_hint(), (4, Some(4)));
        assert_eq!(iter.next(), Some((&String::from("bat"), &400)) );
        assert_eq!(iter.size_hint(), (3, Some(3)));
        assert_eq!(iter.next(), Some((&String::from("baz"), &300)) );
        assert_eq!(iter.size_hint(), (2, Some(2)));
        assert_eq!(iter.next(), Some((&String::from("bar"), &200)) );
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next(), Some((&String::from("foo"), &100)) );
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None );
        assert_eq!(iter.next(), None ); 
    }

        #[test]
    fn backward_iteration() {
        let mut fmap = MyMap::new();
        add_some_data(&mut fmap, 4);
        assert!(fmap.size() == 4);

        let mut iter = fmap.iter_tail();

        assert_eq!(iter.size_hint(), (4, Some(4)));
        assert_eq!(iter.next(), Some((&String::from("foo"), &100)) );
        assert_eq!(iter.size_hint(), (3, Some(3)));
        assert_eq!(iter.next(), Some((&String::from("bar"), &200)) );
        assert_eq!(iter.size_hint(), (2, Some(2)));
        assert_eq!(iter.next(), Some((&String::from("baz"), &300)) );
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next(), Some((&String::from("bat"), &400)) );
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None );
        assert_eq!(iter.next(), None ); 
    }
}
