use uniplate::derive::Uniplate;

#[derive(Uniplate, PartialEq, Eq, Clone)]
#[uniplate()]
struct NamingConflict {
    foo: i32,
    foo_children: Vec<NamingConflict>,
    foo_copy: Box<NamingConflict>,
}

fn main() {}
