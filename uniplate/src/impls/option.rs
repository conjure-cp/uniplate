//! Default Uniplate impl for Option<T>

use crate::Biplate;
use crate::Tree;
use crate::Uniplate;

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
