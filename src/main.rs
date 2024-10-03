use libloading::Library;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Load the libcudaimg library
    let lib_path = Path::new("data/libcudaimg.dll");
    let libcudaimg = unsafe { Library::new(lib_path)? };

    let options = eframe::NativeOptions {
        vsync: true,
        hardware_acceleration: eframe::HardwareAcceleration::Required,
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Image Processing Utility",
        options,
        Box::new(|_cc| Ok(Box::new(img_utils::app::MyApp::new(libcudaimg)))),
    );

    Ok(())
}
