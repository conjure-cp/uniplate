//! A Tagged Zipper is a wrapper around a Zipper that allows for attaching arbitrary data to each node.
//!
//! See the [`Zipper`] type for more information on how to use this type.

use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use crate::{zipper::Zipper, Uniplate};

struct TagNode<D> {
    data: D,
    parent: Option<Rc<RefCell<TagNode<D>>>>,
    children: Vec<Rc<RefCell<TagNode<D>>>>,
}

/// A cursor into Uniplate types that allows for attaching arbitrary data to each node.
///
/// This is an extension of the [`Zipper`] type where each node in the tree has a unique
/// persistent 'tag' associated with it, accessible via the `tag` and `tag_mut` methods.
///
/// ## Lazy Construction & Invalidation
///
/// Tags are lazily constructed using the constructor function provided on initialisation
/// of the zipper.
/// The constructor is only called as necessary to ensure the current focus has a valid tag,
/// including when the zipper is first created and when the focus is replaced.
///
/// Tags for the whole subtree are invalidated and removed when the focus is replaced, since
/// parts of the subtree may have changed.
///
/// Similarly, mutable access to the inner tree is not provided, as it could lead to inconsistencies
/// between the tree and associated tags.
/// Instead, use the methods provided by this type to mutate the zipper.
///
/// # Example
/// In this example, tags are used to cache the height of a binary tree.
/// ```rust
/// use uniplate::{Uniplate, tagged_zipper::TaggedZipper};
///
/// /// A simple binary tree.
/// #[derive(Uniplate, Debug, Clone, PartialEq, Eq)]
/// enum Tree {
///     Node(Box<Tree>, Box<Tree>),
///     Leaf,
/// }
///
/// impl Tree {
///     /// Returns the height of the binary tree.
///     fn height(&self) -> usize {
///         match self {
///             Tree::Node(left, right) => 1 + left.height().max(right.height()),
///             Tree::Leaf => 0,
///         }
///     }
/// }
///
/// // Create a tree like so:
/// //       o
/// //      / \
/// //     o   o
/// //    / \
/// //   o   o
///
/// let tree = Tree::Node(
///     Box::new(Tree::Node(Box::new(Tree::Leaf), Box::new(Tree::Leaf))),
///     Box::new(Tree::Leaf),
/// );
///
/// // Create a TaggedZipper that lazily calculates the height of each node.
/// let mut zipper = TaggedZipper::new(tree, Tree::height);
///
/// assert_eq!(*zipper.tag(), 2); // Height of the root
///
/// // Replace the left subtree with a leaf
/// zipper.go_down().unwrap();
/// zipper.replace_focus(Tree::Leaf);
/// assert_eq!(*zipper.tag(), 0);
///
/// // The tree is now:
/// //     o
/// //    / \
/// //   o   o
///
/// zipper.go_up().unwrap(); // Move back to the root
/// zipper.reset_tag(); // Re-calculate the tree height
/// assert_eq!(*zipper.tag(), 1); // New height of the root
/// ```
#[derive(Clone)]
pub struct TaggedZipper<T, D, F>
where
    T: Uniplate,
    D: Clone,
    F: FnMut(&T) -> D,
{
    zipper: Zipper<T>,
    tag_node: Rc<RefCell<TagNode<D>>>,
    construct_tag: F,
}

