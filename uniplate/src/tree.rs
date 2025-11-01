use std::collections::VecDeque;

use self::Tree::*;

///
/// `Tree` stores the children of type `T` of a value, preserving its structure.
///
/// It is primarily used for implementing [`Uniplate`](super::Uniplate) and
/// [`Biplate`](super::Biplate) instances.
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Tree<T: Sized + Clone + Eq> {
    /// This element cannot contains no children.
    Zero,

    /// This element contains exactly one child.
    One(T),

    /// This element potentially contains many children.
    Many(VecDeque<Tree<T>>),
}

// NOTE (niklasdewally): This converts the entire tree into a list. Therefore this is only really
// worth it when we use all the children returned. This is what we use this for inside Uniplate.
// Because of this, I think a .iter() / IntoIterator for Tree<&T> is a bad idea.

impl<T: Sized + Clone + Eq> Tree<T> {
    /// Returns true if the tree contains any `One` variants, false otherwise.
    pub fn is_empty(&self) -> bool {
        match self {
            Tree::Zero => true,
            Tree::One(_) => false,
            Tree::Many(children) => children.iter().all(|tr| tr.is_empty()),
        }
    }
}

impl<T: Sized + Clone + Eq + 'static> IntoIterator for Tree<T> {
    type Item = T;

    type IntoIter = std::collections::vec_deque::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.list().0.into_iter()
    }
}
impl<T: Sized + Clone + Eq + 'static> Tree<T> {
    /// Returns the tree as a list alongside a function to reconstruct the tree from a list.
    ///
    /// This preserves the structure of the tree.
    #[allow(clippy::type_complexity)]
    pub fn list(self) -> (VecDeque<T>, Box<dyn Fn(VecDeque<T>) -> Tree<T>>) {
        // inspired by the Uniplate Haskell equivalent Data.Generics.Str::strStructure
        // https://github.com/ndmitchell/uniplate/blob/master/Data/Generics/Str.hs#L85

        fn flatten<T: Sized + Clone + Eq>(t: Tree<T>, xs: VecDeque<T>) -> VecDeque<T> {
            match (t, xs) {
                (Zero, xs) => xs,
                (One(x), mut xs1) => {
                    xs1.push_back(x);
                    xs1
                }
                (Many(ts), xs) => ts.into_iter().fold(xs, |xs, t| flatten(t, xs)),
            }
        }

        // Iterate over both the old tree and the new list.
        // We use the node types of the old tree to know what node types to use for the new tree.
        fn recons<T: Sized + Clone + Eq>(
            old_tree: Tree<T>,
            xs: VecDeque<T>,
        ) -> (Tree<T>, VecDeque<T>) {
            #[allow(clippy::unwrap_used)]
            match (old_tree, xs) {
                (Zero, xs) => (Zero, xs),
                (One(_), mut xs1) => (One(xs1.pop_front().unwrap()), xs1),
                (Many(ts), xs) => {
                    let (ts1, xs1) =
                        ts.into_iter()
                            .fold((VecDeque::new(), xs), |(mut ts1, xs), t| {
                                let (t1, xs1) = recons(t, xs);
                                ts1.push_back(t1);
                                (ts1, xs1)
                            });
                    (Many(ts1), xs1)
                }
            }
        }
        (
            flatten(self.clone(), VecDeque::new()),
            Box::new(move |xs| recons(self.clone(), xs).0),
        )
    }

    /// Applies a function over all elements in the tree.
    pub fn map(self, op: &impl Fn(T) -> T) -> Tree<T> {
        match self {
            Zero => Zero,
            One(t) => One(op(t)),
            Many(ts) => Many(ts.into_iter().map(|t| t.map(op)).collect::<_>()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::iter::zip;

    use proptest::prelude::*;

    use super::*;

    #[allow(dead_code)]
    // Used by proptest for generating test instances of Tree<i32>.
    fn proptest_integer_trees() -> impl Strategy<Value = Tree<i32>> {
        // https://proptest-rs.github.io/proptest/proptest/tutorial/enums.html
        // https://proptest-rs.github.io/proptest/proptest/tutorial/recursive.html
        let leaf = prop_oneof![Just(Tree::Zero), any::<i32>().prop_map(Tree::One),];

        leaf.prop_recursive(
            10,  // levels deep
            512, // Shoot for maximum size of 512 nodes
            20,  // We put up to 20 items per collection
            |inner| proptest::collection::vec_deque(inner.clone(), 0..20).prop_map(Tree::Many),
        )
    }

    proptest! {
        #[test]
        // Is tree.recons() isomorphic?
        fn list_is_isomorphic(tree in proptest_integer_trees()) {
            let (children,func) = tree.clone().list();
            let new_tree = func(children);
            prop_assert_eq!(new_tree,tree);
        }

        #[test]
        fn map_add(tree in proptest_integer_trees(), diff in -100i32..100i32) {
            let new_tree = tree.clone().map(&|a| a+diff);
            let (old_children,_) = tree.list();
            let (new_children,_) = new_tree.list();

            for (old,new) in zip(old_children,new_children) {
                prop_assert_eq!(old+diff,new);
            }
        }
    }
    #[test]
    fn list_preserves_ordering() {
        let my_tree: Tree<i32> = Many(VecDeque::from([
            Many(VecDeque::from([One(0), Zero])),
            Many(VecDeque::from([Many(VecDeque::from([
                Zero,
                One(1),
                One(2),
            ]))])),
            One(3),
            Zero,
            One(4),
        ]));

        let flat = my_tree.list().0;

        for i in 0..5 {
            assert_eq!(flat[i], i.try_into().unwrap());
        }
    }
}
