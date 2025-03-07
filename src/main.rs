#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// BROWN SUGAR OAT AMERICANO

mod trilateration_calc;

use eframe::{*};
use eframe::egui::{self, Event, Vec2, FontId, FontFamily};

use egui_plot::{Legend, PlotPoint, PlotPoints, Polygon};

use emath::Pos2;
use egui::{Button, Color32, IconData, Stroke, Theme, ViewportCommand};
use trilateration_calc::Point;
use std::process::Command;
use std::vec;

// COME UP WITH UNIQUE STANDALONE METHOD FOR DRAWING POINTS AND SUCH AS A CLIKCABLE, HOVERABLE UI POINT, PROBABLY AS ITS OWN PLOT
struct TriangleGator {
    available_networks: Vec<String>, // Store networks in a vector
    selected_network: String, // Store the currently selected network REPLACE WITH NETWORK STRUCT
    points: [Point; 3],  // Triangle points
    selected_point: Option<usize>, // Index of selected point


    lock_x: bool,
    lock_y: bool,
    ctrl_to_zoom: bool,
    shift_to_horizontal: bool,
    zoom_speed: f32,
    scroll_speed: f32,
}

impl Default for TriangleGator {
    fn default() -> Self {
        Self {
            available_networks: Vec::new(),
            selected_network: String::new(),
            points: [
                Point::new(0.0, 10.0, None),  // Top
                Point::new(100.0, 10.0, None),  // Bottom left
                Point::new(50.0, 96.0, None),  // Bottom right
            ],
            selected_point: None,

            lock_x: false,
            lock_y: false,
            ctrl_to_zoom: false,
            shift_to_horizontal: false,
            zoom_speed: 1.0,
            scroll_speed: 1.0,
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

            let mut points_scanned: u16 = 0; // I DONT KNOW, OKAY?

            ui.horizontal_centered(|ui| {
                egui::Frame::NONE
                    .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY)) // Border thickness and color
                    .outer_margin(egui::Margin::same(12)) // Margin outside the border
                    .inner_margin(egui::Margin::same(10)) // Margin inside of the border
                    .corner_radius(5.0) // Rounded Corners
                    .fill(egui::Color32::from_black_alpha(0)) // Clear Background
                    .show(ui, |ui| {
                        ui.set_min_width(200.0);

                        let (scroll, pointer_down, pointer_clicked, modifiers) = ui.input(|i| {
                            let scroll = i.events.iter().find_map(|e| match e {
                                Event::MouseWheel {
                                    unit: _,
                                    delta,
                                    modifiers: _,
                                } => Some(*delta),
                                _ => None,
                            });
                            (scroll, i.pointer.primary_down(), i.pointer.primary_clicked(), i.modifiers)
                        });

                        if !self.selected_network.trim().is_empty() {
                            egui_plot::Plot::new("plot")
                            .allow_zoom(false)
                            .allow_drag(false)
                            .allow_scroll(false)
                            .show_axes(false)
                            .legend(Legend::default())
                            .width(272.0)
                            .height(200.0)
                            .min_size(egui::vec2(0.0, 180.0))

                            // .label_formatter(|name, value| {
                            //     if name.is_empty() {
                            //         format!("{}: {:.*}%", name, 1, value.y)
                            //     } else {
                            //         "".to_owned()
                            //     }
                            // })

                            .show(ui, |plot_ui| {
                                if let Some(mut scroll) = scroll {
                                    if modifiers.ctrl == self.ctrl_to_zoom {
                                        scroll = Vec2::splat(scroll.x + scroll.y);
                                        let mut zoom_factor = Vec2::from([
                                            (scroll.x * self.zoom_speed / 10.0).exp(),
                                            (scroll.y * self.zoom_speed / 10.0).exp(),
                                        ]);
                                        if self.lock_x {
                                            zoom_factor.x = 1.0;
                                        }
                                        if self.lock_y {
                                            zoom_factor.y = 1.0;
                                        }
                                        plot_ui.zoom_bounds_around_hovered(zoom_factor);
                                    } else {
                                        if modifiers.shift == self.shift_to_horizontal {
                                            scroll = Vec2::new(scroll.y, scroll.x);
                                        }
                                        if self.lock_x {
                                            scroll.x = 0.0;
                                        }
                                        if self.lock_y {
                                            scroll.y = 0.0;
                                        }
                                        let delta_pos = self.scroll_speed * scroll;
                                        plot_ui.translate_bounds(delta_pos);
                                    }
                                }
                                if plot_ui.response().hovered() && pointer_down {
                                    let mut pointer_translate = -plot_ui.pointer_coordinate_drag_delta();
                                    if self.lock_x {
                                        pointer_translate.x = 0.0;
                                    }
                                    if self.lock_y {
                                        pointer_translate.y = 0.0;
                                    }
                                    plot_ui.translate_bounds(pointer_translate);
                                }

                                let mut points_vec = vec![];

                                self.points.iter().clone().for_each(|point| { // COMBINE THIS WITH THE HOVER DETECTION FOR BETTER EFFICIENCY
                                    points_vec.push([f64::from(point.x), f64::from(point.y)]);
                                });

                                let triangle_bounds = Polygon::new(PlotPoints::from(points_vec.clone())).fill_color(Color32::from_rgba_unmultiplied(255, 255, 255, 20)).stroke(Stroke::new(1.0, Color32::WHITE)).allow_hover(true);

                                plot_ui.polygon(triangle_bounds);

                                // HOVER DETECTION CODE
                                if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
                                    let hover_threshold = 3.0;

                                    for (index, point) in self.points.iter().enumerate() {
                                        let distance = ((pointer_pos.x - f64::from(point.x)).powi(2) + (pointer_pos.y - f64::from(point.y)).powi(2)).sqrt();

                                        if distance < hover_threshold {
                                            let rssi = point.d;
                                            // let screen_pos = plot_ui.transform().position_from_point(&pointer_pos);
                                            let screen_pos = plot_ui.transform().position_from_point(&PlotPoint::new(self.points[index].x, self.points[index].y));

                                            if rssi.is_some() {
                                                plot_ui.ctx().debug_painter().text(
                                                    screen_pos,
                                                    egui::Align2::CENTER_BOTTOM,
                                                    format!("RSSI: {:.2}", rssi.unwrap()),   // MAYBE MAKE THE DISTANCE VAR A OPTION SO U CAN SET IT TO NONE, CHECK, AND HAVE NOTING DISPLAY UNDER THE RSSI
                                                    FontId::new(12.0, FontFamily::Proportional), // SPLIT THIS INTO MULTIPLE FILES
                                                    egui::Color32::RED,
                                                );

                                                points_scanned += 1;
                                            }

                                            // CLICKING WORKS HORRAY
                                            if pointer_clicked {
                                                if self.selected_point.is_some() && self.selected_point == Some(index) {
                                                    self.selected_point = None;
                                                } else {
                                                    self.selected_point = Some(index);
                                                }
                                                // if self.selected_point.is_some() && self.selected_point == Some(point.clone()) {
                                                //     self.selected_point = None;
                                                // } else {
                                                //     self.selected_point = Some(Point::from(point));
                                                // }
                                            }
                                        }
                                    }
                                }

                                if self.selected_point.is_some() {
                                    let selected_point = self.points[self.selected_point.unwrap()].clone();

                                    let point_x = f64::from(selected_point.x);
                                    let point_y = f64::from(selected_point.y);

                                    let points_vec = vec![
                                        [point_x - 3.0, point_y],
                                        [point_x, point_y + 3.0],
                                        [point_x + 3.0, point_y],
                                        [point_x, point_y- 3.0],
                                    ];

                                    // let selected_point_circle_shape = CircleShape { center: Pos2::new(selected_point.x, selected_point.y), radius: 3.0, fill: Color32::from_rgba_unmultiplied(255, 255, 255, 50), stroke: Stroke::new(2.0, Color32::RED) };

                                    let point_bounds = Polygon::new(PlotPoints::from(points_vec)).fill_color(Color32::from_rgba_unmultiplied(255, 0, 0, 80)).stroke(Stroke::new(2.0, Color32::RED));

                                    plot_ui.polygon(point_bounds);
                                }

                                // if plot_ui.response().hovered() && pointer_down {
                                //     let mut pointer_translate = -plot_ui.pointer_coordinate_drag_delta();
                                //     if line {
                                //         pointer_translate.x = 0.0;
                                //     }
                                // }
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
                                            self.selected_network = network.to_string();
                                        }
                                    }
                                });
                            });
                        }
                    });
            });
            
            if !is_network_selected(self) {
                ui.label(self.selected_network.to_string());
            }
            
            if self.selected_network != "" {
                ui.horizontal(|ui| {
                    if ui.add_enabled(self.selected_point.is_some(),Button::new("Test Point")).clicked() {
                        let trilat_calc = trilateration_calc::TrilaterationCalculator::default();

                        // trilat_calc.test_levmar();
                        trilat_calc.test_calc();

                        self.points[self.selected_point.unwrap()].d = Some(ping_selected_network(self)); // CALC RSSI HERE AND SET IT INTO THE POINT STRUCT 

                        self.selected_point = None;
                    }   

                    if ui.add_enabled(points_scanned >= 2, Button::new("Calculate")).clicked() {
                        println!("CALCULATINGGGG!!!");
                    }

                    if ui.button("Reset Calculation").clicked() {
                        self.selected_network.clear();
                        list_networks(self);
                    }
                });
            }
        });
    }
}

fn ping_selected_network(selph: &mut TriangleGator) -> f32 {
    return 5.0;
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
