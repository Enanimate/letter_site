#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
fn web_entry() {
    use lib_core::{user_interface::interface::GraphicsInterface};
    use lib_core::{run_web};

    let mut interface = GraphicsInterface::new();

    interface.show(|ui| {
    });

    run_web(interface).unwrap_throw();
}