//! Free functions with various bound placements and (un)usage.
/// Bound on type param; **unused** in body.
pub fn unused_bound_clone<T>(x: T) -> T {
    x
}
/// Bound on type param; **used** in body.
pub fn used_bound_clone<T: Clone>(x: T) -> T {
    let _y = x.clone();
    x
}
/// `where`-clause bound; **unused** in body.
pub fn where_unused_default<T>(x: T) -> T {
    x
}
/// `where`-clause bound; **used** in body.
pub fn where_used_default<T>(x: Option<T>) -> T
where
    T: Default,
{
    x.unwrap_or_default()
}
/// HRTB bound; **used** in body.
pub fn hrtb_used<F>(f: F) -> usize
where
    for<'a> F: Fn(&'a str) -> usize,
{
    f("hello")
}
/// HRTB bound; **unused** in body.
pub fn hrtb_unused<F>() {}


// //! Free functions with various bound placements and (un)usage.

// /// Bound on type param; **unused** in body.
// pub fn unused_bound_clone<T: Clone>(x: T) -> T {
//     // NOTE: we never call `x.clone()`, so `Clone` is removable if not required elsewhere.
//     x
// }

// /// Bound on type param; **used** in body.
// pub fn used_bound_clone<T: Clone>(x: T) -> T {
//     let _y = x.clone(); // Uses Clone
//     x
// }

// /// `where`-clause bound; **unused** in body.
// pub fn where_unused_default<T>(x: T) -> T
// where
//     T: Default, // Default not used
// {
//     x
// }

// /// `where`-clause bound; **used** in body.
// pub fn where_used_default<T>(x: Option<T>) -> T
// where
//     T: Default + Clone, // Default used; Clone not required here (left in to test multi-bound pruning)
// {
//     x.unwrap_or_default()
// }

// /// HRTB bound; **used** in body.
// pub fn hrtb_used<F>(f: F) -> usize
// where
//     for<'a> F: Fn(&'a str) -> usize,
// {
//     f("hello")
// }

// /// HRTB bound; **unused** in body.
// pub fn hrtb_unused<F>()
// where
//     for<'a> F: Fn(&'a str) -> usize, // not used
// {
//     // no-op
// }
