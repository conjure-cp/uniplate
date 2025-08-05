use std::collections::BTreeMap;

use quote::format_ident;
use syn::token;

use crate::prelude::*;

/// A datatype we are deriving uniplate on
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

    pub fn generics(&self) -> &Generics {
        match self {
            Data::DataEnum(data_enum) => &data_enum.generics,
            Data::DataStruct(data_struct) => &data_struct.generics,
        }
    }
}

impl From<Data> for ast::Type {
    fn from(value: Data) -> Self {
        ast::Type::Basic(value.into())
    }
}

impl From<Data> for ast::BasicType {
    fn from(val: Data) -> Self {
        let mut typ_segments: Punctuated<syn::PathSegment, syn::token::PathSep> = Punctuated::new();

        let arguments = if val.generics().any_generic_params() {
            syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: Default::default(),
                args: val.generics().as_generic_arguments(),
                gt_token: Default::default(),
            })
        } else {
            syn::PathArguments::None
        };

        typ_segments.push(syn::PathSegment {
            ident: val.ident(),
            arguments,
        });

        ast::BasicType::new(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: typ_segments,
            },
        }))
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
    pub generics: Generics,
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

        let generic_params: GenericParameters = input.parse()?;

        let lookahead = input.lookahead1();

        let where_clause: Option<syn::WhereClause> = if lookahead.peek(Token![where]) {
            Some(input.parse()?)
        } else {
            None
        };

        let generics = Generics::new(generic_params, where_clause);

        let content;
        braced! {content in input};

        let variants: Punctuated<Variant, Token![,]> =
            content.parse_terminated(Variant::parse, Token![,])?;

        Ok(DataEnum {
            generics,
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
    pub fields: Fields,
}
impl Parse for Variant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Layout of a variant as per:
        // https://docs.rs/syn/latest/syn/struct.Variant.html
        // https://doc.rust-lang.org/stable/reference/items/enumerations.html

        input.call(syn::Attribute::parse_outer)?;
        let ident: syn::Ident = input.parse()?;
        let fields: Fields = input.parse()?;

        Ok(Variant {
            span: ident.span(),
            ident,
            fields,
        })
    }
}

#[derive(Clone, Debug)]
pub struct DataStruct {
    pub ident: syn::Ident,
    pub span: Span,
    pub generics: Generics,
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

        let generic_params: GenericParameters = input.parse()?;

        let lookahead = input.lookahead1();

        let where_clause: Option<syn::WhereClause> = if lookahead.peek(Token![where]) {
            Some(input.parse()?)
        } else {
            None
        };

        let generics = Generics::new(generic_params, where_clause);

        let fields: Fields = input.parse()?;

        match fields {
            Fields::Struct(_) => {}
            Fields::Tuple(_) => {
                input.parse::<Token![;]>()?;
            }
            Fields::Unit => {
                input.parse::<Token![;]>()?;
            }
        }

