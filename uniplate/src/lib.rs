#![doc = include_str!("intro.md")]
#![warn(missing_docs)]

#[doc(hidden)]
extern crate self as uniplate;

#[doc(hidden)]
pub mod impls;

pub mod zipper;

mod traits;
mod tree;

pub use traits::{Biplate, Uniplate};

pub use tree::Tree;

#[doc(hidden)]
pub mod test_common;

/// The Uniplate derive macro.
///
/// The macro supports `structs` (including [tuple
/// structs](https://doc.rust-lang.org/stable/reference/items/structs.html#r-items.struct.tuple))
/// and `enums`.
///
/// Enums with [struct-like
/// variants](https://doc.rust-lang.org/stable/reference/items/enumerations.html#r-items.enum.struct-expr)
/// are not yet supported.
///
/// **See the top level crate documentation for usage details.**
pub use uniplate_derive::Uniplate;

/// Generates [`Biplate`] and [`Uniplate`] instances for an unplateable type.
///
/// An unplateable type is one that you don't want Uniplate to traverse inside of.
///
/// The type must implement `Clone` and `Eq`.
///
/// Consider marking a type unplateable if it has no children (e.g. `String`) or does not support
/// the derive macro, but you still need a `Uniplate` or `Biplate` implementation for it.
///
/// # Example
///
/// For example, the target of a `Biplate` operation must implement `Uniplate`.
///
/// The below example uses `Biplate` to get all the `Names` in a binary tree.
///
/// ```
/// use uniplate::{derive_unplateable,Uniplate,Biplate};
///
/// // If you don't care about the children of a type, use derive_unplateable!
/// #[derive(Clone,PartialEq,Eq)]
/// struct Name {
///   first: String,
///   last: String
/// }
///
/// derive_unplateable!(Name);
///
/// #[derive(Clone,PartialEq,Eq,Uniplate)]
/// #[uniplate()]
/// #[biplate(to=Name)]
/// enum MyTree {
///   Leaf(Name),
///   Branch(Name,Box<MyTree>,Box<MyTree>)
/// }
///
/// /// Gets all the names in the tree
/// fn names_in_tree(tree: &MyTree) -> Vec<Name> {
///     let names: Vec<Name> = tree.universe_bi().into_iter().collect();
///     names
/// }
/// ```
#[macro_export]
macro_rules! derive_unplateable {
    ($t:ty) => {
        impl ::uniplate::Uniplate for $t {
            fn uniplate(
                &self,
            ) -> (
                ::uniplate::Tree<Self>,
                Box<dyn Fn(::uniplate::Tree<Self>) -> Self>,
            ) {
                let val = self.clone();
                (::uniplate::Tree::Zero, Box::new(move |_| val.clone()))
            }
        }

        impl ::uniplate::Biplate<$t> for $t {
            fn biplate(
                &self,
            ) -> (
                ::uniplate::Tree<$t>,
                Box<dyn Fn(::uniplate::Tree<$t>) -> $t>,
            ) {
                let val = self.clone();
                (
                    ::uniplate::Tree::One(val.clone()),
                    Box::new(move |x| {
                        let ::uniplate::Tree::One(x) = x else {
                            panic!();
                        };
                        x
                    }),
                )
            }
        }

        impl ::uniplate::Biplate<Option<$t>> for $t {
            fn biplate(
                &self,
            ) -> (
                ::uniplate::Tree<Option<$t>>,
                Box<dyn Fn(::uniplate::Tree<Option<$t>>) -> $t>,
            ) {
                let val = self.clone();
                (::uniplate::Tree::Zero, Box::new(move |_| val.clone()))
            }
        }
    };
}

