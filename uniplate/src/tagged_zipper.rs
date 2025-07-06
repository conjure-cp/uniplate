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
/// persistent 'tag' associated with it, accessible with `tag` and `tag_mut`.
///
/// Mutable access to the inner tree is not provided, as it could break the tag structure.
/// Instead, use the methods provided by this type to mutate the Zipper.
///
/// # Example
/// ```rust
/// use uniplate::{derive::Uniplate, zipper::Zipper, tagged_zipper::TaggedZipper};
///
/// #[derive(Uniplate, Debug, Clone, PartialEq, Eq)]
/// enum Tree {
///     Leaf,
///     Branch(Vec<Tree>),
/// }
///
/// // A persistent i32 tag is associated with each node
/// let tree = Tree::Branch(vec![Tree::Leaf, Tree::Leaf]);
/// let mut zipper = TaggedZipper::new(tree, |node: &Tree| 0);
///
/// // Update the tag at the root
/// zipper.replace_tag(42);
///
/// // Move down to the first (left-most) child
/// // Since this node hasn't been visited before, the constructor is called
/// zipper.go_down().unwrap();
/// assert_eq!(*zipper.tag(), 0);
///
/// // The tag at the root persists
/// zipper.go_up().unwrap();
/// assert_eq!(*zipper.tag(), 42);
/// ```
pub struct TaggedZipper<T, D, F>
where
    T: Uniplate,
    D: Clone,
    F: Fn(&T) -> D,
{
    zipper: Zipper<T>,
    tag_node: Rc<RefCell<TagNode<D>>>,
    construct_tag: F,
}

impl<T, D, F> TaggedZipper<T, D, F>
where
    T: Uniplate,
    D: Clone,
    F: Fn(&T) -> D,
{
    /// Creates a new `TaggedZipper` with the given root and tag constructor.
    ///
    /// The focus is initially set to the root. The constructor is called as necessary on nodes in the tree.
    pub fn new(root: T, constructor: F) -> Self {
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
    pub fn replace_focus(&mut self, new_focus: T) -> T {
        // Tags for the old subtree are now invalid
        let parent_node = self.tag_node.borrow().parent.clone();
        let new_tag = Rc::new(RefCell::new(TagNode {
            data: (self.construct_tag)(&new_focus),
            parent: parent_node,
            children: Vec::new(),
        }));
        let _ = std::mem::replace(&mut self.tag_node, new_tag);

        self.zipper.replace_focus(new_focus)
    }

    /// Rebuilds the root node, consuming the [`Zipper`].
    pub fn rebuild_root(self) -> T {
        // Don't care about tag structure as the zipper is consumed
        self.zipper.rebuild_root()
    }

    /// Borrows the tag associated with the current focus.
    pub fn tag(&self) -> Ref<D> {
        Ref::map(self.tag_node.borrow(), |node| &node.data)
    }

    /// Mutably borrows the tag associated with the current focus.
    pub fn tag_mut(&mut self) -> RefMut<D> {
        RefMut::map(self.tag_node.borrow_mut(), |node| &mut node.data)
    }

    /// Replaces the tag associated with the current focus with `new_tag`.
    pub fn replace_tag(&mut self, new_tag: D) {
        self.tag_node.borrow_mut().data = new_tag;
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
        let child_tag = self.tag_node.borrow().children.get(0).cloned();
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

        let parent_node = self.tag_node.borrow().parent.clone().unwrap();

        let sibling_i = self.zipper.siblings_index().unwrap();
        let sibling_node = parent_node.borrow().children.get(sibling_i).cloned();

        self.tag_node = match sibling_node {
            Some(tag) => tag.clone(),
            None => {
                let new_tag = Rc::new(RefCell::new(TagNode {
                    data: (self.construct_tag)(self.zipper.focus()),
                    parent: Some(parent_node.clone()),
                    children: Vec::new(),
                }));

                parent_node.borrow_mut().children.push(new_tag.clone());
                new_tag
            }
        };

        Some(())
    }
}
