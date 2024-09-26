**Uniplate helps you write simple, boilerplate-free operations on tree shaped data types.**

A port of Haskell's [Uniplate](https://hackage.haskell.org/package/uniplate) in
Rust.

---

# Getting Started 

*Adapted from (Mitchell and Runciman 2009)*

Consider the abstract syntax tree for a simple calculator language:

```rust
enum Expr {
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Val(i32),
    Var(String),
    Neg(Box<Expr>),
}
```

Say we want to list all the used variable names inside a given expression:

```rust
# use uniplate::test_common::paper::Expr::*;
# use uniplate::test_common::paper::Expr;
fn vars(expr: &Expr) -> Vec<String>{
    match expr {
        Add(a,b) => {
            [vars(a),vars(b)].concat()
        },
        Sub(a,b) => {
            [vars(a),vars(b)].concat()
        },
        Mul(a,b) => {
            [vars(a),vars(b)].concat()
        },
        Div(a,b) => {
            [vars(a),vars(b)].concat()
        },
        Val(a) => {
            Vec::new()
        },
        Var(a) => {
            vec![a.clone()]
        },
        Neg(a) =>{
            vars(a)
        }
    }
}
```

Functions like these are annoying to write: the first 4 constructors are basically identical,
adding a new expression type requires a new line to be added to all match statement, and this
code cannot be shared with similar functions (e.g. one that change all the variable names).


With Uniplate, this boilerplate can be eliminated:

```rust
# use uniplate::test_common::paper::Expr::*;
# use uniplate::test_common::paper::Expr;
use uniplate::Biplate;
fn vars(expr: &Expr) -> Vec<String>{
    <Expr as Biplate<String>>::universe_bi(expr).into_iter().collect()
}
```

The functionality of Uniplate comes from two main traits: [`Uniplate`](Uniplate) and
[`Biplate<T>`](Biplate).

* The [`Uniplate`](Uniplate) of `Expr` operates over all nested `Expr`s.
* The [`Biplate<T>`](Biplate) of `Expr` operates over all nested values of some given type `T` within the
  expression tree.

These traits provide traversal operations (e.g. [`children`](Uniplate::children)) as well as
functional programming constructs such as [`map`](Uniplate::map) and [`fold`](Uniplate::fold).
See the trait documentation for the full list of operations provided.

The easiest way to use Uniplate is with the derive macro.

## Derive Macro

When no arguments are provided, the macro derives a Uniplate instance:

```rust
use uniplate::derive::Uniplate;
#[derive(Clone,PartialEq,Eq,Debug,Uniplate)]
enum Expr {
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Val(i32),
    Var(String),
    Neg(Box<Expr>),
}
```

To derive Biplate instances, use the `#[biplate]` attribute:

```rust
use uniplate::derive::Uniplate;
#[derive(Clone,PartialEq,Eq,Debug,Uniplate)]
#[biplate(to=String)]
#[biplate(to=i32)]
enum Expr {
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Val(i32),
    Var(String),
    Neg(Box<Expr>),
}
```

## Multi-type traversals

Lets expand our calculator language to include statements as well as expressions:

```rust
enum Expr {
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Val(i32),
    Var(String),
    Neg(Box<Expr>),
}

enum Stmt {
    Assign(String, Expr),
    Sequence(Vec<Stmt>),
    If(Expr, Box<Stmt>, Box<Stmt>),
    While(Expr, Box<Stmt>),
}
```

When looking for `Strings` in a given `Stmt`, we may want to identify not only the strings
directly contained within a `Stmt`, but also any strings contained inside an `Expr` inside a
`Stmt`.

For example:

```rust
# use uniplate::test_common::paper::Expr::*;
# use uniplate::test_common::paper::Expr::*;
# use uniplate::test_common::paper::Stmt;
# use uniplate::test_common::paper::Stmt::*;
use uniplate::{Biplate,Uniplate};
use uniplate::derive::Uniplate;

let stmt = Assign("x".into(), Add(Box::new(Var("y".into())),Box::new(Val(10))));
let strings = <Stmt as Biplate<String>>::universe_bi(&stmt);

assert!(strings.contains(&"x".into()));

// Despite being inside an Expr::String, "y" is found by Biplate
assert!(strings.contains(&"y".into()));

assert_eq!(strings.len(), 2);
```


To do this, a list of types to "walk into" can be given as an argument to the Biplate
declaration:
```rust
use uniplate::Uniplate;
use uniplate::derive::Uniplate;
#[derive(Clone,PartialEq,Eq,Debug,Uniplate)]
#[uniplate()]
#[biplate(to=String,walk_into=[Expr])]
#[biplate(to=Stmt)]
enum Expr {
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Val(i32),
    Var(String),
    Neg(Box<Expr>),
}

// Uniplate also supports walk_into.
// In this case, it doesn't do much.
#[derive(Clone,PartialEq,Eq,Debug,Uniplate)]
#[biplate(to=String, walk_into=[Expr])]
#[biplate(to=Expr,walk_into=[Expr])]
#[uniplate(walk_into=[Expr])]
enum Stmt {
    Assign(String, Expr),
    Sequence(Vec<Stmt>),
    If(Expr, Box<Stmt>, Box<Stmt>),
    While(Expr, Box<Stmt>),
}
```


# Bibliography

The techniques implemented in this crate originate from the following:
 
* [The Uniplate Haskell Library](https://hackage.haskell.org/package/uniplate).

* Neil Mitchell and Colin Runciman. 2007. Uniform boilerplate and list processing. In
Proceedings of the ACM SIGPLAN workshop on Haskell workshop (Haskell '07). Association for
Computing Machinery, New York, NY, USA, 49–60. <https://doi.org/10.1145/1291201.1291208>
[(free copy)](https://www.cs.york.ac.uk/plasma/publications/pdf/MitchellRuncimanHW07.pdf)

* Huet G. The Zipper. Journal of Functional Programming. 1997;7(5):549–54. <https://doi.org/10.1017/S0956796897002864>
[(free copy)](https://www.cambridge.org/core/services/aop-cambridge-core/content/view/0C058890B8A9B588F26E6D68CF0CE204/S0956796897002864a.pdf/zipper.pdf)
