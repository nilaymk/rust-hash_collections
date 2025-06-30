use std::{hash::{Hash, Hasher}, ops::Index};
use crate::hash_map_internal::{FixedSizeHashMapImpl, Entry};
use crate::hash_map::{
    OutOfCapacityError, 
    FixedSizeHashMap,
    FixedSizeHashSet
};
use std::mem;

use crate::check::{Check, IsTrue, is_prime_and_within_limit};


type Edges = FixedSizeHashMap<usize, u32, 151>;
type FromNodes = FixedSizeHashSet<usize, 151>;

struct Node<K, V, const C: usize> {
    _key: K,
    _value: V,
    _next: usize,
    _prev: usize,
    _connected_to: Edges,
    _connected_from: FromNodes
}

impl<K, V, const C: usize> Entry<K, V, C> for Node<K, V, C> {
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
            _connected_from: Default::default(),
            _connected_to: Default::default(),
        }
    }
}

struct FixedSizeHashGraphImpl<K, V, const C: usize, H>
where
    Check<{ is_prime_and_within_limit(C, 25013) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    _hash_map: FixedSizeHashMapImpl<K, V, C, H, Node<K, V, C>>,
}

impl<'a, K: 'a, V: 'a, const C: usize, H> FixedSizeHashGraphImpl<K, V, C, H>
where
    Check<{ is_prime_and_within_limit(C, 25013) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    pub fn insert(&'a mut self, key_value: (K, V), connections: Vec<(K, V)>) -> Result<(), OutOfCapacityError> {
        let Ok((index, _)) = self._hash_map.insert_get_index(key_value.0, key_value.1) else {
            return Result::Err(OutOfCapacityError{capacity: self._hash_map.capacity()});
        };

        for (to_key, to_value) in connections {
            let Ok((to_index, _)) = self._hash_map.insert_get_index(to_key, to_value) else {
                return Result::Err(OutOfCapacityError{capacity: self._hash_map.capacity()});
            };
            
            if index == to_index {continue;}

            if let Some(node) = self._hash_map.get_mut_entry_at(index) {
                match node._connected_to.get_mut(&to_index) {
                    Some(edge_weight) => {*edge_weight+=1;},
                    None => {let _ = node._connected_to.insert(to_index, 1);}
                }
            }
            
            if let Some(to_node) = self._hash_map.get_mut_entry_at(to_index) {
                let _ = to_node._connected_from.insert(index, ());
            }
        }

        return Ok(())
    }

    fn connect_to(&mut self, k: &K, to_keys: Vec<&K>) {
        let Some(index) = self._hash_map.get_index_of(k) else {
            return
        };
        
        for to_key in to_keys {
            let Some(to_index) = self._hash_map.get_index_of(to_key) else {
                continue;
            };
            if index != to_index {
                if let Some(node) = self._hash_map.get_mut_entry_at(index) {
                    let _ = node._connected_to.insert_or(to_index, 1, |weight| *weight+=1);
                }
                if let Some(to_node) = self._hash_map.get_mut_entry_at(to_index) {
                    let _ = to_node._connected_from.insert(index, ());
                }
            }
        }
    }

    fn remove(&mut self, key: &K) {
        let (index, edges, from_nodes): (usize, Edges, FromNodes) =
            match self._hash_map.get_mut_entry_and_index_of(key) {
                Some((node, index)) => {(
                    index,
                    mem::take(&mut node._connected_to),
                    mem::take(&mut node._connected_from)
                )},
                None => return,
            };
        
        for (edge_index, _) in edges.iter_head() {
            if let Some(to_node) = self._hash_map.get_mut_entry_at(*edge_index) {
                to_node._connected_from.remove(&index);
            }
        }

        for (from_index, _) in from_nodes.iter_head() {
            if let Some(from_node) = self._hash_map.get_mut_entry_at(*from_index) {
                from_node._connected_to.remove(&index);
            };
        }

        self._hash_map.remove(&key);
    }

    fn disconnect_from(&mut self, key: &K, to_keys: Vec<&K>) {
        let Some(index) = self._hash_map.get_index_of(key) else {return};
        
        for to_key in to_keys {
            if let Some(to_index) = self._hash_map.get_index_of(to_key) {
                if let Some(node) = self._hash_map.get_mut_entry_at(index){
                    if let Some(edge_weight) = node._connected_to.get_mut(&to_index) {
                        if *edge_weight == 1 {
                            node._connected_to.remove(&to_index);
                            if let Some(to_node) = self._hash_map.get_mut_entry_at(to_index) {
                                to_node._connected_from.remove(&index);
                            }
                        } else {
                            *edge_weight-=1;
                        }
                    }
                }
            }
        }
    }

    fn disconnect_all(&mut self, key: &K) {
        let Some((node, index)) = self._hash_map.get_mut_entry_and_index_of(key) else {return};
        
        for (to_index, _) in mem::take(&mut node._connected_to).iter_head() {
            if let Some(to_node) = self._hash_map.get_mut_entry_at(*to_index) {
                to_node._connected_from.remove(&index);
            }
        }
    }
}
