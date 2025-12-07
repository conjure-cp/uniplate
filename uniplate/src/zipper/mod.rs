//! Cursors into Uniplate types.
//!
//! A zipper is a cursor into a functional data structure. The cursor can be moved around the data
//! structure, and the value at the cursor can be quickly updated.
//!
//! Zippers are particularly useful for mutating self-referential data structures. Updating the
//! value at the cursor is O(1), regardless of its position inside the data structure.
//!
//! For this reason, Zippers should be used instead of [`contexts`](super::Uniplate::contexts) or
//! [`contexts_bi`](super::Biplate::contexts_bi) if you plan to do a lot of mutation during
//! traversal. These functions recreate the root node each time the context function is called,
//! which has a logarithmic complexity.
//!
//! For more information, see:
//!
//!   - [the original paper by Huet](https://www.st.cs.uni-saarland.de/edu/seminare/2005/advanced-fp/docs/huet-zipper.pdf)
//!
//!   - [this explanatory blog post](https://pavpanchekha.com/blog/zippers/huet.html)

mod tagged_zipper;
mod zipper;

pub use tagged_zipper::TaggedZipper;
pub use zipper::{SimpleZipper, SimpleZipperBi};

use crate::Uniplate;

/// A cursor into a tree-like data structure. See the [module-level documentation](crate::zipper) for more.
///
/// Custom types can implement this trait and add useful behaviour on top of simple tree traversal.
/// Some already do: see [`TaggedZipper`].
pub trait Zipper<T>
where
    T: Uniplate,
{
    /// Borrows the current focus.
    fn focus(&self) -> &T;

    /// Replaces the current focus, returning the old focus.
    ///
    /// This operation is usually O(1); see the [module-level documentation](crate::zipper)
    fn replace_focus(&mut self, new_focus: T) -> T;

    /// Rebuilds the root node, consuming the [`Zipper`].
    fn rebuild_root(self) -> T;

    /// Sets the focus to the parent of the current focus (if it exists).
    fn go_up(&mut self) -> Option<()>;

    /// Sets the focus to the left-most child of the current focus (if it exists).
    fn go_down(&mut self) -> Option<()>;

    /// Sets the focus to the left sibling of the current focus (if it exists).
    fn go_left(&mut self) -> Option<()>;

    /// Sets the focus to the right sibling of the current focus (if it exists).
    fn go_right(&mut self) -> Option<()>;

    /// Returns whether the current focus has a parent.
    fn has_up(&self) -> bool;

    /// Returns whether the current focus has children.
    fn has_down(&self) -> bool;

    /// Returns whether the current focus has a left sibling.
    fn has_left(&self) -> bool;

    /// Returns whether the current focus has a right sibling.
    fn has_right(&self) -> bool;
}
