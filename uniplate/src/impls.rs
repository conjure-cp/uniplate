//! Implementations of Uniplate and Biplate for common types.
//!
//! This includes stdlib types as well as common collections
//!
//! Box types are excluded, and are inlined by the macro.

// NOTE (niklasdewally): my assumption is that we can do all this here, and that llvm will inline
// this and/or devirtualise the Box<dyn Fn()> when necessary to make this fast.
// https://users.rust-lang.org/t/why-box-dyn-fn-is-the-same-fast-as-normal-fn/96392

use std::collections::VecDeque;

use crate::derive_iter;
use crate::derive_unplateable;
use crate::try_biplate_to;
use crate::Biplate;
use crate::Tree;
use crate::Tree::*;
use crate::Uniplate;

// `std` base types {{{
derive_unplateable!(i8);
derive_unplateable!(i16);
derive_unplateable!(i32);
derive_unplateable!(i64);
derive_unplateable!(i128);
derive_unplateable!(u8);
derive_unplateable!(u16);
derive_unplateable!(u32);
derive_unplateable!(u64);
derive_unplateable!(u128);
derive_unplateable!(String);
// }}}
// `std` collections {{{

// Implement Biplate for collections by converting them to iterators.

derive_iter!(Vec);
derive_iter!(VecDeque);

// }}}
// `std::option::Option` and `std::result::Result` {{{

//
// + Biplate<A> for Option<T>:
//
//     - `None`  => `Tree::Zero`
//     - `Some(x)` => `<T as Biplate<A>>::biplate(x)`
//
//       (The `A===T` case returns `x` due to the Biplate base case.)
//
// + Biplate<Option<T>> for Option<T>:
//
//     - return input expression.
//
// + Uniplate for Option<T>:
//
//     - `Some(x)` => <T as Biplate<Option<T>>>::biplate(x)
//
//        (NB: Biplate<Option<T>> for T is derived as part of T's implementation not generically here.)
//
//        TODO: the derive macro should derive `Biplate<Option<T>> for T` by default and walk into
//        them by default. If there are no `Option<T>'s, a stub implementation should be added.
//
//     - `None` => `Tree::Zero`.

impl<T> Uniplate for Option<T>
where
    T: Uniplate + Biplate<Option<T>>,
{
    fn uniplate(&self) -> (crate::Tree<Self>, Box<dyn Fn(crate::Tree<Self>) -> Self>) {
        match self {
            Some(x) => {
                let (tree, ctx) = <T as Biplate<Option<T>>>::biplate(x);

                (tree, Box::new(move |x| Some(ctx(x))))
            }
            None => (Tree::Zero, Box::new(move |_| None)),
        }
    }
}

impl<From, To> Biplate<To> for Option<From>
where
    To: Uniplate,
    From: Uniplate + Biplate<Option<From>> + Biplate<To>,
{
    fn biplate(&self) -> (Tree<To>, Box<dyn Fn(Tree<To>) -> Self>) {
        if std::any::TypeId::of::<To>() == std::any::TypeId::of::<Option<From>>() {
            unsafe {
                // Convert self: Option<From> to self: To, and return self.
                // SAFETY: checked the types above.
                let self_as_to: &To = std::mem::transmute(self);
                (
                    Tree::One(self_as_to.clone()),
                    Box::new(move |x| {
                        let Tree::One(x) = x else {
                            panic!();
                        };

                        let x_as_option_from: &Option<From> = std::mem::transmute(&x);
                        x_as_option_from.clone()
                    }),
                )
            }
        } else {
            match self {
                Some(x) => {
                    let (tree, ctx) = <From as Biplate<To>>::biplate(x);
                    (tree, Box::new(move |x| Some(ctx(x))))
                }
                None => (Tree::Zero, Box::new(move |_| None)),
            }
        }
    }
}

