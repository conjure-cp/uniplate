use uniplate::Uniplate;

#[derive(Uniplate,PartialEq,Eq,Clone)]
#[uniplate()]
enum EnumStruct {
    A {a: i32, b: Box<EnumStruct>, c: Vec<EnumStruct> },
    B (i32,i32,Box<EnumStruct>),
    C
}

pub fn main() {
    
}
