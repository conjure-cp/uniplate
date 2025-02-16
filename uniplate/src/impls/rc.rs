//! Implementations for std::rc types.

use std::rc::Rc;
use std::cell::RefCell;
use crate::{Tree, Uniplate,Biplate};

// do not walk into Rcs looking for other Rcs.
impl<T: Clone + Eq + Uniplate> Uniplate for Rc<T> {
    fn uniplate(&self) -> (crate::Tree<Self>, Box<dyn Fn(crate::Tree<Self>) -> Self>) {
        let self2 = self.clone();
        (Tree::Zero, Box::new(move |_| self2.clone()))
    }
}

// TODO: semantics of this are weird.
// Also means that we cannot impl Biplate<To> for Rc<From>.

/// Implementation of `Biplate` for `Rc<RefCell<T>>`.
///
/// This modifies the data in-place, maintaining shared mutability. Due to this behaviour, this
/// implementation is locked behind a feature flag.
#[cfg(feature = "rc-refcell")]
impl<To, From> Biplate<To> for Rc<RefCell<From>>
where
    To: Clone + Eq + Uniplate,
    From: Clone + Eq + Uniplate + Biplate<To>,
{
    fn biplate(&self) -> (Tree<To>, Box<dyn Fn(Tree<To>) -> Self>) {
        let self2 = self.clone();

        if std::any::TypeId::of::<To>() == std::any::TypeId::of::<Rc<RefCell<From>>>() {
            // Biplate<Rc<RefCell<T>> for Rc<RefCell<<T>> returns self.
            //
            // SAFETY: This branch checked that To === Rc<RefCell<From>>. Therefore, self is also of
            // type To.
            unsafe {
                let self2_to = std::mem::transmute::<&Rc<RefCell<From>>, &To>(self).clone();
                (Tree::One(self2_to), Box::new(move |_| self2.clone()))
            }
        } else {
            // Unwrap Rc<RefCell<From>>, call Biplate<To> on From, and reconstruct.

            let inner: From = self.borrow().clone();

            // (Tree<To>, context function to reconstruct value of type `From`)
            let (tree, inner_ctx) = <From as Biplate<To>>::biplate(&inner);

            (
                tree,
                Box::new(move |x| {
                    let self3 = self2.clone();
                    *self3.borrow_mut() = inner_ctx(x);
                    self3
                }),
            )
        }
    }
}