// tuples {{{
impl<T: Uniplate, U: Uniplate> Uniplate for (T, U) {
    fn uniplate(&self) -> (Tree<Self>, Box<dyn Fn(Tree<Self>) -> Self>) {
        let (t, u) = self.clone();
        let (t_tree, t_recons) = try_biplate_to!(t, (T, U));
        let (u_tree, u_recons) = try_biplate_to!(u, (T, U));

        let tree = Tree::Many(VecDeque::from([t_tree, u_tree]));

        let ctx = Box::new(move |x| {
            let Tree::Many(xs) = x else {
                panic!();
            };
            let t = t_recons(xs[0].clone());
            let u = u_recons(xs[1].clone());
            (t, u)
        });

        (tree, ctx)
    }
}

impl<T: Uniplate, U: Uniplate, To: Uniplate> Biplate<To> for (T, U) {
    fn biplate(&self) -> (Tree<To>, Box<dyn Fn(Tree<To>) -> Self>) {
        if std::any::TypeId::of::<To>() == std::any::TypeId::of::<(T, U)>() {
            unsafe {
                // Convert self: (T,U) to self: To, and return self.
                // SAFETY: checked the types above.
                let self_as_to: &To = std::mem::transmute(self);
                (
                    Tree::One(self_as_to.clone()),
                    Box::new(move |x| {
                        let Tree::One(x) = x else {
                            panic!();
                        };

                        let x_as_tuple: &(T, U) = std::mem::transmute(&x);
                        x_as_tuple.clone()
                    }),
                )
            }
        } else {
            let (t, u) = self.clone();
            let (t_tree, t_recons) = try_biplate_to!(t, To);
            let (u_tree, u_recons) = try_biplate_to!(u, To);

            let tree = Tree::Many(VecDeque::from([t_tree, u_tree]));

            let ctx = Box::new(move |x| {
                let Tree::Many(xs) = x else {
                    panic!();
                };
                let t = t_recons(xs[0].clone());
                let u = u_recons(xs[1].clone());
                (t, u)
            });

            (tree, ctx)
        }
    }
}

impl<T: Uniplate, U: Uniplate, V: Uniplate> Uniplate for (T, U, V) {
    fn uniplate(&self) -> (Tree<Self>, Box<dyn Fn(Tree<Self>) -> Self>) {
        let (t, u, v) = self.clone();
        let (t_tree, t_recons) = try_biplate_to!(t, (T, U, V));
        let (u_tree, u_recons) = try_biplate_to!(u, (T, U, V));
        let (v_tree, v_recons) = try_biplate_to!(v, (T, U, V));

        let tree = Tree::Many(VecDeque::from([t_tree, u_tree, v_tree]));

        let ctx = Box::new(move |x| {
            let Tree::Many(xs) = x else {
                panic!();
            };
            let t = t_recons(xs[0].clone());
            let u = u_recons(xs[1].clone());
            let v = v_recons(xs[2].clone());
            (t, u, v)
        });

        (tree, ctx)
    }
}

impl<T: Uniplate, U: Uniplate, V: Uniplate, To: Uniplate> Biplate<To> for (T, U, V) {
    fn biplate(&self) -> (Tree<To>, Box<dyn Fn(Tree<To>) -> Self>) {
        if std::any::TypeId::of::<To>() == std::any::TypeId::of::<(T, U, V)>() {
            unsafe {
                // Convert self: (T,U,V) to self: To, and return self.
                // SAFETY: checked the types above.
                let self_as_to: &To = std::mem::transmute(self);
                (
                    Tree::One(self_as_to.clone()),
                    Box::new(move |x| {
                        let Tree::One(x) = x else {
                            panic!();
                        };

                        let x_as_tuple: &(T, U, V) = std::mem::transmute(&x);
                        x_as_tuple.clone()
                    }),
                )
            }
        } else {
            let (t, u, v) = self.clone();
            let (t_tree, t_recons) = try_biplate_to!(t, To);
            let (u_tree, u_recons) = try_biplate_to!(u, To);
            let (v_tree, v_recons) = try_biplate_to!(v, To);

            let tree = Tree::Many(VecDeque::from([t_tree, u_tree, v_tree]));

            let ctx = Box::new(move |x| {
                let Tree::Many(xs) = x else {
                    panic!();
                };
                let t = t_recons(xs[0].clone());
                let u = u_recons(xs[1].clone());
                let v = v_recons(xs[2].clone());
                (t, u, v)
            });

            (tree, ctx)
        }
    }
}

