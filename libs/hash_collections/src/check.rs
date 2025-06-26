use const_primes::is_prime;

pub struct Check<const U: bool>;
pub trait IsTrue {}

impl IsTrue for Check<true> {}

pub const fn is_prime_and_within_limit(c: usize, max_cap: usize) -> bool {
    is_prime(c as u64) && c <= max_cap
}