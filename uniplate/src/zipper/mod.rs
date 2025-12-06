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
