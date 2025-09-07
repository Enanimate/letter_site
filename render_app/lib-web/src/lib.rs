#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
fn web_entry() {
    use lib_core::{user_interface::interface::GraphicsInterface};
    use lib_core::{run_web};

    let mut interface = GraphicsInterface::new();

    interface.show(|ui| {
        ui.add_panel([0.0, 0.0, 1.0], [0.5, 0.5]);
        ui.add_panel([0.5, 0.5, 1.0], [0.5, 0.5]);
    });

    run_web(interface).unwrap_throw();
}