//#![cfg(feature = "unstable")]

#![allow(clippy::type_complexity)]

mod context;

use context::{ContextIter, ContextIterBi};

use std::{collections::VecDeque, sync::Arc};

pub use super::Tree;

/// `Biplate<U>` for type `T` operates over all values of type `U` within `T`.
///
/// **Note: `Biplate<T>` for `T` returns the input expression, not its children of type `T`. Use
/// [`Uniplate`] instead.**
pub trait Biplate<To>
where
    Self: Sized + Clone + Eq + Uniplate + 'static,
    To: Sized + Clone + Eq + Uniplate + 'static,
{
    /// Definition of a Biplate.
    ///
    /// This is a low-level method useful only for implementing the `Biplate` trait.
    ///
    /// Returns all the top most children of type `To` within `From`.
    ///
    /// If from == to then this function should return the root as the single child.
    fn biplate(&self) -> (Tree<To>, Box<dyn Fn(Tree<To>) -> Self>);

    /// Reconstructs the node with the given children.
    ///
    /// Biplate variant of [`Uniplate::children`]
    ///
    /// # Panics
    ///
    /// If there are a different number of children given as there were originally returned by
    /// children().
    fn with_children_bi(&self, children: VecDeque<To>) -> Self {
        // 1. Turn old tree into list.
        // 2. Check lists are same size.
        // 3. Use the reconstruction function given by old_children.list() to
        //   create a tree with the same structure but the new lists' elements .

        let (old_children, ctx) = self.biplate();
        let (old_children_lst, rebuild) = old_children.list();
        if old_children_lst.len() != children.len() {
            panic!("with_children() given an unexpected amount of children");
        } else {
            ctx(rebuild(children))
        }
    }

    /// Biplate variant of [`Uniplate::descend`]
    ///
    /// If from == to then this function does not descend. Therefore, when writing definitions it
    /// is highly unlikely that this function should be used in the recursive case. A common
    /// pattern is to first match the types using descend_bi, then continue the recursion with
    /// descend.
    fn descend_bi(&self, op: Arc<dyn Fn(To) -> To>) -> Self {
        let (children, ctx) = self.biplate();
        ctx(children.map(op))
    }

    // NOTE (niklasdewally): Uniplate does something different here, and  I don't know why. In
    // particular, it doesn't use structure (its version of tree.list()) at all here, and uses some
    // builder thing I don't understand. My children_bi and universe_bi work though, so this might
    // just be performance and Haskell related.
    //
    // https://github.com/ndmitchell/uniplate/blob/66a2c55a7de0f5d8b0e437479719469244e00fa4/Data/Generics/Uniplate/Internal/OperationsInc.hs#L189

    /// Gets all children of a node, including itself and all children.
    ///
    /// Biplate variant of [`Uniplate::universe`]
    ///
    /// Universe_bi does a preorder traversal: it returns a given node first, followed by its
    /// children from left to right.
    ///
    /// If to == from then it returns the original element.
    fn universe_bi(&self) -> VecDeque<To> {
        self.children_bi()
            .into_iter()
            .flat_map(|child| child.universe())
            .collect()
    }

    /// Returns the children of a type. If to == from then it returns the original element (in contrast to children).
    ///
    /// Biplate variant of [`Uniplate::children`]
    fn children_bi(&self) -> VecDeque<To> {
        self.biplate().0.list().0
    }

    /// Applies the given function to all nodes bottom up.
    ///
    /// Biplate variant of [`Uniplate::transform`]
    fn transform_bi(&self, op: Arc<dyn Fn(To) -> To>) -> Self {
        let (children, ctx) = self.biplate();
        ctx(children.map(Arc::new(move |child| child.transform(op.clone()))))
    }

    /// Returns an iterator over all direct children of the input, paired with a function that
    /// "fills the hole" where the child was with a new value.
    ///
    /// `Biplate` variant of [`Uniplate::holes`]
    fn holes_bi(&self) -> impl Iterator<Item = (To, Arc<dyn Fn(To) -> Self>)> {
        // must be an iterator as we cannot clone Box<dyn Fn()>'s, so cannot stick them in
        // vectors, etc
        HolesBiIterator::new(self.clone())
    }

    /// Returns an iterator over the universe of the input, paired with a function that "fills the
    /// hole" where the child was with a new value.
    ///
    /// `Biplate` variant of [`Uniplate::contexts`]
    ///
    /// To efficiently update multiple values in a single traversal, use
    /// [`ZipperBi`](crate::zipper::ZipperBi) instead.
    fn contexts_bi(&self) -> impl Iterator<Item = (To, Arc<dyn Fn(To) -> Self>)> {
        ContextIterBi::new(self.clone())
    }
}

