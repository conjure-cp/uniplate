use uniplate::{Uniplate, tagged_zipper::TaggedZipper};

#[derive(Debug, Clone, PartialEq, Eq, Uniplate)]
enum Tree {
    Leaf,
    One(Box<Tree>),
    Many(Vec<Tree>),
}

#[test]
fn tagged_zipper_root() {
    let zipper = TaggedZipper::new(Tree::Leaf, |_| 1);
    assert_eq!(*zipper.tag(), 1);
}

#[test]
fn tagged_zipper_move_down_up() {
    let mut zipper = TaggedZipper::new(Tree::One(Box::new(Tree::Leaf)), |t| match t {
        Tree::Leaf => 1,
        Tree::One(_) => 2,
        Tree::Many(_) => 3,
    });

    assert_eq!(*zipper.tag(), 2);
    zipper.go_down().unwrap();
    assert_eq!(*zipper.tag(), 1);
    zipper.go_up().unwrap();
    assert_eq!(*zipper.tag(), 2);
}

#[test]
fn tagged_zipper_come_back_up_to_mutated() {
    let mut zipper = TaggedZipper::new(Tree::One(Box::new(Tree::Leaf)), |t| match t {
        Tree::Leaf => 1,
        Tree::One(_) => 2,
        Tree::Many(_) => 3,
    });

    *zipper.tag_mut() = 42;
    assert_eq!(*zipper.tag(), 42);

    zipper.go_down().unwrap();
    *zipper.tag_mut() = 43;
    assert_eq!(*zipper.tag(), 43);

    zipper.go_up().unwrap();
    assert_eq!(*zipper.tag(), 42);

    zipper.go_down().unwrap();
    assert_eq!(*zipper.tag(), 43);
}

#[test]
fn tagged_zipper_come_back_left_to_mutated() {
    let mut zipper = TaggedZipper::new(Tree::Many(vec![Tree::Leaf, Tree::Leaf]), |t| match t {
        Tree::Leaf => 1,
        Tree::One(_) => 2,
        Tree::Many(_) => 3,
    });

    zipper.go_down().unwrap();
    *zipper.tag_mut() = 42;
    assert_eq!(*zipper.tag(), 42);

    zipper.go_right().unwrap();
    *zipper.tag_mut() = 43;
    assert_eq!(*zipper.tag(), 43);

    zipper.go_left().unwrap();
    assert_eq!(*zipper.tag(), 42);

    zipper.go_right().unwrap();
    assert_eq!(*zipper.tag(), 43);
}

#[test]
fn tagged_zipper_replace_focus() {
    let mut zipper = TaggedZipper::new(Tree::Leaf, |t| match t {
        Tree::Leaf => 1,
        Tree::One(_) => 2,
        Tree::Many(_) => 3,
    });

    assert_eq!(*zipper.tag(), 1);
    zipper.replace_focus(Tree::One(Box::new(Tree::Leaf)));
    assert_eq!(*zipper.tag(), 2);
    *zipper.tag_mut() = 42;

    zipper.go_down().unwrap();
    assert_eq!(*zipper.tag(), 1);

    zipper.replace_focus(Tree::Many(vec![Tree::Leaf, Tree::Leaf]));
    assert_eq!(*zipper.tag(), 3);

    zipper.go_up().unwrap();
    assert_eq!(*zipper.tag(), 42);
}

#[test]
fn tagged_zipper_reset_tag() {
    let mut zipper = TaggedZipper::new(Tree::Leaf, |_| 1);
    assert_eq!(*zipper.tag(), 1);
    zipper.replace_tag(42);
    assert_eq!(*zipper.tag(), 42);
    zipper.reset_tag();
    assert_eq!(*zipper.tag(), 1);
}

#[test]
fn tagged_zipper_invalidate_subtree() {
    let mut zipper = TaggedZipper::new(Tree::One(Box::new(Tree::Leaf)), |_| 1);

    zipper.go_down().unwrap();
    assert_eq!(*zipper.tag(), 1);
    zipper.replace_tag(42);

    zipper.go_up().unwrap();
    zipper.invalidate_subtree();

    zipper.go_down().unwrap();
    assert_eq!(*zipper.tag(), 1);
}
