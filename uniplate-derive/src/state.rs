//! Global(ish) Parser State variables

use crate::{ast::InstanceMeta, prelude::*};
use std::collections::VecDeque;

use self::ast::HasBaseType;

/// State variables for biplate derive.
#[derive(Debug)]
pub struct ParserState {
    /// The type we are deriving Biplate on.
    pub from: ast::PlateableType,

    /// The current target type.
    pub to: Option<ast::PlateableType>,

    /// The data structure itself.
    pub data: ast::Data,

    /// Information about the current instance being generated.
    pub current_instance: Option<ast::InstanceMeta>,

    /// Instances left to generate.
    pub instances_to_generate: VecDeque<ast::InstanceMeta>,

    /// Instances generated
    pub instances_generated: VecDeque<ast::InstanceMeta>,
}

impl ParserState {
    pub fn new(inp: ast::DeriveInput) -> Self {
        let data = inp.data;
        let from: ast::PlateableType = data.clone().into();

        let mut instances_to_generate: VecDeque<ast::InstanceMeta> = inp.instance_metadata.into();

        // always generate Biplate<From,From>
        instances_to_generate.push_front(ast::InstanceMeta::Biplate(ast::BiplateInstanceMeta {
            to: ast::Type::Plateable(from.clone()),
            walk_into: Vec::new(),
        }));

        Self {
            current_instance: None,
            to: None,
            instances_to_generate,
            instances_generated: Default::default(),
            from,
            data,
        }
    }

    pub fn next_instance(&mut self) -> Option<()> {
        let next_instance = self.instances_to_generate.pop_back();
        if let Some(current_instance) = self.current_instance.take() {
            self.instances_generated.push_back(current_instance);
        }

        self.current_instance = next_instance;

        self.to = match &self.current_instance {
            Some(ast::InstanceMeta::Uniplate(_)) => Some(self.from.clone()),
            Some(ast::InstanceMeta::Biplate(b)) => {
                let ast::Type::Plateable(t) = b.clone().to else {
                    // TODO: better error for this?
                    // probably should be in the parser not here!
                    unreachable!();
                };
                Some(t)
            }
            None => None,
        };

        self.current_instance.as_ref()?;

        Some(())
    }

    /// Checks if a type can be walked into or not
    ///
    /// This acts similarly to ast::InstanceMeta::walk_into_type but also considers the
    /// current to and from type as walkable.
    pub fn walk_into_type(&self, typ: &ast::Type) -> bool {
        let res = self.walk_into_type_inner(typ);
        // trace walk_into calls if cfg value uniplate_trace = "walkinto" is set
        //
        // to enable it on build: RUSTFLAGS='--cfg=uniplate_trace="walkinto"' cargo ...
        if cfg!(uniplate_trace = "walkinto") {
            let from = &self.from;
            let to = &self.to;
            if res {
                eprintln!(
                    "[walk-into+] Instance from {from} to {to}: walking into type {typ}",
                    from = quote!(#from),
                    to = quote!(#to),
                    typ = quote!(#typ)
                );
            } else {
                let from_basetype = from.base_typ();
                let to_basetype = to.clone().unwrap().base_typ();
                eprintln!(
                    "[walk-into-] Instance from {from} to {to}: NOT walking into type {typ}",
                    from = quote!(#from),
                    to = quote!(#to),
                    typ = quote!(#typ)
                );
                eprintln!(
                    "             .. from base type is {}",
                    quote!(#from_basetype)
                );
                eprintln!("             .. to base type is {}", quote!(#to_basetype));
                match typ {
                    ast::Type::BoxedPlateable(boxed_plateable_type) => {
                        let box_type = &boxed_plateable_type.box_typ;
                        let wrapper_type = &boxed_plateable_type.inner_typ.wrapper_typ;
                        let base_type = &boxed_plateable_type.inner_typ.base_typ;
                        eprintln!("             .. target type is a boxed plateable type");
                        eprintln!("             .. box type is {}", quote!(#box_type));
                        eprintln!("             .. wrapper type is {}", quote!(#wrapper_type));
                        eprintln!("             .. base type is {}", quote!(#base_type));
                    }
                    ast::Type::Plateable(plateable_type) => {
                        let wrapper_type = &plateable_type.wrapper_typ;
                        let base_type = &plateable_type.base_typ;
                        eprintln!("             .. target type is a plateable type");
                        eprintln!("             .. wrapper type is {}", quote!(#wrapper_type));
                        eprintln!("             .. base type is {}", quote!(#base_type));
                    }
                    ast::Type::Unplateable => {
                        eprintln!("             .. target type is an unplateable type")
                    }
                }
            }
        }

        res
    }

    #[inline(always)]
    fn walk_into_type_inner(&self, typ: &ast::Type) -> bool {
        let Some(base_typ) = typ.base_typ() else {
            return false;
        };

        if base_typ == self.to.clone().expect("").base_typ() {
            return true;
        };

        if base_typ == self.from.base_typ() {
            return true;
        };

        self.current_instance.clone().expect("").walk_into_type(typ)
    }

    /// Returns a reference to the uniplate instance
    ///
    /// # Panics
    ///
    /// If there are more than one uniplate instances
    pub fn get_uniplate_instance(&self) -> &InstanceMeta {
        if let Some(instance @ InstanceMeta::Uniplate(_)) = &self.current_instance {
            return instance;
        } else {
            for instance in &self.instances_to_generate {
                if let InstanceMeta::Uniplate(_) = instance {
                    return instance;
                }
            }

            for instance in &self.instances_generated {
                if let InstanceMeta::Uniplate(_) = &instance {
                    return instance;
                }
            }
        }
        unreachable!();
    }
}
