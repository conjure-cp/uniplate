use super::context::ContextIter;
use super::holes::HolesIter;

use std::collections::VecDeque;

use crate::Tree;

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
    fn descend(&self, op: &impl Fn(Self) -> Self) -> Self {
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
    fn transform(&self, f: &impl Fn(Self) -> Self) -> Self {
        let (children, ctx) = self.uniplate();
        f(ctx(children.map(&|child| child.transform(f))))
    }

    /// Rewrites by applying a rule everywhere it can.
    fn rewrite(&self, f: &impl Fn(Self) -> Option<Self>) -> Self {
        let (children, ctx) = self.uniplate();

        let new_children = children.map(&|child| child.rewrite(f));

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
    fn cata<T>(&self, op: &impl Fn(Self, VecDeque<T>) -> T) -> T {
        let children = self.children();
        (*op)(
            self.clone(),
            children.into_iter().map(|c| c.cata(op)).collect(),
        )
    }

    /// Returns an iterator over all direct children of the input, paired with a function that
    /// "fills the hole" where the child was with a new value.
    fn holes(&self) -> impl Iterator<Item = (Self, impl Fn(Self) -> Self)> {
        // must be an iterator as we cannot clone Box<dyn Fn()>'s, so cannot stick them in
        // vectors, etc

        HolesIter::new(self.clone())
    }

    /// Returns an iterator over the universe of the input, paired with a function that "fills the
    /// hole" where the child was with a new value.
    ///
    /// The [`universe`](Uniplate::universe) equivalent of [`holes`](Uniplate::holes).
    ///
    /// To efficiently update multiple values in a single traversal, use
    /// [`Zipper`](crate::zipper::Zipper) instead.
    fn contexts(&self) -> impl Iterator<Item = (Self, impl Fn(Self) -> Self)> {
        ContextIter::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::test_common::paper::proptest_stmts;

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
        fn test_context_isomorphic(ast in proptest_stmts()) {
            for (e,c) in ast.contexts() {
                prop_assert_eq!(c(e.clone()),ast.clone())
            }
        }
    }
}
