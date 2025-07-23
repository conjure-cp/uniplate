#![allow(dead_code)]
//! Using derive with a simple container type (a binary tree)

use std::collections::VecDeque;

// Using a mix of where clauses and type parameter bounds, to show that either works.
//
// Note that Uniplate does not walk into T here.
use uniplate::{Biplate, Uniplate};
#[derive(Eq, PartialEq, Uniplate, Clone)]
#[biplate(to=String, walk_into=[T])]
enum BinaryTree<T: PartialEq + Eq>
where
    T: Clone,
{
    Leaf(T),
    Branch(T, Box<BinaryTree<T>>, Box<BinaryTree<T>>),
}

#[derive(Eq, PartialEq, Clone, Uniplate)]
#[biplate(to=String)]
enum Foo {
    A(String),
    B(i32),
}

pub fn main() {
    let tree1: BinaryTree<Foo> = BinaryTree::Leaf(Foo::A("Hello".into()));
    let tree2: BinaryTree<Foo> = BinaryTree::Branch(
        Foo::A("World".into()),
        Box::new(tree1.clone()),
        Box::new(tree1.clone()),
    );
    let tree3: BinaryTree<Foo> = BinaryTree::Branch(
        Foo::A("Foo".into()),
        Box::new(tree1.clone()),
        Box::new(tree2.clone()),
    );

    let tree1_universe: VecDeque<String> = VecDeque::from(["Hello".into()]);
    let tree2_universe: VecDeque<String> =
        VecDeque::from(["World".into(), "Hello".into(), "Hello".into()]);

    let tree3_universe: VecDeque<String> = VecDeque::from([
        "Foo".into(),
        "Hello".into(),
        "World".into(),
        "Hello".into(),
        "Hello".into(),
    ]);

    assert_eq!(tree1_universe, Biplate::<String>::universe_bi(&tree1));
    assert_eq!(tree2_universe, Biplate::<String>::universe_bi(&tree2));
    assert_eq!(tree3_universe, Biplate::<String>::universe_bi(&tree3));
}