        Ok(DataStruct {
            generics,
            span: ident.span(),
            ident,
            fields,
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
            Ok(Fields::Tuple(fields.into_iter().collect()))
        } else {
            // Unit-like (no fields)
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
                    .map(|(i, _)| format_ident!("_{}", i)),
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

/// The generics for a declaration, and any bounds or conditions on them.
///
/// This includes generics stored inside angle brackets, as well as conditions on them using where
/// clauses.
#[derive(Clone, Debug)]
pub struct Generics {
    /// type parameters and their bounds
    pub type_parameters: BTreeMap<syn::Ident, Vec<syn::TypeParamBound>>,

    /// lifetime parameters and their bounds
    pub lifetime_parameters: BTreeMap<syn::Lifetime, Vec<syn::Lifetime>>,

    /// constant parameters
    pub const_parameters: Vec<syn::ConstParam>,

    /// where predicates
    pub where_predicates: Vec<syn::WherePredicate>,
}

impl Generics {
    pub fn new(params: GenericParameters, where_clause: Option<syn::WhereClause>) -> Generics {
        let mut type_parameters: BTreeMap<_, _> = BTreeMap::new();
        let mut lifetime_parameters: BTreeMap<_, _> = BTreeMap::new();
        let mut const_parameters: Vec<_> = Vec::new();
        let mut where_predicates: Vec<_> = Vec::new();
        for param in params.params.into_iter() {
            match param {
                syn::GenericParam::Lifetime(lifetime_param) => {
                    let lifetime = lifetime_param.lifetime;
                    let lifetime_bounds: Vec<_> = lifetime_param.bounds.into_iter().collect();
                    if lifetime_parameters.contains_key(&lifetime) {
                        syn::Error::new(lifetime.span(), "Duplicate lifetime parameter")
                            .to_compile_error();
                    }

                    lifetime_parameters.insert(lifetime, lifetime_bounds);
                }
                syn::GenericParam::Type(type_param) => {
                    let typ = type_param.ident;
                    let bounds: Vec<_> = type_param.bounds.into_iter().collect();

                    if type_parameters.contains_key(&typ) {
                        syn::Error::new(typ.span(), "Duplicate type parameter").to_compile_error();
                    }
                    type_parameters.insert(typ, bounds);
                }
                syn::GenericParam::Const(const_param) => {
                    const_parameters.push(const_param);
                }
            }
        }

        if let Some(where_clause) = where_clause {
            where_predicates.extend(where_clause.predicates);
        };

        Generics {
            type_parameters,
            lifetime_parameters,
            const_parameters,
            where_predicates,
        }
    }

    /// Returns the generic parameters for the type to use in an impl block.
    ///
    /// ```text
    /// impl<...> MyType<...> where ... {
    ///      ~~~         ~~~
    ///            (to go here)
    /// }
    /// ```
    pub fn impl_parameters(&self) -> proc_macro2::TokenStream {
        let mut token_stream = proc_macro2::TokenStream::new();
        for (lifetime, bounds) in &self.lifetime_parameters {
            if bounds.is_empty() {
                token_stream.extend(quote!(#lifetime,));
            } else {
                token_stream.extend(quote! {#lifetime: #(#bounds)+*,});
            }
        }

        for (typ, bounds) in &self.type_parameters {
            if bounds.is_empty() {
                token_stream.extend(quote!(#typ,));
            } else {
                token_stream.extend(quote! {#typ: #(#bounds)+*,});
            }
        }

        let const_params = &self.const_parameters;
        token_stream.extend(quote! {#(#const_params),*});

        token_stream
    }

    /// Returns the where clause to use in an impl for this type.
    ///
    /// ```text
    /// impl<...> MyType<...> where ... {
    ///                       ~~~~~~~~~
    ///                       (this bit)
    /// M
    /// ```
    pub fn impl_type_where_block(&self) -> proc_macro2::TokenStream {
        let where_predicates = &self.where_predicates;
        if where_predicates.is_empty() {
            TokenStream2::new()
        } else {
            quote! {
                where #(#where_predicates),*
            }
        }
    }

    /// Returns the generic parameters of the type without bounds, for use in a type path (e.g.
    /// referencing this type when declaraing a variable, `let a: Foo<T> = bar;`.)
    pub fn as_generic_arguments(&self) -> Punctuated<syn::GenericArgument, Token![,]> {
        let mut punctuated = Punctuated::new();
        for lifetime in self.lifetime_parameters.keys() {
            punctuated.push(syn::GenericArgument::Lifetime(lifetime.clone()));
        }

        for typ in self.type_parameters.keys() {
            // is Type::Verbatim ok?
            punctuated.push(syn::GenericArgument::Type(syn::Type::Verbatim(
                typ.to_token_stream(),
            )));
        }
        for const_param in &self.const_parameters {
            punctuated.push(syn::GenericArgument::Type(syn::Type::Verbatim(
                const_param.to_token_stream(),
            )));
        }

        punctuated
    }

    pub fn any_generic_params(&self) -> bool {
        !self.type_parameters.is_empty()
            && self.lifetime_parameters.is_empty()
            && self.const_parameters.is_empty()
    }
}

/// The generic parameters for a declaration, stored inside angled brackets.
#[derive(Clone, Debug)]
pub struct GenericParameters {
    params: Vec<syn::GenericParam>,
}

impl Parse for GenericParameters {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // just get syn to parse this
        let generics: syn::Generics = input.parse()?;

        Ok(GenericParameters {
            params: generics.params.into_iter().collect(),
        })
    }
}
