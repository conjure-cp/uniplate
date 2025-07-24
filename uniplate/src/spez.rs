//! Auto-deref specialization helpers for the derive macro.
#![allow(clippy::type_complexity)]

use std::{any::TypeId, marker::PhantomData};

use crate::{Biplate, Tree, Uniplate};

/// A wrapper type used for auto-deref specialisation of `Biplate`.
///
/// The `Dest` type stores the destination type of the `Biplate` operation. That is,
/// `SpezBiplate<Src,Dest>` trys to call the implementation `Biplate<Dest> for Src`.
///
/// This is used alongside the [`BiplateYes`] and [`BiplateNo`] traits to do specialization of
/// Biplate calls in the derive macro.
pub struct SpezBiplate<Src, Dest>(pub Src, pub PhantomData<Dest>);

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
pub use super::try_biplate_to;

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
pub use super::impls_biplate_to;

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

/// A wrapper type used for auto-deref specialisation of `Uniplate`.
pub struct SpezUniplate<Src>(pub Src);

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
pub use super::try_uniplate;

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
#[doc(inline)]
/// Returns whether `x` implements `Uniplate`.
pub use super::impls_uniplate;

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
