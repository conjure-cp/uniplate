use uniplate::derive::Uniplate;
#[derive(Clone, PartialEq, Eq, Uniplate)]
#[uniplate()]
enum A {
    B,
    C,
}
pub fn main() {}
