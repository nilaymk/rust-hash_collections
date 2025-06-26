use std::{hash::{Hash, Hasher}, ops::Index};
use crate::{hash_map::{FixedSizeHashMapImpl, OutOfCapacityError}, FixedSizeHashMap, FixedSizeHashSet};
use std::mem;

use crate::check::{Check, IsTrue, is_prime_and_within_limit};

struct Edge {
    _to: usize,
    _weight: usize,
}

struct Node<V> {
    _value: V,
    _connected_to: FixedSizeHashMap<usize, u32, 151>,
    _connected_from: FixedSizeHashSet<usize, 151>,
}

struct FixedSizeHashGraphImpl<K, V, const C: usize, H>
where
    Check<{ is_prime_and_within_limit(C, 25013) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    _hash_map: FixedSizeHashMapImpl<K, Node<V>, C, H>,
}

impl<'a, K: 'a, V: 'a, const C: usize, H> FixedSizeHashGraphImpl<K, V, C, H>
where
    Check<{ is_prime_and_within_limit(C, 25013) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    fn _insert_or_update_contained_value(&'a mut self, key: K, value: V) -> Result<usize, OutOfCapacityError>  {
        match self._hash_map._get_mut_value_and_index_of(&key) {
            Some((node, index)) => {
                let _ = mem::replace(&mut node._value, value);
                return Result::Ok(index)
            }
            None => {
                match self._hash_map._insert_get_index(key, Node {
                    _value: value,
                    _connected_to: Default::default(),
                    _connected_from: Default::default()
                }) {
                    Ok((index, _)) => return Ok(index),
                    Err(e) => return Err(e)
                }
            }
        }
    }

    pub fn insert(&'a mut self, key_value: (K, V), connections: Vec<(K, V)>) -> Result<(), OutOfCapacityError> {
        let Ok(index) = self._insert_or_update_contained_value(key_value.0, key_value.1) else {
            return Result::Err(OutOfCapacityError{capacity: self._hash_map.capacity()});
        };

        for (to_key, to_value) in connections {
            let Ok(to_index) = self._insert_or_update_contained_value(to_key, to_value) else {
                return Result::Err(OutOfCapacityError{capacity: self._hash_map.capacity()});
            };
            
            if index == to_index {continue;}

            if let Some(node) = self._hash_map._get_mut_value_at(index) {
                match node._connected_to.get_mut(&to_index) {
                    Some(edge_weight) => {*edge_weight+=1;},
                    None => {let _ = node._connected_to.insert(to_index, 1);}
                }
            }
            
            if let Some(to_node) = self._hash_map._get_mut_value_at(to_index) {
                let _ = to_node._connected_from.insert(index, ());
            }
        }

        return Ok(())
    }

    fn connect_to(&mut self, k: &K, to_keys: Vec<&K>) {
        let Some(index) = self._hash_map._get_index_of(k) else {
            return
        };
        
        for to_key in to_keys {
            let Some(to_index) = self._hash_map._get_index_of(to_key) else {
                continue;
            };
            if index != to_index {
                if let Some(node) = self._hash_map._get_mut_value_at(index) {
                    let _ = node._connected_to.insert_or(to_index, 1, |weight| *weight+=1);
                }
                if let Some(to_node) = self._hash_map._get_mut_value_at(to_index) {
                    let _ = to_node._connected_from.insert(index, ());
                }
            }
        }
    }

    fn remove(&mut self, key: &K) {
        let (index, to_indexes, from_indexes): (usize, Vec<usize>, Vec<usize>) =
            match self._hash_map._get_mut_value_and_index_of(key) {
                Some((node, index)) => {
                    (
                        index,
                        node._connected_to.iter_head().map(|kv| *kv.0).collect(),
                        node._connected_from.iter_head().map(|kv| *kv.0).collect()
                    )
                },
                None => return,
            };
        
        for to_index in to_indexes {
            let Some(to_node) = self._hash_map._get_mut_value_at(to_index) else {
                continue;
            };
            to_node._connected_from.remove(&index);
        }

        for from_index in from_indexes {
            let Some(from_node) = self._hash_map._get_mut_value_at(from_index) else {
                continue;
            };
            from_node._connected_to.remove(&index);
        }

        self._hash_map.remove(&key);
    }

    // fn disconnect_from(&mut self, key: &K, to_keys: Vec<&K>) {
    //     let Some((node, index)) = self._hash_map._get_mut_and_index(key) else {
    //         return
    //     };
        
    //     for to_key in to_keys {
    //         let Some((to_node, to_index)) = self._hash_map._get_mut_and_index(to_key) else {
    //             continue;
    //         };

    //         let Some(weight) = node._to._get_mut_at(to_index) else {
    //             continue;
    //         };

    //         *weight-=1;
    //         if *weight == 0 {
    //             to_node._from.remove(&index);
    //         }
    //     }
    // }

    // fn disconnect_all() {

    // }
}