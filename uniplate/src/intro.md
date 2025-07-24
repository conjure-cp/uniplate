**Uniplate helps you write simple, boilerplate-free operations on tree shaped data types.**

- A port of Haskell's [Uniplate](https://hackage.haskell.org/package/uniplate)
  library in Rust.

# Getting Started 

*Adapted from (Mitchell and Runciman 2007)*

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
fn var_names(expr: &Expr) -> Vec<String>{
    match expr {
        Add(a,b) => {
            [var_names(a),var_names(b)].concat()
        },
        Sub(a,b) => {
            [var_names(a),var_names(b)].concat()
        },
        Mul(a,b) => {
            [var_names(a),var_names(b)].concat()
        },
        Div(a,b) => {
            [var_names(a),var_names(b)].concat()
        },
        Val(a) => {
            Vec::new()
        },
        Var(a) => {
            vec![a.clone()]
        },
        Neg(a) =>{
            var_names(a)
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
use std::collections::VecDeque;
use uniplate::Biplate;
fn var_names(expr: &Expr) -> Vec<String>{
    let names: VecDeque<String> = expr.universe_bi();
    names.into()
}
```

The functionality of Uniplate comes from two main traits: [`Uniplate`] and
[`Biplate<T>`].

* The [`Uniplate`] of `Expr` operates over all nested `Expr`s.
* The [`Biplate<T>`] of `Expr` operates over all nested values of type
  `T` in the expression tree.

These traits provide traversal operations (e.g. [`children`](Uniplate::children)) as well as
functional programming constructs such as `map` ([`transform`](Uniplate::transform), [`descend`](Uniplate::descend)) and `fold`([`cata`](Uniplate::cata).

See the trait documentation for the full list of operations provided.

The easiest way to use Uniplate is with the derive macro.

## Derive Macro

To derive Uniplate instances, use the `#[uniplate]` attribute:

```rust
use uniplate::Uniplate;
#[derive(Clone,PartialEq,Eq,Debug,Uniplate)]
#[uniplate()]
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
use uniplate::Uniplate;
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

Uniplate also supports trees with multiple nested types. Lets extend our
calculator language to include statements as well as expressions:

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

When looking for variable names in a given statement, we want to identify not
only the variable names directly used inside the statement, but also any
variable names used by child expressions:

```rust
use uniplate::{Uniplate,Biplate};
use std::collections::VecDeque;
#[derive(Clone,PartialEq,Eq,Debug,Uniplate)]
#[biplate(to=String)]
#[biplate(to=Expr)]
#[uniplate()]
enum Stmt {
    Assign(String, Expr),
    Sequence(Vec<Stmt>),
    If(Expr, Box<Stmt>, Box<Stmt>),
    While(Expr, Box<Stmt>),
}

#[derive(Clone,PartialEq,Eq,Debug,Uniplate)]
#[biplate(to=String)]
#[uniplate()]
enum Expr {
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Val(i32),
    Var(String),
    Neg(Box<Expr>),
}

fn vars_names(stmt: &Stmt) -> Vec<String>{
    let names: VecDeque<String> = stmt.universe_bi();
    names.into()
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
