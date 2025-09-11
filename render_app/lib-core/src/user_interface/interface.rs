use std::collections::HashMap;

use lib_render::types::{Element, GeometryType, Instance};

use crate::user_interface::{UserInterface};

pub struct GraphicsInterface {
    elements: HashMap<GeometryType, Vec<Box<dyn Element>>>,
}

impl GraphicsInterface {
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
        }
    }

    pub fn show<R>(&mut self, elements_builder: impl FnOnce(&mut UserInterface) -> R) -> R {
        let mut user_interface = UserInterface { graphics_interface: self };
        elements_builder(&mut user_interface)
    }

    pub(crate) fn add_element(&mut self, element: impl Element + 'static) {
        self.elements.entry(element.geometry())
            .or_insert(Vec::new())
            .push(Box::new(element));
    }

    pub fn stage(&mut self) -> HashMap<GeometryType, Vec<Instance>> {
        let mut staged_data = HashMap::new();
        for (geometry, elements) in &self.elements {
            let mut stage_elements: Vec<Instance> = Vec::new();
            for element in elements {
                stage_elements.push(element.as_instance());
                // TODO: This will ultimately need to make the conversion from elements to instances.
            }
            staged_data.entry(*geometry).insert_entry(stage_elements);
        }
        return staged_data;
    }
}