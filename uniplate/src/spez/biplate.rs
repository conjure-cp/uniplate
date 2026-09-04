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

    #[allow(missing_docs)]
    fn spez_children_bi_count(&self) -> usize;

    #[allow(missing_docs)]
    fn spez_try_replace_child_at_bi(&self, index: usize, child: Self::Dest) -> bool;
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

    #[allow(missing_docs)]
    fn spez_children_bi_count(&self) -> usize;

    #[allow(missing_docs)]
    fn spez_try_replace_child_at_bi(&self, index: usize, child: Self::Dest) -> bool;
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

    fn spez_children_bi_count(&self) -> usize {
        self.0.children_bi_count()
    }

    fn spez_try_replace_child_at_bi(&self, index: usize, child: Self::Dest) -> bool {
        // Owned wrapper: mutation is not supported through `SpezBiplate`.
        let _ = (index, child);
        false
    }
}

impl<Src, Dest> BiplateYes for &SpezBiplateMut<Src, Dest>
where
    Src: Biplate<Dest>,
    Dest: Eq + Clone + Uniplate,
{
    type Src = Src;
    type Dest = Dest;

    fn spez_try_biplate(&self) -> (Tree<Self::Dest>, Box<dyn Fn(Tree<Self::Dest>) -> Self::Src>) {
        // SAFETY: SpezBiplateMut is only constructed from a live &mut Src.
        unsafe { (*self.src).biplate() }
    }

    #[inline(always)]
    fn spez_impls_biplate(&self) -> bool {
        true
    }

    fn spez_children_bi_count(&self) -> usize {
        // SAFETY: SpezBiplateMut is only constructed from a live &mut Src.
        unsafe { (*self.src).children_bi_count() }
    }

    fn spez_try_replace_child_at_bi(&self, index: usize, child: Self::Dest) -> bool {
        // SAFETY: SpezBiplateMut is only constructed from a live &mut Src.
        unsafe { (*self.src).try_replace_child_at_bi(index, child) }
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

    fn spez_children_bi_count(&self) -> usize {
        if TypeId::of::<Src>() == TypeId::of::<Dest>() {
            1
        } else {
            0
        }
    }

    fn spez_try_replace_child_at_bi(&self, index: usize, child: Self::Dest) -> bool {
        let _ = (index, child);
        false
    }
}

impl<Src, Dest> BiplateNo for SpezBiplateMut<Src, Dest>
where
    Src: Eq + Clone + 'static,
    Dest: Eq + Clone + Uniplate + 'static,
{
    type Src = Src;
    type Dest = Dest;

    fn spez_try_biplate(&self) -> (Tree<Self::Dest>, Box<dyn Fn(Tree<Self::Dest>) -> Self::Src>) {
        // SAFETY: SpezBiplateMut is only constructed from a live &mut Src.
        let this = unsafe { (*self.src).clone() };
        if TypeId::of::<Src>() == TypeId::of::<Dest>() {
            unsafe {
                let this_as_dest: Dest = std::mem::transmute::<&Src, &Dest>(&this).clone();
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
            (Tree::Zero, Box::new(move |_| this.clone()))
        }
    }

    #[inline(always)]
    fn spez_impls_biplate(&self) -> bool {
        false
    }

    fn spez_children_bi_count(&self) -> usize {
        if TypeId::of::<Src>() == TypeId::of::<Dest>() {
            1
        } else {
            0
        }
    }

    fn spez_try_replace_child_at_bi(&self, index: usize, child: Self::Dest) -> bool {
        if TypeId::of::<Src>() == TypeId::of::<Dest>() {
            if index != 0 {
                return false;
            }
            // SAFETY: TypeId equality + SpezBiplateMut pointer invariant.
            unsafe {
                let child_as_src = std::mem::transmute_copy::<Dest, Src>(&child);
                std::mem::forget(child);
                *self.src = child_as_src;
            }
            true
        } else {
            false
        }
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

use super::{SpezBiplate, SpezBiplateMut};

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

/// Spez-aware [`Biplate::children_bi_count`].
#[macro_export]
#[doc(hidden)]
macro_rules! try_biplate_children_bi_count {
    ($x:expr, $t:ty) => {{
        #[allow(unused_imports)]
        use ::uniplate::spez::{BiplateNo, BiplateYes, SpezBiplateMut};
        let spez = ::uniplate::spez::SpezBiplateMut::<_, $t>::new($x);
        (&&spez).spez_children_bi_count()
    }};
}

/// Spez-aware [`Biplate::try_replace_child_at_bi`].
#[macro_export]
#[doc(hidden)]
macro_rules! try_biplate_replace_child_at {
    ($x:expr, $t:ty, $index:expr, $child:expr) => {{
        #[allow(unused_imports)]
        use ::uniplate::spez::{BiplateNo, BiplateYes, SpezBiplateMut};
        let spez = ::uniplate::spez::SpezBiplateMut::<_, $t>::new($x);
        (&&spez).spez_try_replace_child_at_bi($index, $child)
    }};
}
