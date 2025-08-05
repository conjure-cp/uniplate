use crate::prelude::*;
use lazy_static::lazy_static;
use syn::{PathArguments, parse_quote};

lazy_static! {
    static ref BOX_PREFIXES: Vec<&'static str> =
        vec!("::std::boxed::Box", "std::boxed::Box", "Box");
}

/// A type
#[derive(Clone, Debug)]
pub enum Type {
    /// A boxed basic type
    BoxedBasic(BasicType),

    /// A basic type
    Basic(BasicType),

    /// A tuple type
    Tuple(TupleType),

    /// A boxed tuple type
    BoxedTuple(TupleType),
}

impl Parse for Type {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let syn_typ: syn::Type = input.parse()?;
        match syn_typ {
            syn::Type::Array(_) => {
                Err(input.error("uniplate_derive: array types are not supported."))
            }
            syn::Type::BareFn(_) => {
                Err(input.error("uniplate_derive: fn types are not supported."))
            }
            syn::Type::Group(_) => {
                Err(input.error("uniplate_derive: group types are not supported."))
            }
            syn::Type::ImplTrait(_) => {
                Err(input.error("uniplate_derive: impl types are not supported."))
            }
            syn::Type::Infer(_) => {
                Err(input.error("uniplate_derive: inferred types are not supported."))
            }
            syn::Type::Macro(_) => {
                Err(input.error("uniplate_derive: macros in the type position are not supported."))
            }
            syn::Type::Never(_) => {
                Err(input.error("uniplate_derive: never types are not supported."))
            }
            syn::Type::Paren(_) => {
                Err(input.error("uniplate_derive: paren types are not supported."))
            }
            syn::Type::Ptr(_) => {
                Err(input.error("uniplate_derive: raw pointer types are not supported."))
            }
            syn::Type::Reference(_) => {
                Err(input.error("uniplate_derive: reference types are not yet supported."))
            }
            syn::Type::Slice(_) => {
                Err(input.error("uniplate_derive: slice types are not supported."))
            }
            syn::Type::TraitObject(_) => {
                Err(input.error("uniplate_derive: trait object types are not supported."))
            }
            syn::Type::Tuple(tuple_type) => Ok(Type::Tuple(TupleType::from_syn(tuple_type)?)),
            syn::Type::Verbatim(_) => {
                Err(input.error("uniplate_derive: verbatim types are not yet supported."))
            }
            syn::Type::Path(ref type_path) => {
                // Is this type boxed?

                // To check whether this type is boxed: store the type without any parameters, and
                // stringify it so that we can compare it against our list of known box types.
                let mut type_segments = type_path.path.segments.clone();
                type_segments.last_mut().unwrap().arguments = PathArguments::None;
                let type_prefix: String = quote!(#type_segments).to_string();

                if BOX_PREFIXES.contains(&type_prefix.as_str()) {
                    // Type is inside a box
                    let type_segments = &type_path.path.segments;
                    if let syn::PathArguments::AngleBracketed(ref args) =
                        type_segments.last().unwrap().arguments
                        && args.args.len() == 1
                        && let syn::GenericArgument::Type(inner_type) = args.args.last().unwrap()
                    {
                        let inner_type: Type = parse_quote!(#inner_type);
                        match inner_type {
                            Type::Basic(basic_type) => Ok(Type::BoxedBasic(basic_type)),
                            Type::Tuple(tuple_type) => Ok(Type::BoxedTuple(tuple_type)),
                            Type::BoxedBasic(_) | Type::BoxedTuple(_) => {
                                Err(input.error("uniplate_derive: nested boxes are not supported."))
                            }
                        }
                    } else {
                        Err(input.error("uniplate_derive: invalid box type"))
                    }
                } else {
                    // Type is not inside a box
                    Ok(Type::Basic(BasicType::new(syn_typ)))
                }
            }
            _ => unreachable!(),
        }
    }
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Type::BoxedBasic(basic_type) => {
                tokens.extend(quote!(Box<#basic_type>));
            }
            Type::Basic(basic_type) => {
                basic_type.to_tokens(tokens);
            }
            Type::Tuple(tuple_type) => {
                tuple_type.to_tokens(tokens);
            }
            Type::BoxedTuple(tuple_type) => {
                tokens.extend(quote!(Box<#tuple_type>));
            }
        }
    }
}

/// A basic type
#[derive(Clone, Debug)]
pub struct BasicType {
    pub typ: syn::Type,
}

impl BasicType {
    pub fn new(typ: syn::Type) -> Self {
        BasicType { typ }
    }
}

impl ToTokens for BasicType {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.typ.to_tokens(tokens);
    }
}

/// A tuple type
#[derive(Clone, Debug)]
pub struct TupleType {
    /// the types of the tuple fields
    ///
    /// for now, these must be unboxed basic types, but that restriction may be lifted later.
    pub fields: Vec<BasicType>,
    /// the number of types this tuple has
    pub n: usize,
}

impl TupleType {
    pub fn from_syn(typ: syn::TypeTuple) -> Result<Self, syn::Error> {
        let mut fields = vec![];
        for syn_field_type in typ.elems.into_iter() {
            let field_type: Type = parse_quote!(#syn_field_type);
            let Type::Basic(field_type) = field_type else {
                return Err(syn::Error::new(
                    syn_field_type.span(),
                    "uniplate_derive: expect tuple field to be an unboxed basic type",
                ));
            };
            fields.push(field_type);
        }

        Ok(TupleType {
            n: fields.len(),
            fields,
        })
    }
}
impl ToTokens for TupleType {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let fields = &self.fields;
        tokens.extend(quote! {
            (#(#fields),*)
        });
    }
}
