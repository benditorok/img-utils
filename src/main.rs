use libloading::Library;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Load the libcudaimg library
    let lib_path = Path::new("data/libcudaimg.dll");
    let libcudaimg = unsafe { Library::new(lib_path)? };

    let options = eframe::NativeOptions {
        vsync: true,
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Image Processing Utility",
        options,
        Box::new(|_cc| {
            //egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(img_utils::app::MyApp::new(libcudaimg)))
        }),
    );

    Ok(())
}
