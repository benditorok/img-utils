use libloading::Library;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Load the libcudaimg library
    let lib_path = Path::new("data/libcudaimg.dll");
    let libcudaimg = unsafe { Library::new(lib_path)? };

    let options = eframe::NativeOptions {
        vsync: true,
        hardware_acceleration: eframe::HardwareAcceleration::Required,
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "Image Processing Utility",
        options,
        Box::new(|_cc| Ok(Box::new(img_utils::app::MyApp::new(libcudaimg)))),
    ) {
        eprintln!("Failed to run eframe native: {:?}", e);
    }

    Ok(())
}