/// `Uniplate` for type `T` operates over all values of type `T` within `T`.
pub trait Uniplate
where
    Self: Sized + Clone + Eq + 'static,
{
    /// Definition of a `Uniplate`.
    ///
    /// This method is only useful for defining a Uniplate.
    fn uniplate(&self) -> (Tree<Self>, Box<dyn Fn(Tree<Self>) -> Self>);

    /// Applies a function to all direct children of this
    ///
    /// Consider using [`transform`](Uniplate::transform) instead, as it does bottom-up
    /// transformation of the entire tree.
    fn descend(&self, op: Arc<dyn Fn(Self) -> Self>) -> Self {
        let (children, ctx) = self.uniplate();
        ctx(children.map(op))
    }

    /// Gets all children of a node, including itself and all children.
    ///
    /// Universe does a preorder traversal: it returns a given node first, followed by its
    /// children from left to right.
    fn universe(&self) -> VecDeque<Self> {
        let mut results = VecDeque::from([self.clone()]);
        for child in self.children() {
            results.append(&mut child.universe());
        }
        results
    }

    /// Gets the direct children (maximal substructures) of a node.
    fn children(&self) -> VecDeque<Self> {
        let (children, _) = self.uniplate();
        children.list().0.clone()
    }

    /// Reconstructs the node with the given children.
    ///
    /// # Panics
    ///
    /// If there are a different number of children given as there were originally returned by
    /// children().
    fn with_children(&self, children: VecDeque<Self>) -> Self {
        // 1. Turn old tree into list.
        // 2. Check lists are same size.
        // 3. Use the reconstruction function given by old_children.list() to
        //   create a tree with the same structure but the new lists' elements .

        let (old_children, ctx) = self.uniplate();
        let (old_children_lst, rebuild) = old_children.list();
        if old_children_lst.len() != children.len() {
            panic!("with_children() given an unexpected amount of children");
        } else {
            ctx(rebuild(children))
        }
    }

    /// Applies the given function to all nodes bottom up.
    fn transform(&self, f: Arc<dyn Fn(Self) -> Self>) -> Self {
        let (children, ctx) = self.uniplate();
        let f2 = f.clone(); // make another pointer to f for map.
        f(ctx(
            children.map(Arc::new(move |child| child.transform(f2.clone())))
        ))
    }

    /// Rewrites by applying a rule everywhere it can.
    fn rewrite(&self, f: Arc<dyn Fn(Self) -> Option<Self>>) -> Self {
        let (children, ctx) = self.uniplate();

        let f2 = f.clone(); // make another pointer to f for map.
        let new_children = children.map(Arc::new(move |child| child.rewrite(f2.clone())));

        match f(ctx(new_children.clone())) {
            None => ctx(new_children),
            Some(n) => n,
        }
    }
    /// Performs a fold-like computation on each value.
    ///
    /// Working from the bottom up, this applies the given callback function to each nested
    /// component.
    ///
    /// Unlike [`transform`](Uniplate::transform), this returns an arbitrary type, and is not
    /// limited to T -> T transformations. In other words, it can transform a type into a new
    /// one.
    ///
    /// The meaning of the callback function is the following:
    ///
    ///   f(element_to_fold, folded_children) -> folded_element
    fn cata<T>(&self, op: Arc<dyn Fn(Self, VecDeque<T>) -> T>) -> T {
        let children = self.children();
        (*op)(
            self.clone(),
            children.into_iter().map(|c| c.cata(op.clone())).collect(),
        )
    }

    /// Returns an iterator over all direct children of the input, paired with a function that
    /// "fills the hole" where the child was with a new value.
    fn holes(&self) -> impl Iterator<Item = (Self, Arc<dyn Fn(Self) -> Self>)> {
        // must be an iterator as we cannot clone Box<dyn Fn()>'s, so cannot stick them in
        // vectors, etc

        HolesIterator::new(self.clone())
    }

    /// Returns an iterator over the universe of the input, paired with a function that "fills the
    /// hole" where the child was with a new value.
    ///
    /// The [`universe`](Uniplate::universe) equivalent of [`holes`](Uniplate::holes).
    ///
    /// To efficiently update multiple values in a single traversal, use
    /// [`Zipper`](crate::zipper::Zipper) instead.
    fn contexts(&self) -> impl Iterator<Item = (Self, Arc<dyn Fn(Self) -> Self>)> {
        ContextIter::new(self.clone())
    }
}

