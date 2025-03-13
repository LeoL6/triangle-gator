#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// BROWN SUGAR OAT AMERICANO

// MAKE SELECTED NETWORK AND AVAILABLE NETWORKS INTO STRUCT THAT HAS NAME, SIGNAL STRENGTH, TX, SECURITY TYPE

// THEN IF THERE IS SECURITY TYPE, ENTER PASSWORD OR SUM, AND THEN LOG IN WITH "nmcli dev wifi connect "SSID"" or nmcli dev wifi connect "SSID" password "YourPassword"

pub mod network_manager;
pub mod trilateration_calc;

use std::ops::RangeInclusive;

use network_manager::{NetworkManager, Network};
use trilateration_calc::{Location, NetInfo, Point, TrilaterationCalculator};

use eframe::{*};
use eframe::egui::{self, Event, Vec2, FontId, FontFamily};

use egui_plot::{Legend, Plot, PlotPoint, PlotPoints, PlotUi, Polygon};

use egui::{Button, Color32, DragValue, Pos2, Stroke, TextEdit, Theme, ViewportCommand, Align, Layout};

// COME UP WITH UNIQUE STANDALONE METHOD FOR DRAWING POINTS AND SUCH AS A CLIKCABLE, HOVERABLE UI POINT, PROBABLY AS ITS OWN PLOT

pub struct TriangleGator {
    // available_networks: Vec<Network>, // Store networks in a vector
    // selected_network: Option<Network>, // Store the currently selected network REPLACE WITH NETWORK STRUCT
    // connected: bool, // Wether or not the user is currently connected to the desired network
    
    network_manager: network_manager::NetworkManager,
    trilat_calc: trilateration_calc::TrilaterationCalculator,

    network_password: String, // Network password (if there is one)

    points: [Point; 3],  // Triangle points
    path_loss_exponent: f32, // User defined path loss exponent
    selected_point: Option<usize>, // Index of selected point
    calculated_location: Option<Location>, // Calculated Location of Network.

    sample_scale: u16,
    sample_length: u64,

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
            // available_networks: Vec::new(),
            // selected_network: None,
            // connected: false,

            network_manager: NetworkManager::default(),
            trilat_calc: TrilaterationCalculator::default(),

            network_password: String::from(""),

            points: [
                Point::new(0.0, 0.0, None),  // Bottom left
                Point::new(100.0, 0.0, None),  // Bottom right
                Point::new(50.0, 86.0, None),  // Top
            ],
            path_loss_exponent: 2.5, // Default path loss exponent of 2.5
            selected_point: None,
            calculated_location: None,

            sample_scale: 5,
            sample_length: 100,

            lock_x: false,
            lock_y: false,
            ctrl_to_zoom: false,
            shift_to_horizontal: false,
            zoom_speed: 1.0,
            scroll_speed: 1.0,
        }
    }
}

impl App for TriangleGator {        
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array() // Make sure we don't paint anything behind the rounded corners
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        if !self.network_manager.get_connection_status() {
            self.network_manager.scan_networks();
        }