impl<T: Uniplate, U: Uniplate, V: Uniplate, W: Uniplate> Uniplate for (T, U, V, W) {
    fn uniplate(&self) -> (Tree<Self>, Box<dyn Fn(Tree<Self>) -> Self>) {
        let (t, u, v, w) = self.clone();
        let (t_tree, t_recons) = try_biplate_to!(t, (T, U, V, W));
        let (u_tree, u_recons) = try_biplate_to!(u, (T, U, V, W));
        let (v_tree, v_recons) = try_biplate_to!(v, (T, U, V, W));
        let (w_tree, w_recons) = try_biplate_to!(w, (T, U, V, W));

        let tree = Tree::Many(VecDeque::from([t_tree, u_tree, v_tree, w_tree]));

        let ctx = Box::new(move |x| {
            let Tree::Many(xs) = x else {
                panic!();
            };
            let t = t_recons(xs[0].clone());
            let u = u_recons(xs[1].clone());
            let v = v_recons(xs[2].clone());
            let w = w_recons(xs[3].clone());
            (t, u, v, w)
        });

        (tree, ctx)
    }
}

impl<T: Uniplate, U: Uniplate, V: Uniplate, W: Uniplate, To: Uniplate> Biplate<To>
    for (T, U, V, W)
{
    fn biplate(&self) -> (Tree<To>, Box<dyn Fn(Tree<To>) -> Self>) {
        if std::any::TypeId::of::<To>() == std::any::TypeId::of::<(T, U, V, W)>() {
            unsafe {
                // Convert self: (T,U,V,W) to self: To, and return self.
                // SAFETY: checked the types above.
                let self_as_to: &To = std::mem::transmute(self);
                (
                    Tree::One(self_as_to.clone()),
                    Box::new(move |x| {
                        let Tree::One(x) = x else {
                            panic!();
                        };

                        let x_as_tuple: &(T, U, V, W) = std::mem::transmute(&x);
                        x_as_tuple.clone()
                    }),
                )
            }
        } else {
            let (t, u, v, w) = self.clone();
            let (t_tree, t_recons) = try_biplate_to!(t, To);
            let (u_tree, u_recons) = try_biplate_to!(u, To);
            let (v_tree, v_recons) = try_biplate_to!(v, To);
            let (w_tree, w_recons) = try_biplate_to!(w, To);

            let tree = Tree::Many(VecDeque::from([t_tree, u_tree, v_tree, w_tree]));

            let ctx = Box::new(move |x| {
                let Tree::Many(xs) = x else {
                    panic!();
                };
                let t = t_recons(xs[0].clone());
                let u = u_recons(xs[1].clone());
                let v = v_recons(xs[2].clone());
                let w = w_recons(xs[3].clone());
                (t, u, v, w)
            });

            (tree, ctx)
        }
    }
}

