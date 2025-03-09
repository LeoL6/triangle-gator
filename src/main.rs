use eframe::{run_native, NativeOptions};
use egui::IconData;

use triangle_gator::TriangleGator;

fn main() -> Result<(), eframe::Error> {
    let icon_image = image::open("assets/narly.png").expect("Should be able to open icon PNG file");
    let width = icon_image.width();
    let height = icon_image.height();
    let icon_rgba8 = icon_image.into_rgba8().to_vec();
    let icon_data =IconData{
            rgba: icon_rgba8,
            width,
            height,
    };
        
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
        .with_inner_size([330.0, 400.0])
        .with_resizable(false)
        .with_decorations(false)
        .with_transparent(true)
        .with_icon(icon_data),
        ..Default::default()
    };

    run_native(
        "Triangle Gator", 
        options, 
        Box::new(|_cc| {
            Ok(Box::new(TriangleGator::default()))
        }) 
    )
}
