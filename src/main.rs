#![feature(adt_const_params, generic_const_exprs)] 

use const_primes::{is_prime};
use const_guards::guard;

type KeyType = String;
type ValType = i64;
type CapType = u16;

struct Entry {
    key: KeyType,
    value: ValType,
    next: CapType,
    previous: CapType
}

struct FixedSizeHashMap <const C: CapType>  {
    _data: Vec<Entry>,
    _size: CapType,
}

#[guard(is_prime(C as u64))]
#[guard(C < 1000)]
impl <const C: CapType> FixedSizeHashMap <C>  {
    const CAPACITY: CapType = C;
    const CHECK: bool = is_prime(C as u64);

    pub fn new() -> FixedSizeHashMap<C> {        
        FixedSizeHashMap { _data: Vec::with_capacity(Self::CAPACITY as usize), _size: 0 }
    }

    pub fn insert(&mut self, key: KeyType, v: ValType ) -> Option<ValType> {
        None

    }

    pub fn exists(&self, key: KeyType) -> bool {    
        false
    }

    pub fn get(&self, key: KeyType) -> Option<&ValType> {
        None
    }

    pub const fn capacity(&self) -> CapType {
        Self::CAPACITY
    }
    
    const fn cap() -> CapType {
        Self::CAPACITY
    }

    pub fn size(&self) -> CapType {
        self._size
    }
}

fn main() {
    let fmap: FixedSizeHashMap<3> = FixedSizeHashMap::new();

    println!("Hello, world!");
}