struct HolesIterator<T: Uniplate> {
    children_iter: std::iter::Enumerate<std::collections::vec_deque::IntoIter<T>>,
    children: VecDeque<T>,
    parent: T,
}

impl<T: Uniplate> Iterator for HolesIterator<T> {
    type Item = (T, Arc<dyn Fn(T) -> T>);

    fn next(&mut self) -> Option<Self::Item> {
        let (i, child) = self.children_iter.next()?;

        let children2 = self.children.clone();
        let parent2 = self.parent.clone();
        let hole = Arc::new(move |x: T| {
            let mut children = children2.clone();
            children[i] = x;
            parent2.with_children(children)
        });

        Some((child.clone(), hole))
    }
}

impl<T: Uniplate> HolesIterator<T> {
    fn new(parent: T) -> HolesIterator<T> {
        let children = parent.children();
        let children_iter = children.clone().into_iter().enumerate();

        HolesIterator {
            children_iter,
            children,
            parent,
        }
    }
}

struct HolesBiIterator<T: Uniplate, F: Biplate<T>> {
    children_iter: std::iter::Enumerate<std::collections::vec_deque::IntoIter<T>>,
    children: VecDeque<T>,
    parent: F,
}

impl<T: Uniplate, F: Biplate<T>> Iterator for HolesBiIterator<T, F> {
    type Item = (T, Arc<dyn Fn(T) -> F>);

    fn next(&mut self) -> Option<Self::Item> {
        let (i, child) = self.children_iter.next()?;

        let children2 = self.children.clone();
        let parent2 = self.parent.clone();
        let hole = Arc::new(move |x: T| {
            let mut children = children2.clone();
            children[i] = x;
            parent2.with_children_bi(children)
        });

        Some((child.clone(), hole))
    }
}

impl<T: Uniplate, F: Biplate<T>> HolesBiIterator<T, F> {
    fn new(parent: F) -> HolesBiIterator<T, F> {
        let children = parent.children_bi();
        let children_iter = children.clone().into_iter().enumerate();

        HolesBiIterator {
            children_iter,
            children,
            parent,
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::test_common::paper::{proptest_stmts, Expr, Stmt};

    use super::*;
    proptest! {
        #[test]
        fn test_context_same_as_universe(ast in proptest_stmts()) {
            prop_assert_eq!(ast.universe(),ast.contexts().map(|(elem,_)| elem).collect::<VecDeque<_>>());
        }

        #[test]
        fn test_holes_same_as_children(ast in proptest_stmts()) {
            prop_assert_eq!(ast.children(),ast.holes().map(|(elem,_)| elem).collect::<VecDeque<_>>());
        }

        #[test]
        fn test_context_bi_same_as_universe_bi(ast in proptest_stmts()) {
            prop_assert_eq!(Biplate::<Expr>::universe_bi(&ast),Biplate::<Expr>::contexts_bi(&ast).map(|(elem,_)| elem).collect::<VecDeque<_>>());
            prop_assert_eq!(Biplate::<Stmt>::universe_bi(&ast),Biplate::<Stmt>::contexts_bi(&ast).map(|(elem,_)| elem).collect::<VecDeque<_>>());
        }

        #[test]
        fn test_context_isomorphic(ast in proptest_stmts()) {
            for (e,c) in ast.contexts() {
                prop_assert_eq!(c(e.clone()),ast.clone())
            }
        }

        #[test]
        fn test_context_bi_isomorphic(ast in proptest_stmts()) {
            for (e,c) in Biplate::<Expr>::contexts_bi(&ast) {
                prop_assert_eq!(c(e.clone()),ast.clone())
            }
        }

        #[test]
        fn test_holes_bi_same_as_children_bi(ast in proptest_stmts()) {
            prop_assert_eq!(Biplate::<Expr>::children_bi(&ast),Biplate::<Expr>::holes_bi(&ast).map(|(elem,_)| elem).collect::<VecDeque<_>>());
            prop_assert_eq!(Biplate::<Stmt>::children_bi(&ast),Biplate::<Stmt>::holes_bi(&ast).map(|(elem,_)| elem).collect::<VecDeque<_>>());
        }
    }
}
