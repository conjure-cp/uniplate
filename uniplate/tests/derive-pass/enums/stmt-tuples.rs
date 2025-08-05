#![allow(dead_code)]

// Like biplate-stmt, but some fields of Expr are boxed tuples.

use std::collections::VecDeque;
use uniplate::{Biplate, Uniplate};

#[derive(Eq, PartialEq, Clone, Debug, Uniplate)]
#[biplate(to=Expr)]
#[biplate(to=i32)]
#[biplate(to=String)]
#[biplate(to=Option<Expr>)]
#[uniplate()]
enum Stmt {
    Assign((String, Option<Expr>)),
    Sequence(Vec<Stmt>),
    If(Expr, Box<Stmt>, Box<Stmt>),
    While(Expr, Box<Stmt>),
}

#[derive(Eq, PartialEq, Clone, Debug, Uniplate)]
#[uniplate()]
#[biplate(to=String)]
#[biplate(to=(Expr,Expr))]
#[biplate(to=Option<Expr>)]
#[biplate(to=Stmt)]
#[biplate(to=i32)]
enum Expr {
    Add(Box<(Expr, Expr)>),
    Sub(Box<(Expr, Expr)>),
    Mul(Box<(Expr, Expr)>),
    Div(Box<(Expr, Expr)>),
    Val(i32),
    Var(String),
    Neg(Box<Expr>),
}

pub fn main() {
    use Expr::*;
    use Stmt::*;

    let expr_1 = (Val(2), Var("y".into()));
    let strings_in_expr_1: VecDeque<String> = expr_1.universe_bi();
    assert_eq!(1, strings_in_expr_1.len());

    let expr_2 = Div(Box::new((Val(2), Var("y".into()))));
    let strings_in_expr_2: VecDeque<String> = expr_2.universe_bi();
    assert_eq!(1, strings_in_expr_2.len());

    let expr_3 = Some(Div(Box::new((Val(2), Var("y".into())))));
    let strings_in_expr_3: VecDeque<String> = expr_3.universe_bi();
    assert_eq!(1, strings_in_expr_3.len());

    let stmt_1 = Assign(("x".into(), Some(Div(Box::new((Val(2), Var("y".into())))))));
    let strings_in_stmt_1: VecDeque<String> = stmt_1.universe_bi();

    // Test multi-type traversals
    assert_eq!(strings_in_stmt_1.len(), 2);
    assert!(strings_in_stmt_1.contains(&"x".into()));
    assert!(strings_in_stmt_1.contains(&"y".into()));

    // same type property
    let children: VecDeque<Stmt> = stmt_1.children_bi();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0], stmt_1);

    // test with_children_bi
    let children: VecDeque<String> = stmt_1.children_bi();
    let reconstructed: Stmt = stmt_1.with_children_bi(children);
    assert_eq!(reconstructed, stmt_1);

    // test descend_bi on ints
    let stmt_1_ints = VecDeque::from([2]);

    assert_eq!(stmt_1.children_bi(), stmt_1_ints);

    let stmt_1_expected = Assign(("x".into(), Some(Div(Box::new((Val(3), Var("y".into())))))));
    assert_eq!(
        stmt_1.with_children_bi(VecDeque::from([3])),
        stmt_1_expected
    );

    let stmt_1_actual = stmt_1.descend_bi(&|x: i32| x + 1);
    assert_eq!(stmt_1_expected, stmt_1_actual);

    // test transform_bi
    let stmt_1_actual = stmt_1.transform_bi(&|x: i32| x + 1);

    assert_eq!(stmt_1_expected, stmt_1_actual);
}
