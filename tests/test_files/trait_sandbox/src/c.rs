//! Traits with supertraits and `where Self: ...` clauses.

use crate::traits::{Sub, Super, Thing};

/// Function requiring a `Sub` (which implies `Super: Debug`); **used** (via `Debug` formatting).
pub fn uses_super_via_sub<T: Sub>(t: &T) -> String {
    format!("{:?}", t)
}

/// Function with **unused** supertrait in signature.
/// We require `Super` but never debug-print; should be removable if not implied elsewhere.
pub fn super_unused<T: Super>(_t: &T) -> usize {
    42
}

/// Returns a Thing to help link trait usage across modules.
pub fn make_thing() -> Thing {
    Thing { n: 0 }
}
