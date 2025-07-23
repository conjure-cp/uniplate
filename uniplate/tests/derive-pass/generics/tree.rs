#![allow(dead_code)]
//! Using derive with a simple container type (a binary tree)

// Using a mix of where clauses and type parameter bounds, to show that either works.
//
// Note that Uniplate does not walk into T here.
use uniplate::Uniplate;
#[derive(Eq, PartialEq, Uniplate, Clone)]
#[uniplate()]
enum BinaryTree<T: PartialEq + Eq>
where
    T: Ord,
    T: Clone,
{
    Leaf(T),
    Branch(T, Box<BinaryTree<T>>, Box<BinaryTree<T>>),
}

pub fn main() {}
