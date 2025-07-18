use uniplate::Uniplate;

/// Fields are named to cause conflicts with derive macro internal variable naming
#[derive(Uniplate, PartialEq, Eq, Clone)]
#[uniplate()]
struct NamingConflict {
    foo: i32,
    foo_children: Vec<NamingConflict>,
    foo_copy: Box<NamingConflict>,
}

fn main() {}
