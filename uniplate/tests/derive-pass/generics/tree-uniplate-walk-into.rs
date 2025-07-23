#![allow(dead_code)]
//! Using derive with a simple container type (a binary tree)

// Using a mix of where clauses and type parameter bounds, to show that either works.
//
use uniplate::Uniplate;
#[derive(Eq, PartialEq, Uniplate, Clone)]
#[uniplate(walk_into=[T])]
enum BinaryTree<T: PartialEq + Eq>
where
    T: Ord,
    T: Clone,
{
    Leaf(T),
    Branch(T, Box<BinaryTree<T>>, Box<BinaryTree<T>>),
}

pub fn main() {}
