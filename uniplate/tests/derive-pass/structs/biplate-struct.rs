use std::collections::VecDeque;
use uniplate::{Uniplate, Biplate};

#[derive(Eq, PartialEq, Clone, Debug, Uniplate)]
#[biplate(to=B)]
#[biplate(to=i32,)]
#[uniplate()]
struct A {
    value: i32,
    children: Vec<B>,
}

#[derive(Eq, PartialEq, Clone, Debug, Uniplate)]
#[uniplate()]
#[biplate(to=i32)]
#[biplate(to=A)]
struct B {
    value: i32,
    child: A,
}

pub fn main() {
    let a = A {
        value: 1,
        children: vec![B {
            value: 2,
            child: A {
                value: 3,
                children: vec![],
            },
        }],
    };

    // Test multi-type traversals
    let ints_in_a: VecDeque<i32> = a.universe_bi();
    assert_eq!(ints_in_a, vec![1, 2, 3]);

    // same type property
    let children: VecDeque<A> = a.children_bi();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0], a);

    // test with_children_bi
    let children: VecDeque<i32> = a.children_bi();
    let reconstructed: A = a.with_children_bi(children);
    assert_eq!(reconstructed, a);
}
