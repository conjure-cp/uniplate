use uniplate::{derive::Uniplate, Biplate};

#[derive(Eq, PartialEq, Clone, Debug, Uniplate)]
#[biplate(to=B)]
#[biplate(to=i32,walk_into=[B])]
#[uniplate(walk_into=[B])]
struct A {
    value: i32,
    children: Vec<B>,
}

#[derive(Eq, PartialEq, Clone, Debug, Uniplate)]
#[uniplate(walk_into=[A])]
#[biplate(to=i32, walk_into=[A])]
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
    let ints_in_a = <A as Biplate<i32>>::universe_bi(&a);
    assert_eq!(ints_in_a, vec![1, 2, 3]);

    // same type property
    let children = <A as Biplate<A>>::children_bi(&a);
    assert_eq!(children.len(), 1);
    assert_eq!(children[0], a);

    // test with_children_bi
    let children = <A as Biplate<i32>>::children_bi(&a);
    let reconstructed = <A as Biplate<i32>>::with_children_bi(&a, children);
    assert_eq!(reconstructed, a);
}
