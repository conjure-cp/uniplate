use uniplate::Uniplate;

#[derive(Uniplate, PartialEq, Eq, Clone)]
#[uniplate()]
struct Tuple(i32, String);

fn main() {}
