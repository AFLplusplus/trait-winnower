//! Impl blocks: bounds on the impl vs. on methods.

/// Simple wrapper to exercise generic method bounds.
pub struct Wrapper<T>(pub T);

impl<T> Wrapper<T> {
    /// Method-level bound; **used** (copying from `&self` requires `T: Copy`).
    pub fn copied(&self) -> T
    where
        T: Copy,
    {
        self.0
    }

    /// Method-level `where` bound; **unused** in body.
    pub fn id(&self)
    where
        T: Ord, // not used
    {
        let _ = &self.0; // no Ord usage
    }
}

/// Impl-level bound; **used** in method body (`T::default()`).
impl<T> Wrapper<T>
where
    T: Default,
{
    pub fn new_default() -> Self {
        Self(T::default())
    }
}
