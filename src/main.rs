#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{*};
use emath::Pos2;
use egui::{IconData, Theme, Ui, ViewportCommand};
use std::process::Command;

struct TriangleGator {
    available_networks: Vec<String>, // Store networks in a vector
    selected_network: String, // Store the currently selected network
    points: [Pos2; 3],  // Triangle vertices
    selected_side: Option<usize>, // Index of selected side
}

impl Default for TriangleGator {
    fn default() -> Self {
        Self {
            available_networks: Vec::new(),
            selected_network: String::new(),
            points: [
                Pos2::new(150.0, 114.0),  // Top
                Pos2::new(100.0, 200.0),  // Bottom left
                Pos2::new(200.0, 200.0),  // Bottom right
            ],
            selected_side: None,
        }
    }
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

            ui.horizontal_centered(|ui| {
                egui::Frame::NONE
                    .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY)) // Border thickness and color
                    .outer_margin(egui::Margin::same(12)) // Space outside the border
                    .inner_margin(egui::Margin::same(10)) // Space inside of the border
                    .corner_radius(5.0) // Optional: Rounded corners
                    .fill(egui::Color32::from_black_alpha(0))
                    .show(ui, |ui| {
                        ui.set_min_width(200.0);
            
                        if !self.selected_network.trim().is_empty() {
                            ui.horizontal(|ui| {
                                if ui.button("Point 1").clicked() { }   
                                if ui.button("Point 2").clicked() { }   
                                if ui.button("Point 3").clicked() { }   
                            });

                            // TRIANGLE LOGIC | NOT COMPLETE YET | WIP, I DONT GIVE A FUCK
                            // Define canvas size 
                            // let canvas_size = egui::vec2(200.0, 200.0);
                            // let (rect, response) = ui.allocate_exact_size(canvas_size, egui::Sense::click_and_drag());
            
                            // let painter = ui.painter();
                            // let stroke = egui::Stroke::new(2.0, egui::Color32::WHITE);
            
                            // // Draw triangle edges
                            // for i in 0..3 {
                            //     let p1 = self.points[i];
                            //     let p2 = self.points[(i + 1) % 3];
                            //     let color = if self.selected_side == Some(i) {
                            //         egui::Color32::RED  // Highlight selected side
                            //     } else {
                            //         egui::Color32::WHITE
                            //     };
                            //     painter.line_segment([p1, p2], egui::Stroke::new(3.0, color));
                            // }
            
                            // // Handle clicks on the canvas
                            // if response.clicked() {
                            //     if let Some(mouse_pos) = response.interact_pointer_pos() {
                            //         for i in 0..3 {
                            //             let p1 = self.points[i];
                            //             let p2 = self.points[(i + 1) % 3];
            
                            //             let closest_point = closest_point_on_line_segment(mouse_pos, p1, p2);
                            //             let distance = mouse_pos.distance(closest_point);
                            //             let threshold = 5.0; // Click sensitivity
            
                            //             if distance < threshold {
                            //                 self.selected_side = if Some(i) == self.selected_side { None } else { Some(i) };
                            //                 break;
                            //             }
                            //         }
                            //     }
                            // }
            
                            // // Show which side is selected
                            // if let Some(side) = self.selected_side {
                            //     ui.label(format!("Selected Side: {}", side + 1));
                            // } else {
                            //     ui.label("No side selected");
                            // }
                        } else {
                            egui::ScrollArea::vertical()
                                .max_height(200.0)
                                .show(ui, |ui| {
                                    ui.vertical_centered(|ui| {
                                        for network in &self.available_networks {
                                            if ui.button(network).clicked() {
                                                println!("Selecting {}", network);
                                                self.selected_network = network.to_string();
                                            }
                                        }
                                    });
                                });
                        }
                    });
            
                if !is_network_selected(self) {
                    ui.label(self.selected_network.to_string());
                }
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
    if is_network_selected(selph) {
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

fn is_network_selected(selph: &mut TriangleGator) -> bool {
    return selph.selected_network.trim().is_empty();
}

// Function to calculate the closest point on a line segment from a given point
fn closest_point_on_line_segment(point: Pos2, start: Pos2, end: Pos2) -> Pos2 {
    let line_vec = end - start;
    let point_vec = point - start;

    // Project the point onto the line (clamped between the start and end points)
    let t = (point_vec.x * line_vec.x + point_vec.y * line_vec.y) / (line_vec.x * line_vec.x + line_vec.y * line_vec.y);
    let t = t.clamp(0.0, 1.0);

    // Calculate the closest point on the line
    start + t * line_vec
}


// ===========================================================================================================





// ===========================================================================================================









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
