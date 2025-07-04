#[cfg(not(target_arch = "wasm32"))]
use wrowalk::MyApp;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    eframe::run_native(
        "Wrowalk",
        Default::default(),
        Box::new(|cc| Ok(Box::new(MyApp::new(cc.egui_ctx.clone())))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    println!("This demo is not meant to be compiled for WASM.");
}
