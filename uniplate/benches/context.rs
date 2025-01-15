//! Benchmarks for `context`,`context_bi`

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use uniplate::{derive::Uniplate, Uniplate};

#[derive(PartialEq, Eq, Clone, Uniplate)]
#[uniplate()]
enum MyEnum {
    A(Box<MyEnum>, Box<MyEnum>),
    B(String, Box<MyEnum>, String, Box<MyEnum>, Box<MyEnum>),
    C(Vec<MyEnum>),
    D(String),
}

/// Big benchmark for `context` using a derived Uniplate instance on an enum
fn benchmark_context_derived_enum_big(c: &mut Criterion) {
    let tree = generate_child(1, 3, 0, 10);
    c.bench_function("context enum big", |b| {
        b.iter(|| walk_ctx(black_box(&tree)))
    });
}

/// Small benchmark for `context` using a derived Uniplate instance on an enum
fn benchmark_context_derived_enum_small(c: &mut Criterion) {
    let tree = generate_child(1, 2, 0, 2);
    c.bench_function("context enum small", move |b| {
        b.iter(|| walk_ctx(black_box(&tree)))
    });
}

// deterministically make some an tree structure
fn generate_child(
    n: i32,
    max_children_per_node: i32,
    current_depth: i32,
    max_depth: i32,
) -> MyEnum {
    if current_depth >= max_depth {
        return MyEnum::D("reached max depth".into());
    }

    if (n + current_depth) % 2 == 0 {
        let mut children = vec![];
        for i in 1..max_children_per_node {
            children.push(generate_child(
                i,
                max_children_per_node,
                current_depth + 1,
                max_depth,
            ))
        }

        MyEnum::C(children)
    } else if (n + current_depth) % 3 == 0 {
        MyEnum::A(
            Box::new(generate_child(
                1,
                max_children_per_node,
                current_depth + 1,
                max_depth,
            )),
            Box::new(generate_child(
                2,
                max_children_per_node,
                current_depth + 1,
                max_depth,
            )),
        )
    } else if (n + current_depth) % 5 == 0 {
        MyEnum::D("hello".into())
    } else {
        MyEnum::B(
            "hello".into(),
            Box::new(generate_child(
                1,
                max_children_per_node,
                current_depth + 1,
                max_depth,
            )),
            "world".into(),
            Box::new(generate_child(
                3,
                max_children_per_node,
                current_depth + 1,
                max_depth,
            )),
            Box::new(generate_child(
                4,
                max_children_per_node,
                current_depth + 1,
                max_depth,
            )),
        )
    }
}

fn walk_ctx(e: &MyEnum) -> &MyEnum {
    for (e1, c) in e.contexts() {
        black_box(e1.clone());
        black_box(c.clone());
        c(e1); // use context to benchmark it too
    }
    black_box(e)
}

criterion_group!(
    benches,
    benchmark_context_derived_enum_big,
    benchmark_context_derived_enum_small
);
criterion_main!(benches);
