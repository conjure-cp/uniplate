use std::collections::VecDeque;
use uniplate::{derive::Uniplate, Biplate};

#[derive(Eq, PartialEq, Clone, Debug, Uniplate)]
#[uniplate()]
#[biplate(to=Vec<A>)]
struct A {
    children: Vec<A>,
}

pub fn main() {
    let inner_as = vec![
        A {
            children: vec![A { children: vec![] }],
        },
        A { children: vec![] },
    ];
    let a = A {
        children: inner_as.clone(),
    };

    let result: VecDeque<Vec<A>> = a.children_bi();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0], inner_as);
    assert_eq!(<Vec<A> as Biplate<Vec<A>>>::children_bi(&inner_as).len(), 1);
    assert_eq!(
        <Vec<A> as Biplate<Vec<A>>>::children_bi(&inner_as)[0],
        inner_as
    );
}
