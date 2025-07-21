use uniplate::zipper::Zipper;
use uniplate_derive::Uniplate;

#[derive(Clone, PartialEq, Eq, Debug, Uniplate)]
enum Tree {
    None,
    Leaf(i32),
    One(i32, Box<Tree>),
    Many(i32, Vec<Tree>),
}

impl Tree {
    fn value(&self) -> i32 {
        match self {
            Tree::None => panic!("Cannot get value from None"),
            Tree::Leaf(v) => *v,
            Tree::One(v, _) => *v,
            Tree::Many(v, _) => *v,
        }
    }
}

#[test]
fn zipper_up_from_root() {
    let mut zipper = Zipper::new(Tree::None);
    assert!(zipper.go_up().is_none());
}

#[test]
fn zipper_up_from_branch() {
    let mut zipper = Zipper::new(Tree::Many(1, vec![Tree::None, Tree::None]));
    zipper.go_down();
    assert!(zipper.go_up().is_some());
}

#[test]
fn zipper_iter_left_siblings() {
    let mut zipper = Zipper::new(Tree::Many(0, (1..6).map(|i| Tree::Leaf(i)).collect()));

    zipper.go_down();
    assert!(zipper.iter_left_siblings().next().is_none());

    zipper.go_right();
    assert!(zipper.iter_left_siblings().map(Tree::value).eq(1..2));

    while zipper.go_right().is_some() {}
    assert!(zipper.iter_left_siblings().map(Tree::value).eq(1..5));
}

#[test]
fn zipper_iter_right_siblings() {
    let mut zipper = Zipper::new(Tree::Many(0, (1..6).map(|i| Tree::Leaf(i)).collect()));

    zipper.go_down();
    assert!(zipper.iter_right_siblings().map(Tree::value).eq(2..6));

    zipper.go_right();
    assert!(zipper.iter_right_siblings().map(Tree::value).eq(3..6));

    while zipper.go_right().is_some() {}
    assert!(zipper.iter_right_siblings().next().is_none());
}

#[test]
fn zipper_iter_siblings() {
    let mut zipper = Zipper::new(Tree::Many(0, (1..6).map(|i| Tree::Leaf(i)).collect()));

    zipper.go_down();
    zipper.go_right();
    zipper.go_right();

    assert!(zipper.iter_siblings().map(Tree::value).eq(1..6));
}

#[test]
fn zipper_iter_ancestors_empty() {
    let zipper = Zipper::new(Tree::None);
    assert!(zipper.iter_ancestors().next().is_none());
}

#[test]
fn zipper_iter_ancestors() {
    let mut zipper = Zipper::new((1..6).fold(Tree::None, |acc, i| Tree::One(i, Box::new(acc))));

    while zipper.go_down().is_some() {}

    assert!(zipper.iter_ancestors().map(|t| Tree::value(&t)).eq(1..6));
}

#[test]
fn zipper_iter_ancestors_mutate() {
    let mut zipper = Zipper::new(Tree::One(0, Box::new(Tree::One(1, Box::new(Tree::None)))));

    zipper.go_down();

    zipper.replace_focus(Tree::None); // Should now appear in parent's subtree

    assert_eq!(
        zipper.iter_ancestors().next().unwrap(),
        Tree::One(0, Box::new(Tree::None))
    );
}
