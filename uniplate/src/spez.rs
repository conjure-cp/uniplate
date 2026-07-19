//! Auto-deref specialization helpers for the derive macro.
#![allow(clippy::type_complexity)]

mod biplate;
pub use biplate::*;

mod uniplate;
pub use uniplate::*;

use std::marker::PhantomData;

/// A wrapper type used for auto-deref specialisation of `Biplate`.
///
/// The `Dest` type stores the destination type of the `Biplate` operation. That is,
/// `SpezBiplate<Src,Dest>` trys to call the implementation `Biplate<Dest> for Src`.
///
/// This is used alongside the [`BiplateYes`] and [`BiplateNo`] traits to do specialization of
/// Biplate calls in the derive macro.
pub struct SpezBiplate<Src, Dest>(pub Src, pub PhantomData<Dest>);

/// Mutable specialisation wrapper for [`Biplate::try_replace_child_at_bi`] / `children_bi_count`.
///
/// Stores a raw pointer so autoref specialisation can still run through shared references while
/// mutating the source. Only construct this from a live `&mut Src` and do not use it after that
/// borrow ends.
pub struct SpezBiplateMut<Src, Dest> {
    /// Raw pointer to the value being specialised over. See [`SpezBiplateMut::new`].
    pub src: *mut Src,
    _pd: PhantomData<Dest>,
}

impl<Src, Dest> SpezBiplateMut<Src, Dest> {
    /// Creates a mutable specialisation wrapper from `src`.
    ///
    /// The returned wrapper must only be used while `src` is still exclusively borrowed.
    #[inline]
    pub fn new(src: &mut Src) -> Self {
        Self {
            src: src as *mut Src,
            _pd: PhantomData,
        }
    }
}

/// A wrapper type used for auto-deref specialisation of `Uniplate`.
pub struct SpezUniplate<Src>(pub Src);