        custom_window_frame(ctx, "Triangle Gator", |ui| {
            ctx.set_theme(Theme::Dark);

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

                        if self.network_manager.get_selected_network().is_some() && self.network_manager.get_connection_status() {
                            Plot::new("plot")
                            .allow_zoom(false)
                            .allow_drag(false)
                            .allow_scroll(false)
                            .show_axes(false)
                            .legend(Legend::default())
                            .width(272.0)
                            .height(200.0)
                            .min_size(egui::vec2(0.0, 180.0))
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

                                let triangle_bounds = Polygon::new(PlotPoints::from(points_vec.clone())).allow_hover(false).fill_color(Color32::from_rgba_unmultiplied(255, 255, 255, 20)).stroke(Stroke::new(1.0, Color32::WHITE)).allow_hover(true);

                                plot_ui.polygon(triangle_bounds);

                                // HOVER DETECTION CODE
                                if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
                                    for (index, point) in self.points.iter().enumerate() {
                                        if point_is_hovered(point, pointer_pos) {
                                            let net_info = point.net_info.as_ref();
                                            // let screen_pos = plot_ui.transform().position_from_point(&pointer_pos);
                                            let screen_pos = plot_ui.transform().position_from_point(&PlotPoint::new(self.points[index].x, self.points[index].y));

                                            if net_info.is_some() {
                                                let measured_power: Option<f32> = net_info.unwrap().measured_power;

                                                plot_ui.ctx().debug_painter().text(
                                                    screen_pos,
                                                    egui::Align2::LEFT_TOP,
                                                    format!("RSSI: {:.2}", measured_power.unwrap()),   // MAYBE MAKE THE DISTANCE VAR A OPTION SO U CAN SET IT TO NONE, CHECK, AND HAVE NOTING DISPLAY UNDER THE RSSI
                                                    FontId::new(12.0, FontFamily::Proportional), // SPLIT THIS INTO MULTIPLE FILES
                                                    egui::Color32::RED,
                                                );
                                            }

                                            // CLICKING WORKS HORRAY
                                            if pointer_clicked {
                                                if self.selected_point.is_some() && self.selected_point == Some(index) {
                                                    self.selected_point = None;
                                                } else {
                                                    self.selected_point = Some(index);
                                                }
                                            }
                                        }
                                    }
                                }

                                if self.selected_point.is_some() {
                                    let selected_point = self.points[self.selected_point.unwrap()].clone();

                                    plot_point(plot_ui, selected_point.x, selected_point.y);
                                }

                                if self.calculated_location.is_some() {
                                    let calculated_loc = self.calculated_location.as_ref().unwrap();
                                    
                                    plot_point(plot_ui, calculated_loc.x, calculated_loc.y);
                                }

                                // if plot_ui.response().hovered() && pointer_down {
                                //     let mut pointer_translate = -plot_ui.pointer_coordinate_drag_delta();
                                //     if line {
                                //         pointer_translate.x = 0.0;
                                //     }
                                // }
                            });
                        } else {
                            let mut selected_network = None;
                            egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    for network in self.network_manager.get_available_networks() {
                                        if ui.button(network.ssid.clone()).clicked() {
                                            selected_network = Some(Network::from(network));
                                        }
                                    }
                                });
                            });

                            if let Some(selected_network) = selected_network {
                                self.network_manager.select_network(Some(&selected_network));
                                self.network_password = String::from("");
                            }
                        }
                    });
            });
            
            if self.network_manager.get_selected_network().is_some() {
                if self.network_manager.get_connection_status() {
                    let ready_to_scan = self.network_manager.ready_to_calc(&self.points);

                    ui.columns(3, |ui| {
                        ui[0].vertical_centered(|ui| {
                            if ui.add_enabled(self.selected_point.is_some(), Button::new("Test Point")).clicked() {
                                let net_info = get_selected_netinfo(&self.network_manager, self.sample_scale, self.sample_length);
        
                                self.points[self.selected_point.unwrap()].net_info = Some(net_info);
                                
                                self.selected_point = None;
                            }   
                        });

                        ui[1].vertical_centered(|ui| {
                            if ui.add_enabled(ready_to_scan, Button::new("Calculate")).clicked() {
                                // Set path loss exponent to user input right before calculation
                                self.trilat_calc.set_path_loss_exponent(self.path_loss_exponent);

                                let location = self.trilat_calc.get_location(&self.points[0], &self.points[1], &self.points[2]);
        
                                println!("Estimated WAP Location: ({:.2}, {:.2})", location.x, location.y);
        
                                self.calculated_location = Some(location);

                                reset_netinfo(self);
                            }
                        });
                        
                        ui[2].vertical_centered(|ui| {
                            if ui.button("Reset").clicked() {
                                reset_calc(self);
                                self.network_manager.scan_networks();
                            }
                        });
                    });

                    if self.selected_point.is_some() {
                        ui.columns(2, |ui| {
                            ui[0].vertical_centered(|ui| {
                                ui.label("Sample Scale");
                                ui.add(DragValue::new(&mut self.sample_scale).speed(1).range(RangeInclusive::new(1, 20)));
                            });

                            ui[1].vertical_centered(|ui| {
                                ui.label("Sample Length");
                                ui.add(DragValue::new(&mut self.sample_length).speed(1).range(RangeInclusive::new(1, 2000)));
                            });
                        });
                    }

                    if ready_to_scan {
                        ui.vertical_centered(|ui| {
                            ui.label("Path Loss Exponent");
                            ui.add(DragValue::new(&mut self.path_loss_exponent).speed(0.1).range(RangeInclusive::new(2.0, 5.0)));
                        });
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        let selected_network = self.network_manager.get_selected_network().as_ref().unwrap();

                        ui.label(selected_network.ssid.clone());

                        if selected_network.security.is_some() {
                                let password_field = TextEdit::singleline(&mut self.network_password).desired_width(100.0).hint_text("password");
                                ui.add(password_field); 
                        }

                        if ui.button("Connect").clicked() {
                            let connected = self.network_manager.connect_to_network(self.network_password.clone());
                            self.network_manager.is_connected(connected);
                            
                        }

                        // if ui.button("Test").clicked() {
                        //     self.network_manager.is_connected(true);
                        // }
                    });
                }
            }
            
            ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                ui.hyperlink_to("Open Source Project by Leonardo Lees", "https://github.com/LeoL6/triangle-gator");
            });
        });
    }
}

// FUNCTIONS TO CHECK SEC OF NETWORK, CONNECT / LOGIN, AND THEN PING THE NETWORK TO GET THE SELECTED NETINFO

// MAYBE ALSO A LOADING KINDA SWIRL OR BAR THING, THAT DISPLAYS WHILE TESTING A POINT, ONCE EVERY quarter SECOND, LIKE 5 TIMES
// Im thinking, little bar graph, also disable reset_calc when bar graph is testing

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

fn point_is_hovered(point: &Point, pointer_pos: PlotPoint) -> bool {
    let hover_threshold = 3.0;

    let distance = ((pointer_pos.x - f64::from(point.x)).powi(2) + (pointer_pos.y - f64::from(point.y)).powi(2)).sqrt();

    return distance < hover_threshold;
}

fn plot_point(plot_ui: &mut PlotUi, x: f32, y: f32) {
    let point_x = f64::from(x);
    let point_y = f64::from(y);

    let points_vec = vec![
        [point_x - 3.0, point_y],
        [point_x, point_y + 3.0],
        [point_x + 3.0, point_y],
        [point_x, point_y- 3.0],
    ];

    let point_bounds = Polygon::new(PlotPoints::from(points_vec)).allow_hover(false).fill_color(Color32::from_rgba_unmultiplied(255, 0, 0, 80)).stroke(Stroke::new(2.0, Color32::RED));

    plot_ui.polygon(point_bounds);
}

fn get_selected_netinfo(network_manager: &NetworkManager, sample_scale: u16, sample_length: u64) -> NetInfo{
    return network_manager.ping_network(sample_scale, sample_length);
}

fn reset_calc(selph: &mut TriangleGator) {
    selph.network_manager.reset_network_manager();
    selph.network_password = String::from("");
    selph.calculated_location = None;

    reset_netinfo(selph);
}

fn reset_netinfo(selph: &mut TriangleGator) {
    for i in 0..3 {
        selph.points[i].net_info = None;
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