#![allow(dead_code)]

use crate::{
    hash_map::{MapIter, FixedSizeHashMap},
    hash_set::{FixedSizeHashSet},
    hash_map_internal::{Entry, FixedSizeHashMapImpl},
    OutOfCapacityError
};

use std::hash::{DefaultHasher, Hash, Hasher};
use std::mem;

use crate::check::{Check, IsTrue, is_prime_and_within_limit};

const MAX_EDGES: usize = 151;

type OutEdges = FixedSizeHashMap<usize, u32, MAX_EDGES>;
type InEdges = FixedSizeHashSet<usize, MAX_EDGES>;

pub struct NodeEntry<K, V, const C: usize> {
    _key: K,
    _value: V,
    _next: usize,
    _prev: usize,
    _out_edges: OutEdges,
    _in_edges: InEdges,
}

impl<K, V, const C: usize> Entry<K, V, C> for NodeEntry<K, V, C> {
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
            _in_edges: Default::default(),
            _out_edges: Default::default(),
        }
    }
}

pub struct Node<'a, K, V, const C: usize, H>
where
    Check<{ is_prime_and_within_limit(C, crate::MAX_CAPACITY) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    _node_entry: &'a NodeEntry<K, V, C>,
    _graph: &'a FixedSizeHashGraphImpl<K, V, C, H>,
}

impl<'a, K, V, const C: usize, H> Node<'a, K, V, C, H>
where
    Check<{ is_prime_and_within_limit(C, crate::MAX_CAPACITY) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    pub fn iter_out_edges(&self) -> EdgeIter<'_, K, V, C, H> {
        EdgeIter {
            _inner_iter: self._node_entry._out_edges.iter_head(),
            _graph: self._graph,
        }
    }

    pub fn out_edge_weight(&self, to_key: &K) -> u32 {
        self._graph
            ._hash_map
            .get_index_of(to_key)
            .map_or(0, |to_index| {
                self._node_entry._out_edges.get(&to_index).map_or(0, |v| *v)
            })
    }

    pub fn key(&self) -> &'a K {
        self._node_entry.key()
    }
    pub fn value(&self) -> &'a V {
        self._node_entry.value()
    }
}

pub struct EdgeIter<'a, K, V, const C: usize, H>
where
    Check<{ is_prime_and_within_limit(C, crate::MAX_CAPACITY) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    _inner_iter: MapIter<'a, usize, u32, MAX_EDGES>,
    _graph: &'a FixedSizeHashGraphImpl<K, V, C, H>,
}