impl<T: Uniplate, U: Uniplate, V: Uniplate, W: Uniplate, X: Uniplate> Uniplate for (T, U, V, W, X) {
    fn uniplate(&self) -> (Tree<Self>, Box<dyn Fn(Tree<Self>) -> Self>) {
        let (t, u, v, w, x) = self.clone();
        let (t_tree, t_recons) = try_biplate_to!(t, (T, U, V, W, X));
        let (u_tree, u_recons) = try_biplate_to!(u, (T, U, V, W, X));
        let (v_tree, v_recons) = try_biplate_to!(v, (T, U, V, W, X));
        let (w_tree, w_recons) = try_biplate_to!(w, (T, U, V, W, X));
        let (x_tree, x_recons) = try_biplate_to!(x, (T, U, V, W, X));

        let tree = Tree::Many(VecDeque::from([t_tree, u_tree, v_tree, w_tree, x_tree]));

        let ctx = Box::new(move |x| {
            let Tree::Many(xs) = x else {
                panic!();
            };
            let t = t_recons(xs[0].clone());
            let u = u_recons(xs[1].clone());
            let v = v_recons(xs[2].clone());
            let w = w_recons(xs[3].clone());
            let x = x_recons(xs[4].clone());
            (t, u, v, w, x)
        });

        (tree, ctx)
    }
}

impl<T: Uniplate, U: Uniplate, V: Uniplate, W: Uniplate, X: Uniplate, To: Uniplate> Biplate<To>
    for (T, U, V, W, X)
{
    fn biplate(&self) -> (Tree<To>, Box<dyn Fn(Tree<To>) -> Self>) {
        if std::any::TypeId::of::<To>() == std::any::TypeId::of::<(T, U, V, W, X)>() {
            unsafe {
                // Convert self: (T,U,V,W,X) to self: To, and return self.
                // SAFETY: checked the types above.
                let self_as_to: &To = std::mem::transmute(self);
                (
                    Tree::One(self_as_to.clone()),
                    Box::new(move |x| {
                        let Tree::One(x) = x else {
                            panic!();
                        };

                        let x_as_tuple: &(T, U, V, W, X) = std::mem::transmute(&x);
                        x_as_tuple.clone()
                    }),
                )
            }
        } else {
            let (t, u, v, w, x) = self.clone();
            let (t_tree, t_recons) = try_biplate_to!(t, To);
            let (u_tree, u_recons) = try_biplate_to!(u, To);
            let (v_tree, v_recons) = try_biplate_to!(v, To);
            let (w_tree, w_recons) = try_biplate_to!(w, To);
            let (x_tree, x_recons) = try_biplate_to!(x, To);

            let tree = Tree::Many(VecDeque::from([t_tree, u_tree, v_tree, w_tree, x_tree]));

            let ctx = Box::new(move |x| {
                let Tree::Many(xs) = x else {
                    panic!();
                };
                let t = t_recons(xs[0].clone());
                let u = u_recons(xs[1].clone());
                let v = v_recons(xs[2].clone());
                let w = w_recons(xs[3].clone());
                let x = x_recons(xs[4].clone());
                (t, u, v, w, x)
            });

            (tree, ctx)
        }
    }
}

// }}}
#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use crate::Biplate as _;

    #[test]
    fn option_with_children_bi_test() {
        let expr = Some(10);
        let expected = Some(11);
        let actual: Option<i32> = expr.with_children_bi(VecDeque::from([11]));
        assert_eq!(actual, expected);
    }
}

// TODO: Add results. We might want to somehow make it optional whether we traverse into an error
// type or not, allowing errors to not implement Uniplate / Biplate.
//
//  Result is similar to `Option<T>`, but we also need to look inside the error values.
//
// + Biplate<A> for Result<T,U>:
//
//     - `Ok(x)` => <T as Biplate<A>>::biplate(x)
//     - `Err(x)` => <U as Biplate<A>>::biplate(x)
//
//     By the Biplate base-case, correctly covers the `A===T` and `A==U` cases.
//
// + Biplate<Result<T,U>> for Result<T,U>: return input expression.
//
// + Uniplate for Result<T,U>:
//
//     - `Ok(x)` => <T as Biplate<Result<T,U>>>::biplate(x)
//     - `Err(x)` => <U as Biplate<Result<T,U>>>::biplate(x)
//
//       (The `A===T` and `A==U` cases return `x` due to the Biplate base case.)
//

// }}}

// vim: foldmethod=marker foldlevel=0