impl<T, D, F> TaggedZipper<T, D, F>
where
    T: Uniplate,
    D: Clone,
    F: FnMut(&T) -> D,
{
    /// Creates a new `TaggedZipper` with the given root and tag constructor.
    ///
    /// The focus is initially set to the root of the tree.
    pub fn new(root: T, mut constructor: F) -> Self {
        let tag_node = TagNode {
            data: constructor(&root),
            parent: None,
            children: Vec::new(),
        };

        TaggedZipper {
            tag_node: Rc::new(RefCell::new(tag_node)),
            construct_tag: constructor,
            zipper: Zipper::new(root),
        }
    }

    /// Returns an immutable borrow to the underlying zipper.
    ///
    /// Mutable access to the inner tree is not provided, as it could break the tag structure.
    pub fn zipper(&self) -> &Zipper<T> {
        &self.zipper
    }

    /// Borrows the current focus.
    ///
    /// Mutable access to the inner tree is not provided, as it could break the tag structure.
    pub fn focus(&self) -> &T {
        self.zipper.focus()
    }

    /// Replaces the focus of the [Zipper], returning the old focus.
    ///
    /// This will also invalidate the tags for the current focus and all its descendants,
    /// as the subtree structure may have changed.
    pub fn replace_focus(&mut self, new_focus: T) -> T {
        let old_focus = self.zipper.replace_focus(new_focus);

        // Tags for the old subtree are now invalid
        self.invalidate_subtree();

        old_focus
    }

    /// Rebuilds the root node, consuming the [`Zipper`].
    pub fn rebuild_root(self) -> T {
        // Don't care about tag structure as the zipper is consumed
        self.zipper.rebuild_root()
    }

    /// Borrows the tag of the current focus.
    pub fn tag(&self) -> Ref<D> {
        Ref::map(self.tag_node.borrow(), |node| &node.data)
    }

    /// Mutably borrows the tag of the current focus.
    pub fn tag_mut(&mut self) -> RefMut<D> {
        RefMut::map(self.tag_node.borrow_mut(), |node| &mut node.data)
    }

    /// Replaces the tag of the current focus, returning the old tag.
    pub fn replace_tag(&mut self, new_tag: D) -> D {
        std::mem::replace(&mut self.tag_node.borrow_mut().data, new_tag)
    }

    /// Resets the tag of the current focus to the value returned by the constructor,
    /// returning the old tag.
    pub fn reset_tag(&mut self) -> D {
        let new_tag = (self.construct_tag)(self.zipper.focus());
        self.replace_tag(new_tag)
    }

    /// Removes the tags associated with the current focus and all its descendants.
    ///
    /// Any changes made to descendants' tags will be lost, and the constructor will need
    /// to be called again while exploring the subtree.
    pub fn invalidate_subtree(&mut self) {
        let parent_node = self.tag_node.borrow().parent.clone();
        let new_tag = Rc::new(RefCell::new(TagNode {
            data: (self.construct_tag)(self.zipper.focus()),
            parent: parent_node,
            children: Vec::new(),
        }));
        let _ = std::mem::replace(&mut self.tag_node, new_tag);
    }

    /// Sets the focus to the parent of the focus (if it exists).
    pub fn go_up(&mut self) -> Option<()> {
        self.zipper.go_up()?;

        // Can unwrap safely because we know there is a parent node
        let parent_tag = self.tag_node.borrow().parent.clone().unwrap();
        self.tag_node = parent_tag;

        Some(())
    }

    /// Sets the focus to the left-most child of the focus (if it exists).
    pub fn go_down(&mut self) -> Option<()> {
        self.zipper.go_down()?;

        // Move to or create the first child tag node
        let child_tag = self.tag_node.borrow().children.first().cloned();
        self.tag_node = match child_tag {
            Some(tag) => tag.clone(),
            None => {
                let new_tag = Rc::new(RefCell::new(TagNode {
                    data: (self.construct_tag)(self.zipper.focus()),
                    parent: Some(self.tag_node.clone()),
                    children: Vec::new(),
                }));

                self.tag_node.borrow_mut().children.push(new_tag.clone());
                new_tag
            }
        };

        Some(())
    }

    /// Sets the focus to the left sibling of the focus (if it exists).
    pub fn go_left(&mut self) -> Option<()> {
        self.zipper.go_left()?;

        // Left sibling tags always exist, as go_down goes to the left-most child.
        let parent_node = self.tag_node.borrow().parent.clone().unwrap();

        let siblings_i = self.zipper.siblings_index().unwrap();
        self.tag_node = parent_node.borrow().children[siblings_i].clone();

        Some(())
    }

    /// Sets the focus to the right sibling of the focus (if it exists).
    pub fn go_right(&mut self) -> Option<()> {
        self.zipper.go_right()?;

        let parent_tag_node = self.tag_node.borrow().parent.clone().unwrap();

        let sibling_idx = self.zipper.siblings_index().unwrap();
        let sibling_tag_node = parent_tag_node.borrow().children.get(sibling_idx).cloned();

        self.tag_node = match sibling_tag_node {
            Some(tag) => tag.clone(),
            None => {
                let new_tag = Rc::new(RefCell::new(TagNode {
                    data: (self.construct_tag)(self.zipper.focus()),
                    parent: Some(parent_tag_node.clone()),
                    children: Vec::new(),
                }));

                parent_tag_node.borrow_mut().children.push(new_tag.clone());
                new_tag
            }
        };

        Some(())
    }
}