/// Generates [`Biplate`] and [`Uniplate`] instances for a collection using its [`Iterator`]
/// implementation.
///
/// Children will be visited in the order returned by `.iter()`.
#[macro_export]
macro_rules! derive_iter {
    ($iter_ty:ident) => {
        impl<T, F> ::uniplate::Biplate<T> for $iter_ty<F>
        where
            T: Clone + Eq + ::uniplate::Uniplate + Sized + 'static,
            F: Clone + Eq + ::uniplate::Uniplate + ::uniplate::Biplate<T> + Sized + 'static,
        {
            fn biplate(
                &self,
            ) -> (
                ::uniplate::Tree<T>,
                Box<(dyn Fn(::uniplate::Tree<T>) -> $iter_ty<F>)>,
            ) {
                if (self.is_empty()) {
                    let val = self.clone();
                    return (::uniplate::Tree::Zero, Box::new(move |_| val.clone()));
                }

                // T == F: return all types F in the iterator.
                if std::any::TypeId::of::<T>() == std::any::TypeId::of::<F>() {
                    unsafe {
                        // need to cast from F to T
                        let children: ::uniplate::Tree<T> = ::uniplate::Tree::Many(
                            self.clone()
                                .into_iter()
                                .map(|x: F| {
                                    // possibly unsafe, definitely stupid, but seems to be the only way here?
                                    let x: T = std::mem::transmute::<&F, &T>(&x).clone();
                                    ::uniplate::Tree::One(x)
                                })
                                .collect(),
                        );

                        let ctx: Box<dyn Fn(::uniplate::Tree<T>) -> $iter_ty<F>> =
                            Box::new(move |new_tree: ::uniplate::Tree<T>| {
                                let ::uniplate::Tree::Many(xs) = new_tree else {
                                    todo!();
                                };
                                xs.into_iter()
                                    .map(|x| {
                                        let ::uniplate::Tree::One(x) = x else {
                                            todo!();
                                        };
                                        let x: F = std::mem::transmute::<&T, &F>(&x).clone();
                                        x
                                    })
                                    .collect()
                            });

                        return (children, ctx);
                    }
                }
                // Identity / same type case: Biplate<Iter<T>> for Iter<T>
                else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<$iter_ty<F>>() {
                    unsafe {
                        // need to cast from Iter<F> to T
                        let val: T = std::mem::transmute::<&$iter_ty<F>, &T>(&self).clone();

                        let children: ::uniplate::Tree<T> = ::uniplate::Tree::One(val);

                        let ctx: Box<dyn Fn(::uniplate::Tree<T>) -> $iter_ty<F>> =
                            Box::new(move |new_tree: ::uniplate::Tree<T>| {
                                let ::uniplate::Tree::One(x) = new_tree else {
                                    todo!();
                                };
                                // need to cast from T to Iter<F>
                                let val: $iter_ty<F> =
                                    std::mem::transmute::<&T, &$iter_ty<F>>(&x).clone();
                                val
                            });

                        return (children, ctx);
                    }
                }

                // T != F: return all type T's contained in the type F's in the vector
                let mut child_trees: VecDeque<::uniplate::Tree<T>> = VecDeque::new();
                let mut child_ctxs: Vec<Box<dyn Fn(::uniplate::Tree<T>) -> F>> = Vec::new();
                for item in self {
                    let (tree, plate) = <F as ::uniplate::Biplate<T>>::biplate(item);
                    child_trees.push_back(tree);
                    child_ctxs.push(plate);
                }

                let tree = ::uniplate::Tree::Many(child_trees);
                let ctx = Box::new(move |new_tree: ::uniplate::Tree<T>| {
                    let mut out = Vec::<F>::new();
                    let ::uniplate::Tree::Many(new_trees) = new_tree else {
                        todo!()
                    };
                    for (child_tree, child_ctx) in std::iter::zip(new_trees, &child_ctxs) {
                        out.push(child_ctx(child_tree));
                    }
                    out.into_iter().collect::<$iter_ty<F>>()
                });
                (tree, ctx)
            }
        }

        // Traversal Biplate
        impl<T> ::uniplate::Uniplate for $iter_ty<T>
        where
            T: Clone + Eq + ::uniplate::Uniplate + Sized + 'static,
        {
            fn uniplate(
                &self,
            ) -> (
                ::uniplate::Tree<Self>,
                Box<dyn Fn(::uniplate::Tree<Self>) -> Self>,
            ) {
                let val = self.clone();
                (Zero, Box::new(move |_| val.clone()))
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! unreachable {
    ($from:ident,$to:ident) => {
        impl ::uniplate::Biplate<$to> for $from {
            fn biplate(
                &self,
            ) -> (
                ::uniplate::Tree<$to>,
                Box<dyn Fn(::uniplate::Tree<$to>) -> $from>,
            ) {
                let val = self.clone();
                (::uniplate::Tree::Zero, Box::new(move |_| val.clone()))
            }
        }
    };
}
