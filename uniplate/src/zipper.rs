//! Cursors into Uniplate types.
//!
//! A Zippers is a cursor into a functional data structure. The cursor can be moved around the data
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

use std::{collections::VecDeque, sync::Arc};

use crate::{Biplate, Tree, Uniplate};

/// A Zipper over `Uniplate` types.
///
/// See the module-level documentation.
#[derive(Clone)]
pub struct Zipper<T: Uniplate> {
    /// The current node
    focus: T,

    /// The path back to the top, immediate parent last.
    ///
    /// If empty, the focus is the top level node.
    path: Vec<PathSegment<T>>,
}

#[derive(Clone)]
struct PathSegment<T: Uniplate> {
    /// Left siblings of the node
    left: VecDeque<T>,

    /// Right siblings of the node
    right: VecDeque<T>,

    // A possibility was to store parent and use with_children instead of doing it by hand this
    // way. However, that would store an old copy of the subtree we are currently in, which is
    // unnecessary.
    //
    /// Function to convert this layer back into a tree given a full list of children
    rebuild_tree: Arc<dyn Fn(VecDeque<T>) -> Tree<T>>,

    // Function to create the parent node, given its children as a tree
    ctx: Arc<dyn Fn(Tree<T>) -> T>,
}

impl<T: Uniplate> Zipper<T> {
    /// Creates a new [`Zipper`] with `root` as the root node.
    ///
    /// The focus is initially the root node.
    pub fn new(root: T) -> Self {
        Zipper {
            focus: root,
            path: Vec::new(),
        }
    }

    /// Borrows the current focus.
    pub fn focus(&self) -> &T {
        &self.focus
    }

    /// Mutably borrows the current focus.
    pub fn focus_mut(&mut self) -> &T {
        &mut self.focus
    }

    /// Replaces the focus of the [Zipper], returning the old focus.
    pub fn replace_focus(&mut self, new_focus: T) -> T {
        std::mem::replace(&mut self.focus, new_focus)
    }

    /// Rebuilds the root node, consuming the [`Zipper`].
    pub fn rebuild_root(mut self) -> T {
        while self.go_up().is_some() {}
        self.focus
    }

    /// Returns the depth of the focus from the root.
    pub fn depth(&self) -> usize {
        self.path.len()
    }

    /// Sets the focus to the parent of the focus (if it exists).
    pub fn go_up(&mut self) -> Option<()> {
        let mut path_seg = self.path.pop()?;

        // TODO: get rid of the clone if possible
        path_seg.left.push_back(self.focus.clone());
        path_seg.left.append(&mut path_seg.right);

        let tree = (path_seg.rebuild_tree)(path_seg.left);

        self.focus = (path_seg.ctx)(tree);

        Some(())
    }

    /// Sets the focus to the left-most child of the focus (if it exists).
    pub fn go_down(&mut self) -> Option<()> {
        let (children, ctx) = self.focus.uniplate();
        let (mut siblings, rebuild_tree) = children.list();
        let new_focus = siblings.pop_front()?;
        let new_segment = PathSegment {
            left: VecDeque::new(),
            right: siblings,
            rebuild_tree: rebuild_tree.into(),
            ctx: ctx.into(),
        };

        self.path.push(new_segment);
        self.focus = new_focus;
        Some(())
    }

    /// Sets the focus to the left sibling of the focus (if it exists).
    pub fn go_left(&mut self) -> Option<()> {
        let path_segment = self.path.last_mut()?;
        let new_focus = path_segment.left.pop_front()?;
        let old_focus = std::mem::replace(&mut self.focus, new_focus);
        path_segment.right.push_back(old_focus);
        Some(())
    }

    /// Sets the focus to the right sibling of the focus (if it exists).
    pub fn go_right(&mut self) -> Option<()> {
        let path_segment = self.path.last_mut()?;
        let new_focus = path_segment.right.pop_front()?;
        let old_focus = std::mem::replace(&mut self.focus, new_focus);
        path_segment.left.push_back(old_focus);
        Some(())
    }
}

/// A Zipper over `Biplate` types.
///
/// A `ZipperBi<To,From>` traverses through all values of type `To` in a value of type `From`.
///
/// Unlike [`Zipper`], the root node can never be focused on (as it is not of type `To`). Instead,
/// the initial node is the left-most child.
///
/// See the module-level documentation.
#[derive(Clone)]
pub struct ZipperBi<To: Uniplate, From: Biplate<To>> {
    /// The current node
    focus: To,

    /// The path back to the top, immediate parent last.
    ///
    /// If empty, the focus is the top level node.
    path: Vec<PathSegmentBi<To, From>>,
}

#[derive(Clone)]
enum PathSegmentBi<To: Uniplate, From: Biplate<To>> {
    /// The parent node is of type U, so need some slightly different types
    Top {
        /// Left siblings of the node
        left: VecDeque<To>,

        /// Right siblings of the node
        right: VecDeque<To>,

        /// Function to convert this layer back into a tree given a full list of children
        rebuild_tree: Arc<dyn Fn(VecDeque<To>) -> Tree<To>>,

        // Function to create the parent node, given its children as a tree
        ctx: Arc<dyn Fn(Tree<To>) -> From>,
    },

