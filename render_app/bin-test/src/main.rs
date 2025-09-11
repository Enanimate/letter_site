use lib_core::{run, types::Propogate, user_interface::interface::GraphicsInterface};

fn main() {
    let mut interface = GraphicsInterface::new();

    interface.show(|ui| {
        ui.add_panel([0.0, 0.0, 1.0], [0.5, 0.5]);
        ui.add_button([0.0, 0.0, 1.0], [0.25, 0.25], test);
    });

    run(interface).unwrap();
}

fn test() -> Propogate {
    return Propogate::Ok;
}