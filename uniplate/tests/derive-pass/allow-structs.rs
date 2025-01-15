use uniplate::derive::Uniplate;

#[derive(Uniplate,PartialEq,Eq,Clone)]
#[uniplate()]
struct UnitLike;

#[derive(Uniplate,PartialEq,Eq,Clone)]
#[uniplate()]
struct Simple(i32);

#[derive(Uniplate,PartialEq,Eq,Clone)]
#[uniplate()]
struct Tree {
    value: i32,
    children: Vec<Tree>,
}

fn main() {}