    /// After the first level of the tree (where we call biplate), we use uniplate to traverse the
    /// tree deeper.
    ///
    /// Same as [`PathSegment`]
    Node {
        /// Left siblings of the node
        left: VecDeque<To>,

        /// Right siblings of the node
        right: VecDeque<To>,

        /// Function to convert this layer back into a tree given a full list of children
        rebuild_tree: Arc<dyn Fn(VecDeque<To>) -> Tree<To>>,

        // Function to create the parent node, given its children as a tree
        ctx: Arc<dyn Fn(Tree<To>) -> To>,
    },
}

impl<To: Uniplate, From: Biplate<To>> ZipperBi<To, From> {
    /// Creates a new [`ZipperBi`] with `root` as the root node.
    ///
    /// The focus is set to the left-most child of `root`.
    ///
    /// Returns `None` if the root node has no children of type `To`.
    pub fn new(top: From) -> Option<Self> {
        // we can never focus on the top level node, just its immediate children.

        let (children, ctx) = top.biplate();
        let (mut siblings, rebuild_tree) = children.list();
        let focus = siblings.pop_front()?;
        let segment = PathSegmentBi::Top {
            left: VecDeque::new(),
            right: siblings,
            rebuild_tree: rebuild_tree.into(),
            ctx: ctx.into(),
        };

        Some(ZipperBi {
            focus,
            path: vec![segment],
        })
    }

    /// Borrows the current focus.
    pub fn focus(&self) -> &To {
        &self.focus
    }

    /// Mutably borrows the current focus.
    pub fn focus_mut(&mut self) -> &To {
        &mut self.focus
    }

    /// Replaces the focus, returning the old focus.
    pub fn replace_focus(&mut self, new_focus: To) -> To {
        std::mem::replace(&mut self.focus, new_focus)
    }

    /// Rebuilds the root node, consuming the [`ZipperBi`]
    pub fn rebuild_root(mut self) -> From {
        while self.go_up().is_some() {}

        let Some(PathSegmentBi::Top {
            mut left,
            mut right,
            rebuild_tree,
            ctx,
        }) = self.path.pop()
        else {
            // go_up should leave us with a single PathSegmentBi::Top in the path
            unreachable!();
        };

        left.push_back(self.focus.clone());
        left.append(&mut right);

        let tree = (rebuild_tree)(left);

        (ctx)(tree)
    }

    /// Returns the depth of the focus from the root.
    pub fn depth(&self) -> usize {
        self.path.len()
    }

    /// Sets the focus to the parent of the focus, if it exists and is of type `To.
    ///
    /// To get the topmost node (of type `From`), use [`rebuild_root`](ZipperBi::rebuild_root).
    pub fn go_up(&mut self) -> Option<()> {
        let Some(PathSegmentBi::Node {
            left: _,
            right: _,
            rebuild_tree: _,
            ctx: _,
        }) = self.path.last()
        else {
            return None;
        };

        // the above ensures that we do not commit to the pop unless the let passes
        let Some(PathSegmentBi::Node {
            mut left,
            mut right,
            rebuild_tree,
            ctx,
        }) = self.path.pop()
        else {
            unreachable!();
        };

        // TODO: get rid of the clone if possible
        left.push_back(self.focus.clone());
        left.append(&mut right);

        let tree = (rebuild_tree)(left);

        self.focus = (ctx)(tree);

        Some(())
    }

    /// Sets the focus to the left-most child of the focus (if it exists).
    pub fn go_down(&mut self) -> Option<()> {
        let (children, ctx) = self.focus.uniplate();
        let (mut siblings, rebuild_tree) = children.list();
        let new_focus = siblings.pop_front()?;
        let new_segment = PathSegmentBi::Node {
            left: VecDeque::new(),
            right: siblings,
            rebuild_tree: rebuild_tree.into(),
            ctx: ctx.into(),
        };

        self.path.push(new_segment);
        self.focus = new_focus;
        Some(())
    }

    /// Sets the focus to the left sibling of the focus (if it exists).
    pub fn go_left(&mut self) -> Option<()> {
        let (left, right) = match self.path.last_mut()? {
            PathSegmentBi::Top {
                left,
                right,
                rebuild_tree: _,
                ctx: _,
            } => (left, right),
            PathSegmentBi::Node {
                left,
                right,
                rebuild_tree: _,
                ctx: _,
            } => (left, right),
        };
        let new_focus = left.pop_front()?;
        let old_focus = std::mem::replace(&mut self.focus, new_focus);
        right.push_back(old_focus);
        Some(())
    }

    /// Sets the focus to the right sibling of the focus (if it exists).
    pub fn go_right(&mut self) -> Option<()> {
        let (left, right) = match self.path.last_mut()? {
            PathSegmentBi::Top {
                left,
                right,
                rebuild_tree: _,
                ctx: _,
            } => (left, right),
            PathSegmentBi::Node {
                left,
                right,
                rebuild_tree: _,
                ctx: _,
            } => (left, right),
        };
        let new_focus = right.pop_front()?;
        let old_focus = std::mem::replace(&mut self.focus, new_focus);
        left.push_back(old_focus);
        Some(())
    }
}
