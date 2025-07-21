use super::holes::HolesIterBi;
use super::{context::ContextIterBi, Uniplate};

use std::collections::VecDeque;

pub use crate::Tree;
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
    fn descend_bi(&self, op: &impl Fn(To) -> To) -> Self {
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
    fn transform_bi(&self, op: &impl Fn(To) -> To) -> Self {
        self.descend_bi(&|x| x.transform(op))
    }

    /// Returns an iterator over all direct children of the input, paired with a function that
    /// "fills the hole" where the child was with a new value.
    ///
    /// `Biplate` variant of [`Uniplate::holes`]
    fn holes_bi(&self) -> impl Iterator<Item = (To, impl Fn(To) -> Self)> {
        // must be an iterator as we cannot clone Box<dyn Fn()>'s, so cannot stick them in
        // vectors, etc
        HolesIterBi::new(self.clone())
    }

    /// Returns an iterator over the universe of the input, paired with a function that "fills the
    /// hole" where the child was with a new value.
    ///
    /// `Biplate` variant of [`Uniplate::contexts`]
    ///
    /// To efficiently update multiple values in a single traversal, use
    /// [`ZipperBi`](crate::zipper::ZipperBi) instead.
    fn contexts_bi(&self) -> impl Iterator<Item = (To, impl Fn(To) -> Self)> {
        ContextIterBi::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::test_common::paper::{proptest_stmts, Expr, Stmt};

    use super::*;
    proptest! {
        #[test]
        fn test_context_bi_same_as_universe_bi(ast in proptest_stmts()) {
            prop_assert_eq!(Biplate::<Expr>::universe_bi(&ast),Biplate::<Expr>::contexts_bi(&ast).map(|(elem,_)| elem).collect::<VecDeque<_>>());
            prop_assert_eq!(Biplate::<Stmt>::universe_bi(&ast),Biplate::<Stmt>::contexts_bi(&ast).map(|(elem,_)| elem).collect::<VecDeque<_>>());
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
