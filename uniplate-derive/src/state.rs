//! Global(ish) Parser State variables

use crate::prelude::*;
use std::collections::VecDeque;

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
}
