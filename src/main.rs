use eframe::egui;
mod init;
use crate::init::RsCashM;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    let opt = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 800.0]) // Default window size
            .with_min_inner_size([400.0, 500.0]) // Prevents user from making the window too small
            .with_resizable(true) // Allows the user to resize the window
            .with_title("Rust Simple Cash Manager") // Window title
            .with_active(true) // Window starts active
            .with_maximize_button(true), // Allows maximizing the window
        ..Default::default()
    };

    eframe::run_native(
        "RSCASHM",
        opt,
        Box::new(|_cc| Ok(Box::new(RsCashM::default()))),
    )
}// 13,august,2026
