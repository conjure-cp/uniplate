use std::{collections::VecDeque, sync::Arc};

use super::{Biplate, Uniplate};

pub(super) struct HolesIter<T: Uniplate> {
    children_iter: std::iter::Enumerate<std::collections::vec_deque::IntoIter<T>>,
    children: VecDeque<T>,
    parent: T,
}

impl<T: Uniplate> Iterator for HolesIter<T> {
    type Item = (T, Arc<dyn Fn(T) -> T>);

    fn next(&mut self) -> Option<Self::Item> {
        let (i, child) = self.children_iter.next()?;

        let children2 = self.children.clone();
        let parent2 = self.parent.clone();
        let hole = Arc::new(move |x: T| {
            let mut children = children2.clone();
            children[i] = x;
            parent2.with_children(children)
        });

        Some((child.clone(), hole))
    }
}

impl<T: Uniplate> HolesIter<T> {
    pub(super) fn new(parent: T) -> HolesIter<T> {
        let children = parent.children();
        let children_iter = children.clone().into_iter().enumerate();

        HolesIter {
            children_iter,
            children,
            parent,
        }
    }
}

pub(super) struct HolesIterBi<T: Uniplate, F: Biplate<T>> {
    children_iter: std::iter::Enumerate<std::collections::vec_deque::IntoIter<T>>,
    children: VecDeque<T>,
    parent: F,
}

impl<T: Uniplate, F: Biplate<T>> Iterator for HolesIterBi<T, F> {
    type Item = (T, Arc<dyn Fn(T) -> F>);

    fn next(&mut self) -> Option<Self::Item> {
        let (i, child) = self.children_iter.next()?;

        let children2 = self.children.clone();
        let parent2 = self.parent.clone();
        let hole = Arc::new(move |x: T| {
            let mut children = children2.clone();
            children[i] = x;
            parent2.with_children_bi(children)
        });

        Some((child.clone(), hole))
    }
}

impl<T: Uniplate, F: Biplate<T>> HolesIterBi<T, F> {
    pub(super) fn new(parent: F) -> HolesIterBi<T, F> {
        let children = parent.children_bi();
        let children_iter = children.clone().into_iter().enumerate();

        HolesIterBi {
            children_iter,
            children,
            parent,
        }
    }
}
