//! Custom traits to exercise different patterns.

use core::fmt::Debug;

// Supertrait chain
pub trait Super: Debug {}
pub trait Sub: Super {}

// A trait with a `where Self: ...` clause. We'll partly use it.
pub trait SelfWhere
where
    Self: Sized + Clone, // Sized used (method needs a receiver). Clone intentionally **unused**
{
    fn touch(&self) {}
}

// A trivial type implementing the above.
#[derive(Clone, Debug)]
pub struct Thing {
    pub n: i32,
}

impl Super for Thing {}
impl Sub for Thing {}
impl SelfWhere for Thing {}
