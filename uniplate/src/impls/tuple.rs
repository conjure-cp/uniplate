//! Uniplate and Biplate instances for tuples
use std::collections::VecDeque;

use crate::Biplate;
use crate::Tree;
use crate::Uniplate;
use crate::try_biplate_to;

impl<T: Uniplate + Biplate<(T, U)>, U: Uniplate + Biplate<(T, U)>> Uniplate for (T, U) {
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

impl<
    T: Uniplate + Biplate<To> + Biplate<(T, U)>,
    U: Uniplate + Biplate<To> + Biplate<(T, U)>,
    To: Uniplate,
> Biplate<To> for (T, U)
{
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

impl<
    T: Uniplate + Biplate<(T, U, V)>,
    U: Uniplate + Biplate<(T, U, V)>,
    V: Uniplate + Biplate<(T, U, V)>,
> Uniplate for (T, U, V)
{
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

impl<
    T: Uniplate + Biplate<To> + Biplate<(T, U, V)>,
    U: Uniplate + Biplate<To> + Biplate<(T, U, V)>,
    V: Uniplate + Biplate<To> + Biplate<(T, U, V)>,
    To: Uniplate,
> Biplate<To> for (T, U, V)
{
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

impl<
    T: Uniplate + Biplate<(T, U, V, W)>,
    U: Uniplate + Biplate<(T, U, V, W)>,
    V: Uniplate + Biplate<(T, U, V, W)>,
    W: Uniplate + Biplate<(T, U, V, W)>,
> Uniplate for (T, U, V, W)
{
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

impl<
    T: Uniplate + Biplate<To> + Biplate<(T, U, V, W)>,
    U: Uniplate + Biplate<To> + Biplate<(T, U, V, W)>,
    V: Uniplate + Biplate<To> + Biplate<(T, U, V, W)>,
    W: Uniplate + Biplate<To> + Biplate<(T, U, V, W)>,
    To: Uniplate,
> Biplate<To> for (T, U, V, W)
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

impl<
    T: Uniplate + Biplate<(T, U, V, W, X)>,
    U: Uniplate + Biplate<(T, U, V, W, X)>,
    V: Uniplate + Biplate<(T, U, V, W, X)>,
    W: Uniplate + Biplate<(T, U, V, W, X)>,
    X: Uniplate + Biplate<(T, U, V, W, X)>,
> Uniplate for (T, U, V, W, X)
{
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

impl<
    T: Uniplate + Biplate<To> + Biplate<(T, U, V, W, X)>,
    U: Uniplate + Biplate<To> + Biplate<(T, U, V, W, X)>,
    V: Uniplate + Biplate<To> + Biplate<(T, U, V, W, X)>,
    W: Uniplate + Biplate<To> + Biplate<(T, U, V, W, X)>,
    X: Uniplate + Biplate<To> + Biplate<(T, U, V, W, X)>,
    To: Uniplate,
> Biplate<To> for (T, U, V, W, X)
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
