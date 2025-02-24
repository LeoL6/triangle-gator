#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{*};
use egui::{IconData, Theme, ViewportCommand};
use std::process::Command;

#[derive(Default)]
struct TriangleGator {
    available_networks: Vec<String>, // Store networks in a vector
    selected_network: String
}

fn main() -> Result {
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

impl App for TriangleGator {        
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array() // Make sure we don't paint anything behind the rounded corners
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        list_networks(self);

        custom_window_frame(ctx, "Triangle Gator", |ui| {
            ctx.set_theme(Theme::Dark);

            ui.label("Hello :P");

            ui.centered_and_justified(|ui| { // Centers the content
                egui::Frame::NONE
                .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY)) // Border thickness and color
                .outer_margin(egui::Margin::same(5)) // Space outside the border
                .inner_margin(egui::Margin::same(10)) // Space insode of the border
                .corner_radius(5.0) // Optional: Rounded corners
                .fill(egui::Color32::from_black_alpha(0))
                .show(ui, |ui| {
                    if !self.selected_network.trim().is_empty() {
                        // TRIANGLE CODE
                        let points = vec![
                            egui::Pos2::new(100.0, 50.0), // Top point
                            egui::Pos2::new(50.0, 150.0), // Bottom-left point
                            egui::Pos2::new(150.0, 150.0), // Bottom-right point
                        ];

                        // Draw the triangle using the points
                        ui.painter().add(egui::Shape::convex_polygon(
                            points, 
                            egui::Color32::from_black_alpha(0), // Color of the triangle
                            egui::Stroke::new(1.0, egui::Color32::WHITE), // No border
                        ));

                        // ui.label(self.selected_network);
                    } else {
                        egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .max_width(200.0)
                        .show(ui, |ui| {
                            // Display all networks in the network list
                            for network in &self.available_networks {
                                ui.vertical(|ui|{
                                    if ui.button(network).clicked() {
                                        println!("Selecting {}", network);
                                        self.selected_network = network.to_string();
                                    }
                                });
                            }
                        });
                    }
                });
            });

            ui.horizontal(|ui| {
                if ui.button("Place Point").clicked() { }   
                if ui.button("Reset Calculation").clicked() {
                    self.selected_network.clear();
                    list_networks(self);
                }
            });
        });
    }
}

fn list_networks(selph: &mut TriangleGator) {
    if selph.available_networks.is_empty() {
        let output = Command::new("nmcli")
        .args(&["-t", "-f", "SSID, SIGNAL", "dev", "wifi"])
        .output()
        .expect("Failed to execute nmcli");

        if output.status.success() {
            selph.available_networks.clear();
            let networks = String::from_utf8_lossy(&output.stdout);
            if networks.trim().is_empty() {
                print!("Could not find any networks");
            } else {
                for network in networks.lines() {
                    let mut parts = network.splitn(2, ':'); // Split SSID and SIGNAL at the colon
                    if let (Some(ssid), Some(signal)) = (parts.next(), parts.next()) {
                        let network_info = format!("{} (Signal: {}%)", ssid, signal);
                        selph.available_networks.push(network_info);
                    }
                }
            }
        } else {
            eprintln!("Error running nmcli: {}", String::from_utf8_lossy(&output.stderr));
        }
    }
}

fn custom_window_frame(ctx: &egui::Context, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    use egui::{CentralPanel, UiBuilder};

    let panel_frame = egui::Frame::new()
        .fill(ctx.style().visuals.window_fill())
        .corner_radius(10)
        .stroke(ctx.style().visuals.widgets.noninteractive.fg_stroke)
        .outer_margin(1); // so the stroke is within the bounds

    CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
        let app_rect = ui.max_rect();

        let title_bar_height = 32.0;
        let title_bar_rect = {
            let mut rect = app_rect;
            rect.max.y = rect.min.y + title_bar_height;
            rect
        };
        title_bar_ui(ui, title_bar_rect, title);

        // Add the contents:
        let content_rect = {
            let mut rect = app_rect;
            rect.min.y = title_bar_rect.max.y;
            rect
        }
        .shrink(4.0);
        let mut content_ui = ui.new_child(UiBuilder::new().max_rect(content_rect));
        add_contents(&mut content_ui);
    });
}

fn title_bar_ui(ui: &mut egui::Ui, title_bar_rect: eframe::epaint::Rect, title: &str) {
    use egui::{vec2, Align2, FontId, Id, PointerButton, Sense, UiBuilder};

    let painter = ui.painter();

    let title_bar_response = ui.interact(
        title_bar_rect,
        Id::new("title_bar"),
        Sense::click_and_drag(),
    );

    // Paint the title:
    painter.text(
        title_bar_rect.center(),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(20.0),
        ui.style().visuals.text_color(),
    );

    // Paint the line under the title:
    painter.line_segment(
        [
            title_bar_rect.left_bottom() + vec2(1.0, 0.0),
            title_bar_rect.right_bottom() + vec2(-1.0, 0.0),
        ],
        ui.visuals().widgets.noninteractive.bg_stroke,
    );

    // Interact with the title bar (drag to move window):
    if title_bar_response.double_clicked() {
        let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        ui.ctx()
            .send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
    }

    if title_bar_response.drag_started_by(PointerButton::Primary) {
        ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
    }

    ui.allocate_new_ui(
        UiBuilder::new()
            .max_rect(title_bar_rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
        |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.visuals_mut().button_frame = false;
            ui.add_space(8.0);
            close_ui(ui);
        },
    );
}

/// Show close button for the native window.
fn close_ui(ui: &mut egui::Ui) {
    use egui::{Button, RichText};

    let button_height = 12.0;

    let close_response = ui
        .add(Button::new(RichText::new("❌").size(button_height)))
        .on_hover_text("Close the window");
    if close_response.clicked() {
        ui.ctx().send_viewport_cmd(ViewportCommand::Close);
    }
}
