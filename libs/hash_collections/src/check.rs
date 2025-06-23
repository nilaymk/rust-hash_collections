pub struct Check<const U: bool>;
pub trait IsTrue {}

impl IsTrue for Check<true> {}
