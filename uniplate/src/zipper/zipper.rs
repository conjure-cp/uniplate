//! Simple cursors into Uniplate types.
//!
//! See the [module-level documentation](crate::zipper) for more.

use std::{collections::VecDeque, sync::Arc};

use super::Zipper;
use crate::{Biplate, Tree, Uniplate};

/// A Zipper over `Uniplate` types.
///
/// See the module-level documentation.
#[derive(Clone)]
pub struct SimpleZipper<T: Uniplate> {
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

impl<T: Uniplate> SimpleZipper<T> {
    /// Creates a new [`SimpleZipper`] with `root` as the root node.
    ///
    /// The focus is initially the root node.
    pub fn new(root: T) -> Self {
        SimpleZipper {
            focus: root,
            path: Vec::new(),
        }
    }

    /// Mutably borrows the current focus.
    pub fn focus_mut(&mut self) -> &mut T {
        &mut self.focus
    }

    /// Returns the depth of the focus from the root.
    pub fn depth(&self) -> usize {
        self.path.len()
    }

    /// Returns the index of the focus among its siblings from left to right.
    ///
    /// Returns `None` if the focus is the root node.
    pub(crate) fn siblings_index(&self) -> Option<usize> {
        let path_segment = self.path.last()?;
        Some(path_segment.left.len())
    }

    /// Returns an iterator over the left siblings of the focus, in left-right order.
    pub fn iter_left_siblings(&self) -> impl Iterator<Item = &T> {
        self.path
            .last()
            .map(|seg| seg.left.iter())
            .into_iter()
            .flatten()
    }

    /// Returns an iterator over the right siblings of the focus, in left-right order.
    pub fn iter_right_siblings(&self) -> impl Iterator<Item = &T> {
        self.path
            .last()
            .map(|seg| seg.right.iter())
            .into_iter()
            .flatten()
    }

    /// Returns an iterator over all nodes with the same parent as the focus, in left-right order.
    ///
    /// The iterator will yield left siblings first, then the focus, then right siblings.
    pub fn iter_siblings(&self) -> impl Iterator<Item = &T> {
        self.iter_left_siblings()
            .chain(std::iter::once(self.focus()))
            .chain(self.iter_right_siblings())
    }

    /// Returns an iterator over all ancestors of the focus, going upwards (parent to root).
    ///
    /// This is an expensive operation as it rebuilds each ancestor in sequence.
    /// The returned ancestors will reflect any changes made so far during traversal in their subtrees.
    pub fn iter_ancestors<'a>(&'a self) -> impl Iterator<Item = T> + 'a {
        AncestorsIter {
            zipper: self.clone(),
        }
    }
}

impl<T> Zipper<T> for SimpleZipper<T>
where
    T: Uniplate,
{
    fn focus(&self) -> &T {
        &self.focus
    }

    fn replace_focus(&mut self, new_focus: T) -> T {
        std::mem::replace(&mut self.focus, new_focus)
    }

    fn rebuild_root(mut self) -> T {
        while self.go_up().is_some() {}
        self.focus
    }

    fn go_up(&mut self) -> Option<()> {
        let mut path_seg = self.path.pop()?;

        // TODO: get rid of the clone if possible
        path_seg.left.push_back(self.focus.clone());
        path_seg.left.append(&mut path_seg.right);

        let tree = (path_seg.rebuild_tree)(path_seg.left);

        self.focus = (path_seg.ctx)(tree);

        Some(())
    }

    fn has_up(&self) -> bool {
        !self.path.is_empty()
    }

    fn go_down(&mut self) -> Option<()> {
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

    fn has_down(&self) -> bool {
        let (children, _) = self.focus.uniplate();
        !children.is_empty()
    }

    fn go_left(&mut self) -> Option<()> {
        let path_segment = self.path.last_mut()?;
        let new_focus = path_segment.left.pop_front()?;
        let old_focus = std::mem::replace(&mut self.focus, new_focus);
        path_segment.right.push_back(old_focus);
        Some(())
    }

    fn has_left(&self) -> bool {
        let Some(path_segment) = self.path.last() else {
            return false;
        };
        !path_segment.left.is_empty()
    }

    fn go_right(&mut self) -> Option<()> {
        let path_segment = self.path.last_mut()?;
        let new_focus = path_segment.right.pop_front()?;
        let old_focus = std::mem::replace(&mut self.focus, new_focus);
        path_segment.left.push_back(old_focus);
        Some(())
    }

    fn has_right(&self) -> bool {
        let Some(path_segment) = self.path.last() else {
            return false;
        };
        !path_segment.right.is_empty()
    }
}

struct AncestorsIter<T: Uniplate> {
    zipper: SimpleZipper<T>,
}

impl<T: Uniplate> Iterator for AncestorsIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.zipper.go_up().map(|_| self.zipper.focus.clone())
    }
}

/// A Zipper over `Biplate` types.
///
/// A `SimpleZipperBi<To,From>` traverses through all values of type `To` in a value of type `From`.
///
/// Unlike [`SimpleZipper`], the root node can never be focused on (as it is not of type `To`). Instead,
/// the initial node is the left-most child.
///
/// See the module-level documentation.
#[derive(Clone)]
pub struct SimpleZipperBi<To: Uniplate, From: Biplate<To>> {
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

impl<To: Uniplate, From: Biplate<To>> SimpleZipperBi<To, From> {
    /// Creates a new [`SimpleZipperBi`] with `root` as the root node.
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

        Some(SimpleZipperBi {
            focus,
            path: vec![segment],
        })
    }

    /// Borrows the current focus.
    pub fn focus(&self) -> &To {
        &self.focus
    }

    /// Mutably borrows the current focus.
    pub fn focus_mut(&mut self) -> &mut To {
        &mut self.focus
    }

    /// Replaces the focus, returning the old focus.
    pub fn replace_focus(&mut self, new_focus: To) -> To {
        std::mem::replace(&mut self.focus, new_focus)
    }

    /// Rebuilds the root node, consuming the [`SimpleZipperBi`]
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
    /// To get the topmost node (of type `From`), use [`rebuild_root`](SimpleZipperBi::rebuild_root).
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
