mod ast;
mod prelude;
mod state;

use std::collections::VecDeque;

use prelude::*;
use quote::format_ident;
use syn::parse_macro_input;

#[proc_macro_derive(Uniplate, attributes(uniplate, biplate))]
pub fn uniplate_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ast::DeriveInput);
    let mut state: ParserState = ParserState::new(input.clone());

    let mut out_tokens: Vec<TokenStream2> = Vec::new();
    out_tokens.push(quote! {
        use std::borrow::Borrow as _;
    });

    while state.next_instance().is_some() {
        out_tokens.push(match &state.current_instance {
            Some(ast::InstanceMeta::Uniplate(_)) => derive_a_uniplate(&mut state),
            Some(ast::InstanceMeta::Biplate(_)) => derive_a_biplate(&mut state),
            _ => unreachable!(),
        });
    }

    out_tokens.into_iter().collect::<TokenStream2>().into()
}

fn derive_a_uniplate(state: &mut ParserState) -> TokenStream2 {
    let from = state.from.to_token_stream();
    let tokens: TokenStream2 = match state.data.clone() {
        ast::Data::DataEnum(x) => _derive_a_enum_uniplate(state, x),
        ast::Data::DataStruct(x) => _derive_a_struct_uniplate(state, x),
    };

    let mut generics = state.data.generics().clone();
    for (_, bounds) in generics.type_parameters.iter_mut() {
        // Add 'static bounds to all generic type parameters.
        bounds.push(syn::TypeParamBound::Verbatim(quote!('static)));
    }

    let impl_bounds = generics.impl_parameters();
    let where_clause = generics.impl_type_where_block();
    quote! {
        impl<#impl_bounds> ::uniplate::Uniplate for #from #where_clause {
            fn uniplate(&self) -> (::uniplate::Tree<#from>, Box<dyn Fn(::uniplate::Tree<#from>) -> #from>) {
                #tokens
            }
        }
    }
}

fn _derive_a_enum_uniplate(state: &mut ParserState, data: ast::DataEnum) -> TokenStream2 {
    let mut variant_tokens = VecDeque::<TokenStream2>::new();
    for variant in data.variants {
        let fields = &variant.fields;
        let field_idents: Vec<_> = fields.idents().collect();

        let field_defs: Vec<_> = fields
            .defs()
            .map(|(mem, typ)| _derive_for_field_enum(state, typ, &mem))
            .collect();

        let children_def = _derive_children(state, fields);
        let ctx_def = _derive_ctx(state, fields, Some(&variant.ident));
        let ident = variant.ident;
        let enum_ident = state.data.ident();

        match variant.fields {
            ast::Fields::Struct(_) => {
                variant_tokens.push_back(quote! {
                    #enum_ident::#ident{#(#field_idents),*} => {
                        #(#field_defs)*

                        #children_def

                        #ctx_def

                        (children,ctx)
                    },
                });
            }

            ast::Fields::Tuple(_) => {
                variant_tokens.push_back(quote! {
                    #enum_ident::#ident(#(#field_idents),*) => {
                        #(#field_defs)*

                        #children_def

                        #ctx_def

                        (children,ctx)
                    },
                });
            }
            ast::Fields::Unit => {
                variant_tokens.push_back(quote! {
                    #enum_ident::#ident => {
                        #children_def

                        #ctx_def

                        (children,ctx)
                    },
                });
            }
        }
    }

    let variant_tokens = variant_tokens.iter();
    quote! {
        match self {
            #(#variant_tokens)*
        }
    }
}

