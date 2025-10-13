//! Test crate for trait-bound pruning.

pub mod traits;
pub mod a;
pub mod b;
pub mod c;

// Re-exports so a single module path works in tests/tools.
pub use a::*;
pub use b::*;
pub use c::*;
pub use traits::*;

#[cfg(test)]
mod smoke {
    use super::*;

    #[test]
    fn it_compiles_and_runs() {
        // a.rs
        let _ = unused_bound_clone(10);                          // Clone **unused**
        let _ = used_bound_clone(String::from("x"));             // Clone used
        let _ = where_unused_default(7u8);                       // Default **unused**
        let _ = where_used_default(Some(5u32));                  // Default used
        let _ = hrtb_used(|s: &str| s.len());                    // HRTB used
        hrtb_unused::<fn(&str) -> usize>();                      // HRTB **unused**

        // b.rs
        let w = Wrapper(3u32);
        let _ = w.copied();                                      // Copy used (method-level bound)
        let _ = Wrapper::<i32>::new_default();                   // Default used (impl-level bound)
        w.id();                                                  // Ord **unused** (method-level where)

        // c.rs
        let t = Thing { n: 1 };
        t.touch();                                               // `where Self: Sized + Clone` (Sized used; Clone **unused**)
        assert!(format!("{:?}", t).len() > 0);                   // Super: Debug used via Super
    }
}

// //! Test crate for trait-bound pruning.

// pub mod traits;
// pub mod a;
// pub mod b;
// pub mod c;

// // Re-exports so a single module path works in tests/tools.
// pub use a::*;
// pub use b::*;
// pub use c::*;
// pub use traits::*;

// #[cfg(test)]
// mod smoke {
//     use super::*;

//     #[test]
//     fn it_compiles_and_runs() {
//         // a.rs
//         let _ = unused_bound_clone(10);                          // Clone **unused**
//         let _ = used_bound_clone(String::from("x"));             // Clone used
//         let _ = where_unused_default(7u8);                       // Default **unused**
//         let _ = where_used_default(Some(5u32));                  // Default used
//         let _ = hrtb_used(|s: &str| s.len());                    // HRTB used
//         hrtb_unused::<fn(&str) -> usize>();                      // HRTB **unused**

//         // b.rs
//         let w = Wrapper(3u32);
//         let _ = w.copied();                                      // Copy used (method-level bound)
//         let _ = Wrapper::<i32>::new_default();                   // Default used (impl-level bound)
//         w.id();                                                  // Ord **unused** (method-level where)

//         // c.rs
//         let t = Thing { n: 1 };
//         t.touch();                                               // `where Self: Sized + Clone` (Sized used; Clone **unused**)
//         assert!(format!("{:?}", t).len() > 0);                   // Super: Debug used via Super
//     }
// }
