//! Test case for issue #16.

#![allow(dead_code)]

use uniplate::derive::Uniplate;
use uniplate::Biplate;

#[derive(Eq, PartialEq, Clone, Debug, Uniplate)]
#[uniplate()]
#[biplate(to=Vec<Stmt>,walk_into=[Expr])]
enum Stmt {
    Nothing,
    Assign(String, Expr),
    Sequence(Vec<Stmt>),
    If(Expr, Box<Stmt>, Box<Stmt>),
    While(Expr, Box<Stmt>),
    Return(String),
}

#[derive(Eq, PartialEq, Clone, Debug, Uniplate)]
#[uniplate()]
#[biplate(to=Vec<Stmt>)]
enum Expr {
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Val(i32),
    Var(String),
    Neg(Box<Expr>),
}

pub fn main() {
    use Expr::*;
    use Stmt::*;

    let inner_stmts = vec![Assign("x".into(),Val(1)), Return("x".into())];
    let stmt = Sequence(inner_stmts.clone());

    let result = <_ as Biplate<Vec<Stmt>>>::children_bi(&stmt).into_iter().collect::<Vec<_>>();

    assert_eq!(result.len(),1);
    assert_eq!(result[0],inner_stmts);
    assert_eq!(<Vec<Stmt> as Biplate<Vec<Stmt>>>::children_bi(&inner_stmts).len(),1);
    assert_eq!(<Vec<Stmt> as Biplate<Vec<Stmt>>>::children_bi(&inner_stmts)[0],inner_stmts);
}
