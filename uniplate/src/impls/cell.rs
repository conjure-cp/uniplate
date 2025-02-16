//! Implementations for std::cell types.

use crate::{Biplate, Tree, Uniplate};
use std::cell::RefCell;

// do not walk into refcells looking for other refcells.
impl<T: Clone + Eq + Uniplate> Uniplate for RefCell<T> {
    fn uniplate(&self) -> (crate::Tree<Self>, Box<dyn Fn(crate::Tree<Self>) -> Self>) {
        let self2 = self.clone();
        (Tree::Zero, Box::new(move |_| self2.clone()))
    }
}

/// Implementation of `Biplate` for `RefCell`.
///
/// Rc<RefCell<T>> is a special case dealt with in the `Rc` implementation: this modifies the data
/// in-place, maintaining shared mutability.
impl<To, From> Biplate<To> for RefCell<From>
where
    To: Clone + Eq + Uniplate,
    From: Clone + Eq + Uniplate + Biplate<To>,
{
    fn biplate(&self) -> (Tree<To>, Box<dyn Fn(Tree<To>) -> Self>) {
        let self2 = self.clone();

        if std::any::TypeId::of::<To>() == std::any::TypeId::of::<RefCell<From>>() {
            // Biplate<RefCell<T>> for RefCell<T> returns self.
            //
            // SAFETY: This branch checked that To === RefCell<From>. Therefore, self is also of
            // type To.
            unsafe {
                let self2_to = std::mem::transmute::<&RefCell<From>, &To>(self).clone();
                (Tree::One(self2_to), Box::new(move |_| self2.clone()))
            }
        } else {
            // Unwrap RefCell, call Biplate<To> on From, and reconstruct.

            let inner: From = self.borrow().clone();

            // (Tree<To>, context function to reconstruct value of type `From`)
            let (tree, inner_ctx) = <From as Biplate<To>>::biplate(&inner);

            (tree, Box::new(move |x| RefCell::new(inner_ctx(x))))
        }
    }
}
