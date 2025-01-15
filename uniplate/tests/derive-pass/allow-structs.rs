use uniplate::derive::Uniplate;

#[derive(Uniplate, PartialEq, Eq, Clone)]
#[uniplate()]
struct UnitLike;

#[derive(Uniplate, PartialEq, Eq, Clone)]
#[uniplate()]
struct Simple(i32);

#[derive(Uniplate, PartialEq, Eq, Clone)]
#[uniplate()]
struct Tree {
    value: i32,
    children: Vec<Tree>,
}

#[derive(Uniplate, PartialEq, Eq, Clone)]
#[uniplate()]
struct NamingConflict {
    foo: i32,
    foo_copy: i32,
}

fn main() {}
