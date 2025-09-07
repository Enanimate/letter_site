use lib_core::{run, user_interface::interface::GraphicsInterface};

fn main() {
    let mut interface = GraphicsInterface::new();

    interface.show(|ui| {
    });

    run(interface).unwrap();
}