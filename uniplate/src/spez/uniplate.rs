//! Specialisation helpers for uniplate on basic types.

/// Specialization proxy for [`uniplate::Uniplate`].
pub trait UniplateYes {
    /// The type to perform the Uniplate operation on.
    type T: Eq + Clone;

    /// Calls `Uniplate` on the inner value.
    ///
    /// This method is called when the wrapped type implements `Uniplate`.
    fn spez_try_uniplate(&self) -> (Tree<Self::T>, Box<dyn Fn(Tree<Self::T>) -> Self::T>);

    #[allow(missing_docs)]
    fn spez_impls_uniplate(&self) -> bool;
}

/// Specialization proxy for [`uniplate::Uniplate`].
pub trait UniplateNo {
    /// The type to perform the Uniplate operation on.
    type T: Eq + Clone;

    /// Fallback implementation used when the inner value doesn't implement `Uniplate`.
    fn spez_try_uniplate(&self) -> (Tree<Self::T>, Box<dyn Fn(Tree<Self::T>) -> Self::T>);

    #[allow(missing_docs)]
    fn spez_impls_uniplate(&self) -> bool;
}

impl<T> UniplateYes for &SpezUniplate<T>
where
    T: Eq + Clone + Uniplate,
{
    type T = T;

    fn spez_try_uniplate(&self) -> (Tree<Self::T>, Box<dyn Fn(Tree<Self::T>) -> Self::T>) {
        self.0.uniplate()
    }

    #[inline(always)]
    fn spez_impls_uniplate(&self) -> bool {
        true
    }
}

impl<T> UniplateNo for SpezUniplate<T>
where
    T: Eq + Clone + 'static,
{
    type T = T;
    fn spez_try_uniplate(&self) -> (Tree<Self::T>, Box<dyn Fn(Tree<Self::T>) -> Self::T>) {
        let self2 = self.0.clone();
        (Tree::Zero, Box::new(move |_| self2.clone()))
    }

    #[inline(always)]
    fn spez_impls_uniplate(&self) -> bool {
        false
    }
}

#[doc(inline)]
/// Tries to call `Uniplate::uniplate` on `$x`, returning a default implementation if `$x` does not
/// implement `Uniplate`.
pub use crate::try_uniplate;

#[macro_export]
#[doc(hidden)]
macro_rules! try_uniplate {
    ($x:expr) => {{
        #[allow(unused_imports)]
        use ::uniplate::spez::{SpezUniplate, UniplateNo, UniplateYes};
        #[allow(clippy::needless_borrow)]
        (&&SpezUniplate($x)).spez_try_biplate()
    }};
}
use crate::Tree;
use crate::Uniplate;
#[doc(inline)]
/// Returns whether `x` implements `Uniplate`.
pub use crate::impls_uniplate;

use super::SpezUniplate;

#[macro_export]
#[doc(hidden)]
macro_rules! impls_uniplate {
    ($x:expr) => {{
        #[allow(unused_imports)]
        use ::uniplate::spez::{SpezUniplate, UniplateNo as _, UniplateYes as _};
        #[allow(clippy::needless_borrow)]
        (&&SpezUniplate($x)).spez_impls_uniplate()
    }};
}
