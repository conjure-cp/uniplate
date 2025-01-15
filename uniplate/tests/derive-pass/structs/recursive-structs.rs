use uniplate::derive::Uniplate;

#[derive(Uniplate, PartialEq, Eq, Clone)]
#[uniplate()]
struct Tree {
    value: i32,
    children: Vec<Tree>,
}

fn main() {}
