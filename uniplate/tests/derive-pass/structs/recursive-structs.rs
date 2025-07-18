use uniplate::Uniplate;

#[derive(Uniplate, PartialEq, Eq, Clone)]
#[uniplate()]
struct Tree {
    value: i32,
    children: Vec<Tree>,
}

fn main() {
    let x = Tree {
        value: 0,
        children: vec![
            Tree {
                value: 1,
                children: vec![
                    Tree {
                        value: 2,
                        children: vec![],
                    },
                    Tree {
                        value: 3,
                        children: vec![],
                    },
                ],
            },
            Tree {
                value: 4,
                children: vec![Tree {
                    value: 5,
                    children: vec![Tree {
                        value: 6,
                        children: vec![],
                    }],
                }],
            },
        ],
    };

    assert_eq!(
        (0..7).collect::<Vec<_>>(),
        x.universe().iter().map(|x| x.value).collect::<Vec<_>>()
    );
}
