#![allow(incomplete_features)]
#![allow(internal_features)]
#![feature(adt_const_params)]
#![feature(const_type_id)]
#![feature(generic_const_exprs)]
#![feature(core_intrinsics)]
#![feature(inherent_associated_types)]

mod check;
pub mod hash_map;
pub mod hash_graph;

pub use crate::hash_map::FixedSizeHashMap;
pub use crate::hash_map::FixedSizeHashSet;
