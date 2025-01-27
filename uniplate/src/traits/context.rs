//! The underlying iterator for `Uniplate::context()`

use std::sync::Arc;

use crate::zipper::{Zipper, ZipperBi};

use super::{Biplate, Uniplate};

/// Iterator for `context`
pub(super) struct ContextIter<T: Uniplate> {
    zipper: Zipper<T>,
    done: bool,
}

impl<T: Uniplate> ContextIter<T> {
    pub(super) fn new(root: T) -> ContextIter<T> {
        ContextIter {
            zipper: Zipper::new(root),
            done: false,
        }
    }
}

impl<T: Uniplate> Iterator for ContextIter<T> {
    type Item = (T, Arc<dyn Fn(T) -> T>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        };
        let node = self.zipper.focus().clone();
        let zipper1 = self.zipper.clone();
        let hole_fn = Arc::new(move |x| {
            // TODO: if performance is still an issue, maybe we could make this a single call function
            // (FnOnce) then we wouldn't need to clone as much?
            let mut zipper2 = zipper1.clone();
            zipper2.replace_focus(x);
            zipper2.rebuild_root()
        });

        // prepare iterator for next element.
        // try moving down or right. if we can't move up the tree until we can move right.
        if self.zipper.go_down().is_none() {
            while self.zipper.go_right().is_none() {
                if self.zipper.go_up().is_none() {
                    // at the top again, so this will be the last time we return a node
                    self.done = true;
                    break;
                };
            }
        }

        Some((node, hole_fn))
    }
}

/// Iterator for `context_bi`
pub(super) struct ContextIterBi<T: Uniplate, U: Biplate<T>> {
    zipper: Option<ZipperBi<T, U>>,
    done: bool,
}

impl<T: Uniplate, U: Biplate<T>> ContextIterBi<T, U> {
    pub(super) fn new(root: U) -> ContextIterBi<T, U> {
        ContextIterBi {
            zipper: ZipperBi::new(root),
            done: false,
        }
    }
}

impl<T: Uniplate, U: Biplate<T>> Iterator for ContextIterBi<T, U> {
    type Item = (T, Arc<dyn Fn(T) -> U>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        };

        let Some(zipper) = &mut self.zipper else {
            return None;
        };

        let node = zipper.focus().clone();

        let zipper1 = zipper.clone();
        let hole_fn = Arc::new(move |x| {
            // TODO: if performance is still an issue, maybe we could make this a single call function
            // (FnOnce) then we wouldn't need to clone as much?
            let mut zipper1 = zipper1.clone();
            zipper1.replace_focus(x);
            zipper1.rebuild_root()
        });

        // prepare iterator for next element.
        // try moving down or right. if we can't move up the tree until we can move right.
        if zipper.go_down().is_none() {
            while zipper.go_right().is_none() {
                if zipper.go_up().is_none() {
                    // at the top again, so this will be the last time we return a node
                    self.done = true;
                    break;
                };
            }
        }

        Some((node, hole_fn))
    }
}
