# Uniplate

**Uniplate helps you write simple, boilerplate-free operations on tree shaped
data types.**

It is a port of the Haskell library [Uniplate](https://hackage.haskell.org/package/uniplate) into Rust.

----

Quick links:

* [Installing Uniplate from crates.io](https://crates.io/crates/uniplate/0.1.0)
* [Documentation (docs.rs)](https://docs.rs/crate/uniplate/0.1.0)
* [Github Repository](https://github.com/conjure-cp/uniplate)

## A simple example

*Adapted from (Mitchell and Runciman 2007)*

Uniplate makes the traversal and querying of tree shaped data easy and
boilerplate free. A good use case of Uniplate is the manipulation of abstract
syntax trees.

Consider the AST for a simple calculator language:

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

Say we want to list all the variable names used inside a given expression:
                                                                                               
```rust
fn vars_names(expr: &Expr) -> Vec<String>{
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

Functions like these are annoying to write: the first 4 constructors are
basically identical, adding a new expression type requires a new line to be
added to all match statement, and this code cannot be shared with similar
functions (e.g. one that change all the variable names).

With Uniplate, this boilerplate can be eliminated:

```rust
use uniplate::{Uniplate,Biplate};
use uniplate::derive::Uniplate;
#[derive(Clone,PartialEq,Eq,Debug,Uniplate)]
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

fn vars_names(expr: &Expr) -> Vec<String>{
    <Expr as Biplate<String>>::universe_bi(expr).into_iter().collect()
}
```


Uniplate also supports trees with multiple recursive types. Lets extend our
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
variable names used by child expressions.


```rust
use uniplate::{Uniplate,Biplate};
use uniplate::derive::Uniplate;
#[derive(Clone,PartialEq,Eq,Debug,Uniplate)]
// look for strings inside expressions as well as statements 
#[biplate(to=String,walk_into=[Expr])]
#[biplate(to=Expr)]
#[uniplate()]
enum Stmt {
    Assign(String, Expr),
    Sequence(Vec<Stmt>),
    If(Expr, Box<Stmt>, Box<Stmt>),
    While(Expr, Box<Stmt>),
}

#[derive(Clone,PartialEq,Eq,Debug,Uniplate)]
#[biplate(to=String])]
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
    <Stmt as Biplate<String>>::universe_bi(stmt).into_iter().collect()
}

```

Despite having to recursively look through multiple types, this operation is
no harder to write!


## Acknowledgements

This library is inspired by Neil Mitchell's Haskell library
[Uniplate](https://hackage.haskell.org/package/uniplate) and its accompanying
paper: *Neil Mitchell and Colin Runciman. 2007. Uniform boilerplate and list
processing*.


