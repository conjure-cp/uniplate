use uniplate::Uniplate;

#[derive(Clone, Debug, PartialEq, Eq, Uniplate)]
#[uniplate()]
enum Expr {
    Val(i32),
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sum(Vec<Expr>),
}

#[test]
fn replaces_boxed_child_in_place() {
    let mut expr = Expr::Neg(Box::new(Expr::Val(1)));
    assert!(expr.try_replace_child_at(0, Expr::Val(9)));
    assert_eq!(expr, Expr::Neg(Box::new(Expr::Val(9))));
    assert!(!expr.try_replace_child_at(1, Expr::Val(0)));
}

#[test]
fn replaces_vec_child_without_changing_siblings() {
    let mut expr = Expr::Sum(vec![Expr::Val(0), Expr::Val(1), Expr::Val(2)]);
    assert!(expr.try_replace_child_at(1, Expr::Val(99)));
    assert_eq!(
        expr,
        Expr::Sum(vec![Expr::Val(0), Expr::Val(99), Expr::Val(2)])
    );
}

#[test]
fn replaces_binary_child() {
    let mut expr = Expr::Add(Box::new(Expr::Val(1)), Box::new(Expr::Val(2)));
    assert!(expr.try_replace_child_at(1, Expr::Val(7)));
    assert_eq!(
        expr,
        Expr::Add(Box::new(Expr::Val(1)), Box::new(Expr::Val(7)))
    );
}
