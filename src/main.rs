use eframe::*;
use egui::{CentralPanel, IconData};

#[derive(Default)]
struct TriangleGator {}

fn main() -> Result {
    // let icon_data = IconData::from("path/to/icon.png"); // Load an icon from a file

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
        .with_inner_size([600.0, 600.0])
        .with_resizable(false),
        // .with_icon(icon_data),
        ..Default::default()
    };

    run_native(
        "Triangle Gator", 
        options, 
        Box::new(|cc| {
            Ok(Box::new(TriangleGator::default()))
        }) 
    )
}

impl App for TriangleGator {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello!");
        });
    }
}
