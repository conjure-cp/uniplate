//! Specialisation helpers for biplate.
use std::any::TypeId;

/// Specialization proxy for [`uniplate::Biplate`].
pub trait BiplateYes {
    /// The source type.
    type Src;
    /// The destination type of the biplate operation.
    type Dest: Eq + Clone;

    /// Calls `Biplate<Dest>` on the inner value.
    ///
    /// This method is called when the wrapped type implements `Biplate<Dest>`.
    fn spez_try_biplate(&self) -> (Tree<Self::Dest>, Box<dyn Fn(Tree<Self::Dest>) -> Self::Src>);

    #[allow(missing_docs)]
    fn spez_impls_biplate(&self) -> bool;
}

/// Specialization proxy for [`uniplate::Biplate`].
pub trait BiplateNo {
    /// The source type.
    type Src;
    /// The destination type of the biplate operation.
    type Dest: Eq + Clone;

    /// Fallback implementation used when the inner value doesn't implement `Biplate<Dest>`.
    fn spez_try_biplate(&self) -> (Tree<Self::Dest>, Box<dyn Fn(Tree<Self::Dest>) -> Self::Src>);

    #[allow(missing_docs)]
    fn spez_impls_biplate(&self) -> bool;
}

// Implementation of specialisation.

impl<Src, Dest> BiplateYes for &SpezBiplate<Src, Dest>
where
    Src: Biplate<Dest>,
    Dest: Eq + Clone + Uniplate,
{
    type Src = Src;
    type Dest = Dest;

    fn spez_try_biplate(&self) -> (Tree<Self::Dest>, Box<dyn Fn(Tree<Self::Dest>) -> Self::Src>) {
        self.0.biplate()
    }

    #[inline(always)]
    fn spez_impls_biplate(&self) -> bool {
        true
    }
}

impl<Src, Dest> BiplateNo for SpezBiplate<Src, Dest>
where
    Src: Eq + Clone + 'static,
    Dest: Eq + Clone + 'static,
{
    type Src = Src;
    type Dest = Dest;

    fn spez_try_biplate(&self) -> (Tree<Self::Dest>, Box<dyn Fn(Tree<Self::Dest>) -> Self::Src>) {
        // Biplate<T> for T returns self, not immediate childreen
        if TypeId::of::<Src>() == TypeId::of::<Dest>() {
            unsafe {
                let this_as_dest: Dest = (std::mem::transmute::<&Src, &Dest>(&self.0)).clone();

                let tree = Tree::One(this_as_dest);
                let ctx = Box::new(move |x| {
                    let Tree::One(x) = x else {
                        panic!();
                    };

                    std::mem::transmute::<&Dest, &Src>(&x).clone()
                });

                (tree, ctx)
            }
        } else {
            let this = self.0.clone();
            (Tree::Zero, Box::new(move |_| this.clone()))
        }
    }

    #[inline(always)]
    fn spez_impls_biplate(&self) -> bool {
        false
    }
}

#[doc(inline)]
/// Tries to call `Biplate<$t>::biplate` on `$x`, returning a default implementation if `$x` does
/// not implement `Biplate<$t>`.
pub use crate::try_biplate_to;

#[macro_export]
#[doc(hidden)]
macro_rules! try_biplate_to {
    ($x:expr,$t:ty) => {{
        #[allow(unused_imports)]
        use ::uniplate::spez::{BiplateNo, BiplateYes, SpezBiplate};
        #[allow(clippy::needless_borrow)]
        (&&SpezBiplate($x, std::marker::PhantomData::<$t>)).spez_try_biplate()
    }};
}

use crate::Biplate;
use crate::Tree;
use crate::Uniplate;
#[doc(inline)]
/// Returns whether `x` implements `Biplate<$t>`.
///
/// ```
/// use uniplate::{spez::impls_biplate_to,Uniplate};
///
/// #[derive(Clone,PartialEq,Eq,Uniplate)]
/// #[biplate(to=String)]
/// enum Expr {
///  A(String)
/// }
///
/// assert!(!impls_biplate_to!(String::from("foo"),i32));
/// assert!(impls_biplate_to!(String::from("foo"),String));
/// assert!(impls_biplate_to!(Expr::A(String::from("foo")),String));
/// ```
pub use crate::impls_biplate_to;

use super::SpezBiplate;

#[macro_export]
#[doc(hidden)]
macro_rules! impls_biplate_to {
    ($x:expr,$t:ty) => {{
        #[allow(unused_imports)]
        use ::uniplate::spez::{BiplateNo as _, BiplateYes as _, SpezBiplate};
        #[allow(clippy::needless_borrow)]
        (&&SpezBiplate($x, std::marker::PhantomData::<$t>)).spez_impls_biplate()
    }};
}
