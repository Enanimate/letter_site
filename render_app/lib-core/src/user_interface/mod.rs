use crate::user_interface::{elements::Panel, interface::GraphicsInterface};

pub mod interface;
pub mod elements;

pub struct UserInterface<'a> {
    graphics_interface: &'a mut GraphicsInterface
}

impl<'a> UserInterface<'a> {
    pub fn add_panel(&mut self, position: [f32; 3], scale: [f32; 2]) {
        let element = Panel::new(position, scale);
        self.graphics_interface.add_element(element);
    }
}