//! Custom traits to exercise different patterns.
use core::fmt::Debug;
pub trait Super: Debug {}
pub trait Sub: Super {}
pub trait SelfWhere {
    fn touch(&self) {}
}
#[derive(Clone, Debug)]
pub struct Thing {
    pub n: i32,
}
impl Super for Thing {}
impl Sub for Thing {}
impl SelfWhere for Thing {}


// //! Custom traits to exercise different patterns.

// use core::fmt::Debug;

// // Supertrait chain
// pub trait Super: Debug {}
// pub trait Sub: Super {}

// // A trait with a `where Self: ...` clause. We'll partly use it.
// pub trait SelfWhere
// where
//     Self: Sized + Clone, // Unused
// {
//     fn touch(&self) {}
// }

// // A trivial type implementing the above.
// #[derive(Clone, Debug)]
// pub struct Thing {
//     pub n: i32,
// }

// impl Super for Thing {}
// impl Sub for Thing {}
// impl SelfWhere for Thing {}