fn _derive_a_struct_uniplate(state: &mut ParserState, data: ast::DataStruct) -> TokenStream2 {
    let struct_ident = state.data.ident();
    if data.fields.is_empty() {
        // Unit-like or empty struct
        return quote! {
            (::uniplate::Tree::Zero,Box::new(|_| #struct_ident))
        };
    }

    let field_defs: Vec<_> = data
        .fields
        .defs()
        .map(|(mem, typ)| _derive_for_field_struct(state, typ, mem))
        .collect();
    let children_def = _derive_children(state, &data.fields);
    let ctx_def = _derive_ctx(state, &data.fields, None);

    quote! {
        #(#field_defs)*

        #children_def

        #ctx_def

        (children,ctx)
    }
}

fn _derive_for_field_enum(
    state: &mut ParserState,
    field_type: &ast::Type,
    member: &syn::Member,
) -> TokenStream2 {
    // the identifier used in the match clause.
    // either _1, or the field name.
    let match_ident = match member {
        syn::Member::Named(ident) => ident.clone(),
        syn::Member::Unnamed(index) => format_ident!("_{}", index),
    };

    let children_ident = format_ident!("_{}_children", member);
    let ctx_ident = format_ident!("_{}_ctx", member);

    let to_t = state.to.clone().expect("").to_token_stream();

    match field_type {
        ast::Type::BoxedBasic(_) => {
            quote! {
                let (#children_ident,#ctx_ident) = ::uniplate::spez::try_biplate_to!((**#match_ident).clone(), #to_t);
            }
        }
        ast::Type::Basic(_) => {
            quote! {
                let (#children_ident,#ctx_ident) = ::uniplate::spez::try_biplate_to!(#match_ident.clone(), #to_t);
            }
        }
        ast::Type::Tuple(tuple_type) => {
            // destructure the tuple
            let tuple_field_idents =
                (0..tuple_type.n).map(|i| format_ident!("_{}_tuple_field_{i}", member));
            let destructure_tuple = quote! {
                    let (#(#tuple_field_idents),*) = #match_ident;
            };

            // call biplate on each tuple field
            let call_biplate_for_each_field = tuple_type.fields.iter().enumerate().map(|(i,_)| {
                let field_ident = format_ident!("_{}_tuple_field_{i}",member);
                let field_children_ident = format_ident!("_{}_tuple_field_{i}_children",member);
                let field_ctx_ident = format_ident!("_{}_tuple_field_{i}_ctx", member);

                // let index = syn::Index::from(i);
                quote!{
                    let (#field_children_ident,#field_ctx_ident) = ::uniplate::spez::try_biplate_to!(#field_ident.clone(), #to_t);
                }
            });

            let tuple_field_children_idents =
                (0..tuple_type.n).map(|i| format_ident!("_{}_tuple_field_{i}_children", member));
            let tuple_field_ctx_idents =
                (0..tuple_type.n).map(|i| format_ident!("_{}_tuple_field_{i}_ctx", member));

            // build the children tree by combining each fields' tree
            let build_child_tree = quote! {
                let #children_ident = ::uniplate::Tree::Many(::std::collections::VecDeque::from([#(#tuple_field_children_idents),*]));
            };

            let index = (0..tuple_type.n).map(syn::Index::from);

            // build the context function
            let build_child_ctx = quote! {
                let #ctx_ident = Box::new(move |x| {
                    let ::uniplate::Tree::Many(xs) = x else {
                        panic!()
                    };

                    (#(#tuple_field_ctx_idents(xs[#index].clone())),*)
                });
            };

            quote! {
                #destructure_tuple
                #(#call_biplate_for_each_field);*
                #build_child_tree
                #build_child_ctx
            }
        }
        ast::Type::BoxedTuple(tuple_type) => {
            // destructure the tuple
            let tuple_field_idents =
                (0..tuple_type.n).map(|i| format_ident!("_{}_tuple_field_{i}", member));
            let destructure_tuple = quote! {
                    let (#(#tuple_field_idents),*) = (**#match_ident).clone();
            };

            // call biplate on each tuple field
            let call_biplate_for_each_field = tuple_type.fields.iter().enumerate().map(|(i,_)| {
                let field_ident = format_ident!("_{}_tuple_field_{i}",member);
                let field_children_ident = format_ident!("_{}_tuple_field_{i}_children",member);
                let field_ctx_ident = format_ident!("_{}_tuple_field_{i}_ctx", member);            

                // let index = syn::Index::from(i);
                quote!{
                    let (#field_children_ident,#field_ctx_ident) = ::uniplate::spez::try_biplate_to!(#field_ident.clone(), #to_t);
                }
            });

            let tuple_field_children_idents =
                (0..tuple_type.n).map(|i| format_ident!("_{}_tuple_field_{i}_children", member));
            let tuple_field_ctx_idents =
                (0..tuple_type.n).map(|i| format_ident!("_{}_tuple_field_{i}_ctx", member));

            // build the children tree by combining each fields' tree
            let build_child_tree = quote! {
                let #children_ident = ::uniplate::Tree::Many(::std::collections::VecDeque::from([#(#tuple_field_children_idents),*]));
            };

            let index = (0..tuple_type.n).map(syn::Index::from);

            // build the context function
            let build_child_ctx = quote! {
                let #ctx_ident = Box::new(move |x| {
                    let ::uniplate::Tree::Many(xs) = x else {
                        panic!()
                    };

                    (#(#tuple_field_ctx_idents(xs[#index].clone())),*)
                });
            };

            quote! {
                #destructure_tuple
                #(#call_biplate_for_each_field);*
                #build_child_tree
                #build_child_ctx
            }
        }
    }
}

fn _derive_for_field_struct(
    state: &mut ParserState,
    field_type: &ast::Type,
    member: syn::Member,
) -> TokenStream2 {
    let children_ident = format_ident!("_{}_children", member);
    let ctx_ident = format_ident!("_{}_ctx", member);

    let to_t = state.to.clone().expect("").to_token_stream();

    match field_type {
        ast::Type::BoxedBasic(_) => {
            quote! {
                let (#children_ident,#ctx_ident) = ::uniplate::spez::try_biplate_to!((*self.#member).clone(), #to_t);
            }
        }
        ast::Type::Basic(_) => {
            quote! {
                let (#children_ident,#ctx_ident) = ::uniplate::try_biplate_to!(self.#member.clone(), #to_t);
            }
        }
        ast::Type::Tuple(tuple_type) => {
            // destructure the tuple
            let tuple_field_idents =
                (0..tuple_type.n).map(|i| format_ident!("_{}_tuple_field_{i}", member));
            let destructure_tuple = quote! {
                    let (#(#tuple_field_idents),*) = self.#member.clone();
            };

            // call biplate on each tuple field
            let call_biplate_for_each_field = tuple_type.fields.iter().enumerate().map(|(i,_)| {
                let field_ident = format_ident!("_{}_tuple_field_{i}",member);
                let field_children_ident = format_ident!("_{}_tuple_field_{i}_children",member);
                let field_ctx_ident = format_ident!("_{}_tuple_field_{i}_ctx", member);

                // let index = syn::Index::from(i);
                quote!{
                    let (#field_children_ident,#field_ctx_ident) = ::uniplate::spez::try_biplate_to!(#field_ident.clone(), #to_t);
                }
            });

            let tuple_field_children_idents =
                (0..tuple_type.n).map(|i| format_ident!("_{}_tuple_field_{i}_children", member));
            let tuple_field_ctx_idents =
                (0..tuple_type.n).map(|i| format_ident!("_{}_tuple_field_{i}_ctx", member));

            // build the children tree by combining each fields' tree
            let build_child_tree = quote! {
                let #children_ident = ::uniplate::Tree::Many(::std::collections::VecDeque::from([#(#tuple_field_children_idents),*]));
            };

            let index = (0..tuple_type.n).map(syn::Index::from);

            // build the context function
            let build_child_ctx = quote! {
                let #ctx_ident = Box::new(move |x| {
                    let ::uniplate::Tree::Many(xs) = x else {
                        panic!()
                    };

                    (#(#tuple_field_ctx_idents(xs[#index].clone())),*)
                });
            };

            quote! {
                #destructure_tuple
                #(#call_biplate_for_each_field);*
                #build_child_tree
                #build_child_ctx
            }
        }
        ast::Type::BoxedTuple(tuple_type) => {
            // destructure the tuple
            let tuple_field_idents =
                (0..tuple_type.n).map(|i| format_ident!("_{}_tuple_field_{i}", member));
            let destructure_tuple = quote! {
                    let (#(#tuple_field_idents),*) = (*self.#member).clone();
            };

            // call biplate on each tuple field
            let call_biplate_for_each_field = tuple_type.fields.iter().enumerate().map(|(i,_)| {
                let field_ident = format_ident!("_{}_tuple_field_{i}",member);
                let field_children_ident = format_ident!("_{}_tuple_field_{i}_children",member);
                let field_ctx_ident = format_ident!("_{}_tuple_field_{i}_ctx", member);

                // let index = syn::Index::from(i);
                quote!{
                    let (#field_children_ident,#field_ctx_ident) = ::uniplate::spez::try_biplate_to!(#field_ident.clone(), #to_t);
                }
            });

            let tuple_field_children_idents =
                (0..tuple_type.n).map(|i| format_ident!("_{}_tuple_field_{i}_children", member));
            let tuple_field_ctx_idents =
                (0..tuple_type.n).map(|i| format_ident!("_{}_tuple_field_{i}_ctx", member));

            // build the children tree by combining each fields' tree
            let build_child_tree = quote! {
                let #children_ident = ::uniplate::Tree::Many(::std::collections::VecDeque::from([#(#tuple_field_children_idents),*]));
            };

            let index = (0..tuple_type.n).map(syn::Index::from);

            // build the context function
            let build_child_ctx = quote! {
                let #ctx_ident = Box::new(move |x| {
                    let ::uniplate::Tree::Many(xs) = x else {
                        panic!()
                    };

                    (#(#tuple_field_ctx_idents(xs[#index].clone())),*)
                });
            };

            quote! {
                #destructure_tuple
                #(#call_biplate_for_each_field);*
                #build_child_tree
                #build_child_ctx
            }
        }
    }
}

fn _derive_children(_state: &mut ParserState, fields: &ast::Fields) -> TokenStream2 {
    let mut subtrees: VecDeque<TokenStream2> = VecDeque::new();
    for (member, _) in fields.defs() {
        subtrees.push_back({
            let children_ident = format_ident!("_{}_children", member);
            quote!(#children_ident)
        });
    }

    match subtrees.len() {
        0 => quote! {let children = ::uniplate::Tree::Zero;},
        _ => {
            let subtrees = subtrees.iter();
            quote! {let children = ::uniplate::Tree::Many(::std::collections::VecDeque::from([#(#subtrees),*]));}
        }
    }
}

fn _derive_ctx(
    state: &mut ParserState,
    fields: &ast::Fields,
    var_ident: Option<&syn::Ident>,
) -> TokenStream2 {
    let field_ctxs: Vec<_> = fields
        .defs()
        .enumerate()
        .map(|(i, (mem, typ))| match typ {
            ast::Type::Basic(_) | ast::Type::Tuple(_) => {
                let ctx_ident = format_ident!("_{}_ctx", mem);
                quote! {#ctx_ident(x[#i].clone())}
            }

            ast::Type::BoxedBasic(_) | ast::Type::BoxedTuple(_) => {
                let ctx_ident = format_ident!("_{}_ctx", mem);
                quote! {Box::new(#ctx_ident(x[#i].clone()))}
            }
        })
        .collect();

    let data_ident = state.data.ident(); // The enum or struct name
    let typ = state.to.clone();

    // If this is an enum, use the passed variant identifier
    let construct_ident = match var_ident {
        Some(var) => quote! {#data_ident::#var},
        None => quote! {#data_ident},
    };
    if fields.is_empty() {
        quote! {
            let ctx = Box::new(move |x: ::uniplate::Tree<#typ>| {
                let ::uniplate::Tree::Zero = x else { panic!()};
                #construct_ident
            });
        }
    } else {
        // If this is an enum, use the passed variant identifier
        let construct_ident = match var_ident {
            Some(var) => quote! {#data_ident::#var},
            None => quote! {#data_ident},
        };
        let construct = match fields {
            ast::Fields::Tuple(_) => {
                quote! {
                    #construct_ident(#(#field_ctxs),*)
                }
            }
            ast::Fields::Struct(_) => {
                let items = std::iter::zip(fields.idents(), field_ctxs.iter())
                    .map(|(ident, ctx)| quote! {#ident: #ctx});
                quote! {
                    #construct_ident {
                        #(#items),*
                    }
                }
            }
            ast::Fields::Unit => quote! {#var_ident},
        };
        quote! {
            let ctx = Box::new(move |x: ::uniplate::Tree<#typ>| {
                let ::uniplate::Tree::Many(x) = x else { panic!()};
                #construct
        });}
    }
}

fn derive_a_biplate(state: &mut ParserState) -> TokenStream2 {
    let from = state.from.to_token_stream();
    let to = state.to.to_token_stream();

    if from.to_string() == to.to_string() {
        return _derive_identity_biplate(state, from);
    }

    let tokens: TokenStream2 = match state.data.clone() {
        ast::Data::DataEnum(x) => _derive_a_enum_uniplate(state, x),
        ast::Data::DataStruct(x) => _derive_a_struct_uniplate(state, x),
    };

    let mut generics = state.data.generics().clone();
    for (typ, bounds) in generics.type_parameters.iter_mut() {
        // Add 'static bounds to all generic type parameters.
        bounds.push(syn::TypeParamBound::Verbatim(quote!('static)));

        // If we are deriving Biplate<T>, T must be Uniplate
        if to.to_string() == typ.to_token_stream().to_string() {
            bounds.push(syn::TypeParamBound::Verbatim(quote!(Uniplate)));
        }
    }

    let impl_bounds = generics.impl_parameters();
    let where_clause = generics.impl_type_where_block();

    quote! {
        impl<#impl_bounds> ::uniplate::Biplate<#to> for #from #where_clause{
            fn biplate(&self) -> (::uniplate::Tree<#to>, Box<dyn Fn(::uniplate::Tree<#to>) -> #from>) {
                #tokens
            }
        }
    }
}

fn _derive_identity_biplate(state: &mut ParserState, from: TokenStream2) -> TokenStream2 {
    let mut generics = state.data.generics().clone();
    // Add 'static bounds to all generic type parameters.
    for (_, bounds) in generics.type_parameters.iter_mut() {
        bounds.push(syn::TypeParamBound::Verbatim(quote!('static)));
    }

    let impl_bounds = generics.impl_parameters();
    let where_clause = generics.impl_type_where_block();

    quote! {
        impl<#impl_bounds> ::uniplate::Biplate<#from> for #from #where_clause{
            fn biplate(&self) -> (::uniplate::Tree<#from>, Box<dyn Fn(::uniplate::Tree<#from>) -> #from>) {
                let val = self.clone();
                (::uniplate::Tree::One(val.clone()),Box::new(move |x| {
                    let ::uniplate::Tree::One(x) = x else {todo!()};
                    x
                }))
            }
        }
    }
}
