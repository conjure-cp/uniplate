use quote::format_ident;
use syn::token;

use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum Data {
    DataEnum(DataEnum),
    DataStruct(DataStruct),
}

impl Data {
    #[allow(dead_code)]
    pub fn span(&self) -> Span {
        match self {
            Data::DataEnum(x) => x.span,
            Data::DataStruct(x) => x.span,
        }
    }

    pub fn ident(&self) -> syn::Ident {
        match self {
            Data::DataEnum(x) => x.ident.clone(),
            Data::DataStruct(x) => x.ident.clone(),
        }
    }
}

impl From<Data> for ast::PlateableType {
    fn from(val: Data) -> Self {
        let mut typ_segments: Punctuated<syn::PathSegment, syn::token::PathSep> = Punctuated::new();
        typ_segments.push(syn::PathSegment {
            ident: val.ident(),
            arguments: syn::PathArguments::None,
        });

        let base_typ: syn::Path = syn::Path {
            leading_colon: None,
            segments: typ_segments,
        };

        ast::PlateableType {
            base_typ,
            wrapper_typ: None,
        }
    }
}

impl Parse for Data {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<syn::Visibility>()?;
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![enum]) {
            input.parse().map(Data::DataEnum)
        } else if lookahead.peek(Token![struct]) {
            input.parse().map(Data::DataStruct)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Clone, Debug)]
pub struct DataEnum {
    pub ident: syn::Ident,
    pub span: Span,
    pub variants: Vec<Variant>,
}
impl Parse for DataEnum {
    // Layout of an enum as per:
    // https://doc.rust-lang.org/stable/reference/items/enumerations.html
    // https://docs.rs/syn/latest/syn/struct.ItemEnum.html

