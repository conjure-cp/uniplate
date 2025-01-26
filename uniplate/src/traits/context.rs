//! The underlying iterator for `Uniplate::context()`

use std::{collections::VecDeque, sync::Arc};

use super::{Biplate, Tree, Uniplate};

/// A simple Zipper [1] used in [`ContextIter`].
///
///
/// [1]: https://people.mpi-sws.org/~skilpat/plerg/papers/huet-zipper-2up.pdf
#[derive(Clone)]
pub(super) struct Zipper<T: Uniplate> {
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
    fn new(top: T) -> Zipper<T> {
        Zipper {
            focus: top,
            path: Vec::new(),
        }
    }
    /// Traverses to the parent of the current node if it exists
    #[inline]
    fn go_up(&mut self) -> Option<()> {
        let mut path_seg = self.path.pop()?;

        // TODO: get rid of the clone if possible
        path_seg.left.push_back(self.focus.clone());
        path_seg.left.append(&mut path_seg.right);

        let tree = (path_seg.rebuild_tree)(path_seg.left);

        self.focus = (path_seg.ctx)(tree);

        Some(())
    }

    /// Traverses to the left-most child of the current node if it exists
    #[inline]
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

    /// Traverses to the left sibling of the current node if it exists
    #[inline]
    #[allow(dead_code)]
    fn go_left(&mut self) -> Option<()> {
        let path_segment = self.path.last_mut()?;
        let new_focus = path_segment.left.pop_front()?;
        let old_focus = std::mem::replace(&mut self.focus, new_focus);
        path_segment.right.push_back(old_focus);
        Some(())
    }

    /// Traverses to the right sibling of the current node if it exists
    #[inline]
    fn go_right(&mut self) -> Option<()> {
        let path_segment = self.path.last_mut()?;
        let new_focus = path_segment.right.pop_front()?;
        let old_focus = std::mem::replace(&mut self.focus, new_focus);
        path_segment.left.push_back(old_focus);
        Some(())
    }

    #[inline]
    /// Rebuilds the top level node of the tree.
    fn go_to_top(mut self) -> T {
        while self.go_up().is_some() {}
        self.focus
    }
}

#[derive(Clone)]
pub(super) struct ZipperBi<T: Uniplate, U: Biplate<T>> {
    /// The current node
    focus: T,

    /// The path back to the top, immediate parent last.
    path: Vec<PathSegmentBi<T, U>>,
}

#[derive(Clone)]

enum PathSegmentBi<T: Uniplate, U: Biplate<T>> {
    /// The parent node is of type U, so need some slightly different types
    Top {
        /// Left siblings of the node
        left: VecDeque<T>,

        /// Right siblings of the node
        right: VecDeque<T>,

        /// Function to convert this layer back into a tree given a full list of children
        rebuild_tree: Arc<dyn Fn(VecDeque<T>) -> Tree<T>>,

        // Function to create the parent node, given its children as a tree
        ctx: Arc<dyn Fn(Tree<T>) -> U>,
    },

    /// After the first level of the tree (where we call biplate), we use uniplate to traverse the
    /// tree deeper.
    ///
    /// Same as [`PathSegment`]
    Node {
        /// Left siblings of the node
        left: VecDeque<T>,

        /// Right siblings of the node
        right: VecDeque<T>,

        /// Function to convert this layer back into a tree given a full list of children
        rebuild_tree: Arc<dyn Fn(VecDeque<T>) -> Tree<T>>,

        // Function to create the parent node, given its children as a tree
        ctx: Arc<dyn Fn(Tree<T>) -> T>,
    },
}

impl<T: Uniplate, U: Biplate<T>> ZipperBi<T, U> {
    /// Returns `None` if the biplate returns no children.
    fn new(top: U) -> Option<Self> {
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

    #[inline]
    /// Traverses to the parent of the current node if it exists and is of type `T`
    ///
    /// To get the topmost node (with type `U`), use [`go_to_top`].
    fn go_up(&mut self) -> Option<()> {
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

    /// Traverses to the left-most child of the current node if it exists
    #[inline]
    fn go_down(&mut self) -> Option<()> {
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

    /// Traverses to the left sibling of the current node if it exists
    #[inline]
    #[allow(dead_code)]
    fn go_left(&mut self) -> Option<()> {
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

    /// Traverses to the right sibling of the current node if it exists
    #[inline]
    fn go_right(&mut self) -> Option<()> {
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

    #[inline]
    /// Rebuilds the top level node of the tree.
    fn go_to_top(mut self) -> U {
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
}

/// Iterator for `context`
pub(super) struct ContextIter<T: Uniplate> {
    zipper: Zipper<T>,
    done: bool,
}

impl<T: Uniplate> ContextIter<T> {
    pub(super) fn new(root: T) -> ContextIter<T> {
        ContextIter {
            zipper: Zipper::new(root),
            done: false,
        }
    }
}

impl<T: Uniplate> Iterator for ContextIter<T> {
    type Item = (T, Arc<dyn Fn(T) -> T>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        };
        let node = self.zipper.focus.clone();
        let zipper1 = self.zipper.clone();
        let hole_fn = Arc::new(move |x| {
            // TODO: if performance is still an issue, maybe we could make this a single call function
            // (FnOnce) then we wouldn't need to clone as much?
            let mut zipper1 = zipper1.clone();
            zipper1.focus = x;
            zipper1.go_to_top()
        });

        // prepare iterator for next element.
        // try moving down or right. if we can't move up the tree until we can move right.
        if self.zipper.go_down().is_none() {
            while self.zipper.go_right().is_none() {
                if self.zipper.go_up().is_none() {
                    // at the top again, so this will be the last time we return a node
                    self.done = true;
                    break;
                };
            }
        }

        Some((node, hole_fn))
    }
}

/// Iterator for `context_bi`
pub(super) struct ContextIterBi<T: Uniplate, U: Biplate<T>> {
    zipper: Option<ZipperBi<T, U>>,
    done: bool,
}

impl<T: Uniplate, U: Biplate<T>> ContextIterBi<T, U> {
    pub(super) fn new(root: U) -> ContextIterBi<T, U> {
        ContextIterBi {
            zipper: ZipperBi::new(root),
            done: false,
        }
    }
}

impl<T: Uniplate, U: Biplate<T>> Iterator for ContextIterBi<T, U> {
    type Item = (T, Arc<dyn Fn(T) -> U>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        };

        let Some(zipper) = &mut self.zipper else {
            return None;
        };

        let node = zipper.focus.clone();

        let zipper1 = zipper.clone();
        let hole_fn = Arc::new(move |x| {
            // TODO: if performance is still an issue, maybe we could make this a single call function
            // (FnOnce) then we wouldn't need to clone as much?
            let mut zipper1 = zipper1.clone();
            zipper1.focus = x;
            zipper1.go_to_top()
        });

        // prepare iterator for next element.
        // try moving down or right. if we can't move up the tree until we can move right.
        if zipper.go_down().is_none() {
            while zipper.go_right().is_none() {
                if zipper.go_up().is_none() {
                    // at the top again, so this will be the last time we return a node
                    self.done = true;
                    break;
                };
            }
        }

        Some((node, hole_fn))
    }
}
