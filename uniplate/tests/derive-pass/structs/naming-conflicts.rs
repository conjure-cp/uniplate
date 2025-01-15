use uniplate::derive::Uniplate;

#[derive(Uniplate, PartialEq, Eq, Clone)]
#[uniplate()]
struct NamingConflict {
    foo: i32,
    foo_copy: i32,
}

fn main() {}