    fn parse(input: ParseStream) -> syn::Result<Self> {
        //input.parse::<syn::Visibility>()?;
        input.parse::<Token![enum]>()?;
        let ident = input.parse::<syn::Ident>()?;

        input.parse::<syn::Generics>()?;

        let content;
        braced! {content in input};

        let variants: Punctuated<Variant, Token![,]> =
            content.parse_terminated(Variant::parse, Token![,])?;

        Ok(DataEnum {
            span: ident.span(),
            ident,
            variants: variants.into_iter().collect(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct Variant {
    pub ident: syn::Ident,
    #[allow(dead_code)]
    pub span: Span,
    pub fields: Vec<TupleField>, // TODO: Support named fields
}
impl Parse for Variant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Layout of a variant as per:
        // https://docs.rs/syn/latest/syn/struct.Variant.html
        // https://doc.rust-lang.org/stable/reference/items/enumerations.html

        input.call(syn::Attribute::parse_outer)?;
        let ident: syn::Ident = input.parse()?;

        if !input.peek(token::Paren) {
            return Ok(Variant {
                span: ident.span(),
                ident,
                fields: Default::default(),
            });
        }

        let content;
        parenthesized! {content in input};

        let fields: Punctuated<TupleField, Token![,]> =
            content.call(Punctuated::parse_terminated)?;
        Ok(Variant {
            span: ident.span(),
            ident,
            fields: fields.into_iter().collect(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct DataStruct {
    pub ident: syn::Ident,
    pub span: Span,
    pub fields: Fields,
}
impl Parse for DataStruct {
    // Layout of a struct as per:
    // https://doc.rust-lang.org/stable/reference/items/structs.html
    // https://docs.rs/syn/latest/syn/struct.ItemStruct.html

    fn parse(input: ParseStream) -> syn::Result<Self> {
        // input.parse::<syn::Visibility>()?;
        input.parse::<Token![struct]>()?;
        let ident = input.parse::<syn::Ident>()?;

        input.parse::<syn::Generics>()?;

        Ok(DataStruct {
            span: ident.span(),
            ident,
            fields: input.parse()?,
        })
    }
}

// A collection of fields, such as in a struct or enum variant
// See https://doc.rust-lang.org/stable/reference/items/structs.html
#[derive(Clone, Debug)]
pub enum Fields {
    Struct(Vec<StructField>), // { name: Type, ... }
    Tuple(Vec<TupleField>),   // (Type, ...)
    Unit,                     // Unit-like struct or enum variant
}

impl Parse for Fields {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let lookahead = input.lookahead1();
        if lookahead.peek(token::Brace) {
            // Struct fields (named)
            braced!(content in input);
            let fields: Punctuated<StructField, Token![,]> =
                content.parse_terminated(StructField::parse, Token![,])?;
            Ok(Fields::Struct(fields.into_iter().collect()))
        } else if lookahead.peek(token::Paren) {
            // Tuple fields (anonymous)
            parenthesized!(content in input);
            let fields: Punctuated<TupleField, Token![,]> =
                content.parse_terminated(TupleField::parse, Token![,])?;
            input.parse::<Token![;]>()?;
            Ok(Fields::Tuple(fields.into_iter().collect()))
        } else {
            // Unit-like (no fields)
            input.parse::<Token![;]>()?;
            Ok(Fields::Unit)
        }
    }
}

impl Fields {
    pub fn is_empty(&self) -> bool {
        match self {
            Fields::Struct(fields) => fields.is_empty(),
            Fields::Tuple(fields) => fields.is_empty(),
            Fields::Unit => true,
        }
    }

    pub fn idents(&self) -> Box<dyn Iterator<Item = syn::Ident> + '_> {
        match self {
            Fields::Struct(fields) => Box::new(fields.iter().map(|f| f.ident.clone())),
            Fields::Tuple(fields) => Box::new(
                fields
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format_ident!("f{}", i)),
            ),
            Fields::Unit => Box::new([].iter().cloned()),
        }
    }

    pub fn members(&self) -> Box<dyn Iterator<Item = syn::Member> + '_> {
        match self {
            Fields::Struct(fields) => {
                Box::new(fields.iter().map(|f| syn::Member::Named(f.ident.clone())))
            }
            Fields::Tuple(fields) => {
                Box::new(fields.iter().enumerate().map(|(i, _)| syn::Member::from(i)))
            }
            Fields::Unit => Box::new([].iter().cloned()),
        }
    }

    pub fn types(&self) -> Box<dyn Iterator<Item = &ast::Type> + '_> {
        match self {
            Fields::Struct(fields) => Box::new(fields.iter().map(|f| &f.typ)),
            Fields::Tuple(fields) => Box::new(fields.iter().map(|f| &f.typ)),
            Fields::Unit => Box::new([].iter()),
        }
    }

    pub fn defs(&self) -> Box<dyn Iterator<Item = (syn::Member, &ast::Type)> + '_> {
        Box::new(std::iter::zip(self.members(), self.types()))
    }
}

/// An unnamed (anonymous) field in a tuple struct or enum variant
/// e.g. `struct TupleLike(i32, i32);`
#[derive(Clone, Debug)]
pub struct TupleField {
    #[allow(dead_code)]
    pub span: Span,
    pub typ: ast::Type,
}

impl Parse for TupleField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Layout of a field as per:
        // https://docs.rs/syn/latest/syn/struct.Field.html
        // https://doc.rust-lang.org/stable/reference/items/structs.html (tuple field)
        input.call(syn::Attribute::parse_outer)?;
        input.parse::<syn::Visibility>()?;
        let span = input.span();
        let typ: ast::Type = input.parse()?;
        Ok(TupleField { span, typ })
    }
}

/// A named (non-anonymous) field in a struct or enum variant
/// e.g. `struct Struct { field: i32 };`
#[derive(Clone, Debug)]
pub struct StructField {
    #[allow(dead_code)]
    pub span: Span,
    pub ident: syn::Ident,
    pub typ: ast::Type,
}

impl Parse for StructField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.call(syn::Attribute::parse_outer)?;
        input.parse::<syn::Visibility>()?;

        let ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let typ = input.parse()?;

        Ok(StructField {
            span: input.span(),
            ident,
            typ,
        })
    }
}
