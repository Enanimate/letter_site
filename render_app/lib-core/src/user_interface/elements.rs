use lib_render::types::{Element, GeometryType, Instance};

pub struct Panel {
    geometry_type: GeometryType,
    position: [f32; 3],
    scale: [f32; 2]
}

impl Panel {
    pub(crate) fn new(position: [f32; 3], scale: [f32; 2]) -> Self {
        Self {
            geometry_type: GeometryType::Quadrilateral,
            position,
            scale
        }
    }
}

impl Element for Panel {
    fn geometry(&self) -> GeometryType {
        self.geometry_type
    }

    fn as_instance(&self) -> lib_render::types::Instance {
        Instance {
            position: [self.position[0], self.position[1]],
            scale: self.scale,
        }
    }
}