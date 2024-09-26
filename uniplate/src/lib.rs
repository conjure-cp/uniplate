#![doc = include_str!("intro.md")]

#[doc(hidden)]
extern crate self as uniplate;

pub mod impls;
mod traits;
mod tree;

pub use traits::{Biplate, Uniplate};

#[doc(hidden)]
pub use tree::Tree;

#[doc(hidden)]
pub mod test_common;

/// The derive macro.
pub mod derive {
    pub use uniplate_derive::Uniplate;
}

#[doc(hidden)]
pub mod _dependencies {
    pub use im;
}

/// Generates a Biplate and Uniplate instance for an unplateable type.
#[macro_export]
macro_rules! derive_unplateable {
    ($t:ty) => {
        impl Uniplate for $t {
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

        impl Biplate<$t> for $t {
            fn biplate(
                &self,
            ) -> (
                ::uniplate::Tree<$t>,
                Box<dyn Fn(::uniplate::Tree<$t>) -> $t>,
            ) {
                let val = self.clone();
                (
                    ::uniplate::Tree::One(val.clone()),
                    Box::new(move |_| val.clone()),
                )
            }
        }
    };
}

/// Generates a Biplate and Uniplate instance for an iterable type.
#[macro_export]
macro_rules! derive_iter {
    ($iter_ty:ident) => {
        impl<T, F> Biplate<T> for $iter_ty<F>
        where
            T: Clone + Eq + Uniplate + Sized + 'static,
            F: Clone + Eq + Uniplate + Biplate<T> + Sized + 'static,
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

                // this is an example of the special biplate case discussed in the paper.
                // T == F: return all types F in the Vector.
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

                // T != F: return all type T's contained in the type F's in the vector
                let mut child_trees: im::Vector<::uniplate::Tree<T>> = im::Vector::new();
                let mut child_ctxs: Vec<Box<dyn Fn(::uniplate::Tree<T>) -> F>> = Vec::new();
                for item in self {
                    let (tree, plate) = <F as Biplate<T>>::biplate(item);
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
        impl<T> Uniplate for $iter_ty<T>
        where
            T: Clone + Eq + Uniplate + Sized + 'static,
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
