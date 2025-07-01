use std::error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct OutOfCapacityError {
    pub capacity: usize,
}

impl fmt::Display for OutOfCapacityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HashMap has reached its capacity of {} entries",
            self.capacity
        )
    }
}

impl error::Error for OutOfCapacityError {}