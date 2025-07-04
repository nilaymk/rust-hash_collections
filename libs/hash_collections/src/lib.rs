#![allow(incomplete_features)]
#![allow(internal_features)]
#![feature(adt_const_params)]
#![feature(const_type_id)]
#![feature(generic_const_exprs)]
#![feature(core_intrinsics)]
#![feature(inherent_associated_types)]

mod check;
mod hash_map_internal;

pub mod hash_graph;
pub mod hash_map;
pub mod hash_set;
pub mod errors;

const MAX_CAPACITY: usize = 50849;

pub use crate::hash_graph::FixedSizeHashGraphMap;
pub use crate::hash_map::FixedSizeHashMap;
pub use crate::hash_set::FixedSizeHashSet;
pub use crate::errors::OutOfCapacityError;

mod unittests;
