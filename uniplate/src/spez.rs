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

/// A wrapper type used for auto-deref specialisation of `Uniplate`.
pub struct SpezUniplate<Src>(pub Src);