impl<'a, K, V, const C: usize, H> Iterator for EdgeIter<'a, K, V, C, H>
where
    Check<{ is_prime_and_within_limit(C, crate::MAX_CAPACITY) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    type Item = (Node<'a, K, V, C, H>, u32);
    fn next(&mut self) -> Option<Self::Item> {
        self._inner_iter
            .next()
            .map(|(edge_index, edge_weight)| {
                self._graph
                    ._hash_map
                    .get_entry_at(*edge_index)
                    .map(|edge_entry| {
                        (
                            Node {
                                _node_entry: edge_entry,
                                _graph: self._graph,
                            },
                            *edge_weight,
                        )
                    })
            })
            .flatten()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self._inner_iter.size_hint()
    }
    fn count(self) -> usize {
        self._inner_iter.count()
    }
}

pub struct FixedSizeHashGraphImpl<K, V, const C: usize, H>
where
    Check<{ is_prime_and_within_limit(C, crate::MAX_CAPACITY) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    _hash_map: FixedSizeHashMapImpl<K, V, C, H, NodeEntry<K, V, C>>,
    _empty_out_edges: OutEdges,
    _empty_in_edges: InEdges,
}

impl<'a, K: 'a, V: 'a, const C: usize, H> FixedSizeHashGraphImpl<K, V, C, H>
where
    Check<{ is_prime_and_within_limit(C, crate::MAX_CAPACITY) }>: IsTrue,
    K: Hash + std::cmp::Eq,
    H: Default + Hasher,
{
    pub fn new() -> Self {
        Self {
            _hash_map: Default::default(),
            _empty_in_edges: InEdges::placeholder(),
            _empty_out_edges: OutEdges::placeholder()
        }
    }

    pub fn insert(
        &'a mut self,
        key_value: (K, V),
        connections: Vec<(K, V)>,
    ) -> Result<(), OutOfCapacityError> {
        let Ok((index, _)) = self._hash_map.insert_get_index(key_value.0, key_value.1) else {
            return Result::Err(OutOfCapacityError {
                capacity: self._hash_map.capacity(),
            });
        };

        for (to_key, to_value) in connections {
            let Ok((to_index, _)) = self._hash_map.insert_get_index(to_key, to_value) else {
                return Result::Err(OutOfCapacityError {
                    capacity: self._hash_map.capacity(),
                });
            };

            if index == to_index {
                continue;
            }

            if let Some(node) = self._hash_map.get_mut_entry_at(index) {
                match node._out_edges.get_mut(&to_index) {
                    Some(edge_weight) => {
                        *edge_weight += 1;
                    }
                    None => {
                        let _ = node._out_edges.insert(to_index, 1);
                    }
                }
            }

            if let Some(to_node) = self._hash_map.get_mut_entry_at(to_index) {
                let _ = to_node._in_edges.insert(index);
            }
        }

        return Ok(());
    }

    pub fn connect_to(&mut self, k: &K, to_keys: Vec<&K>) {
        let Some(index) = self._hash_map.get_index_of(k) else {
            return;
        };

        for to_key in to_keys {
            let Some(to_index) = self._hash_map.get_index_of(to_key) else {
                continue;
            };
            if index != to_index {
                if let Some(node) = self._hash_map.get_mut_entry_at(index) {
                    let _ = node
                        ._out_edges
                        .insert_or(to_index, 1, |weight| *weight += 1);
                }
                if let Some(to_node) = self._hash_map.get_mut_entry_at(to_index) {
                    let _ = to_node._in_edges.insert(index);
                }
            }
        }
    }

    pub fn remove(&mut self, key: &K) {
        let (index, edges, from_nodes): (usize, OutEdges, InEdges) =
            match self._hash_map.get_mut_entry_and_index_of(key) {
                Some((node, index)) => (
                    index,
                    mem::take(&mut node._out_edges),
                    mem::take(&mut node._in_edges),
                ),
                None => return,
            };

        for (edge_index, _) in edges.iter_head() {
            if let Some(to_node) = self._hash_map.get_mut_entry_at(*edge_index) {
                to_node._in_edges.remove(&index);
            }
        }

        for from_index in from_nodes.iter_head() {
            if let Some(from_node) = self._hash_map.get_mut_entry_at(*from_index) {
                from_node._out_edges.remove(&index);
            };
        }

        self._hash_map.remove(&key);
    }

    pub fn disconnect_from(&mut self, key: &K, to_keys: Vec<&K>) {
        let Some(index) = self._hash_map.get_index_of(key) else {
            return;
        };

        for to_key in to_keys {
            if let Some(to_index) = self._hash_map.get_index_of(to_key) {
                if let Some(node) = self._hash_map.get_mut_entry_at(index) {
                    if let Some(edge_weight) = node._out_edges.get_mut(&to_index) {
                        if *edge_weight == 1 {
                            node._out_edges.remove(&to_index);
                            if let Some(to_node) = self._hash_map.get_mut_entry_at(to_index) {
                                to_node._in_edges.remove(&index);
                            }
                        } else {
                            *edge_weight -= 1;
                        }
                    }
                }
            }
        }
    }

    pub fn disconnect_all(&mut self, key: &K) {
        let Some((node, index)) = self._hash_map.get_mut_entry_and_index_of(key) else {
            return;
        };

        for (to_index, _) in mem::take(&mut node._out_edges).iter_head() {
            if let Some(to_node) = self._hash_map.get_mut_entry_at(*to_index) {
                to_node._in_edges.remove(&index);
            }
        }
    }

    pub fn node(&self, key: &K) -> Option<Node<'_, K, V, C, H>> {
        self._hash_map
            .get_entry_and_index_of(key)
            .map(|(e, _)| Node {
                _node_entry: e,
                _graph: &self,
            })
    }

    pub fn iter_out_edges(&self, k: &K) -> EdgeIter<'_, K, V, C, H> {
        if let Some((node_entry, _)) = self._hash_map.get_entry_and_index_of(k) {
            EdgeIter {
                _inner_iter: node_entry._out_edges.iter_head(),
                _graph: &self,
            }     
        } else {
            EdgeIter {
                _inner_iter: self._empty_out_edges.iter_head(),
                _graph: &self,
            }       
        }
    }

    pub fn out_edge_weight(&self, from_key: &K, to_key: &K) -> u32 {
        self._hash_map.get_entry_and_index_of(from_key)
            .map_or(0, |(node_entry, _)| {
                self._hash_map.get_index_of(to_key).map_or(
                    0, |to_index| node_entry._out_edges.get(&to_index).map_or(
                        0, |weight| *weight
                    )
                )
            })
    }

}

pub type FixedSizeHashGraphMap<K, V, const C: usize> =
    FixedSizeHashGraphImpl<K, V, C, DefaultHasher>;
