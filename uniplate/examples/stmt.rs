#![allow(dead_code)]

use std::collections::VecDeque;

use uniplate::Biplate;
use uniplate::Uniplate;

#[derive(Eq, PartialEq, Clone, Debug, Uniplate)]
#[uniplate()]
#[biplate(to=String)]
enum Stmt {
    Nothing,
    Assign(String, Expr),
    Sequence(Vec<Stmt>),
    If(Expr, Box<Stmt>, Box<Stmt>),
    While(Expr, Box<Stmt>),
}

#[derive(Eq, PartialEq, Clone, Debug, Uniplate)]
#[uniplate()]
#[biplate(to=String)]
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

    let stmt_1 = Assign("x".into(), Div(Box::new(Val(2)), Box::new(Var("y".into()))));

    let strings_in_stmt_1: VecDeque<String> = stmt_1.universe_bi();

    // Test multi-type traversals
    assert_eq!(strings_in_stmt_1.len(), 2);
    assert!(strings_in_stmt_1.contains(&"x".into()));
    assert!(strings_in_stmt_1.contains(&"y".into()));
}
