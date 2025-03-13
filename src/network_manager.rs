use std::process::Command;
use std::str;

use std::{thread, time};

use crate::trilateration_calc::{NetInfo, Point};

pub struct NetworkManager {
    available_networks: Vec<Network>, // Store networks in a vector
    selected_network: Option<Network>, // Store the currently selected network REPLACE WITH NETWORK STRUCT

    connected: bool, // Wether or not the user is currently connected to the desired network
}

impl Default for NetworkManager {
    fn default() -> Self {
        Self {
            available_networks: Vec::new(),
            selected_network: None,

            connected: false,
        }
    }
}

impl NetworkManager {
    pub fn ready_to_calc(&self, points: &[Point; 3]) -> bool {
        return self.get_selected_network().is_some() && points.iter().all(|point| point.net_info.is_some());
    }

    pub fn get_available_networks(&self) -> &Vec<Network> {
        return &self.available_networks;
    }

    pub fn clear_available_networks(&mut self) {
        self.available_networks.clear();
    }

    pub fn get_selected_network(&self) -> &Option<Network> {
        return &self.selected_network;
    }

    pub fn select_network(&mut self, network: Option<&Network>) {
        self.selected_network = Some(Network::from(network.unwrap()));
    }

    pub fn get_connection_status(&self) -> bool {
        return self.connected;
    }

    pub fn is_connected(&mut self, connected: bool) {
        self.connected = connected;
    }

    pub fn reset_network_manager(&mut self) {
        self.disconnect_from_network();
        self.is_connected(false);
        self.selected_network = None;
    }

    pub fn connect_to_network(&mut self, password: String) -> bool {
        let ssid = &self.get_selected_network().as_ref().unwrap().ssid;

        println!("Connecting to '{}' with password: {}", ssid, password);

        let output = Command::new("nmcli")
            .arg("dev")
            .arg("wifi")
            .arg("connect")
            .arg(ssid)
            .arg("password")
            .arg(password)
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    println!("Successfully connected to '{}'", ssid);
                } else {
                    eprintln!(
                        "Failed to connect: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    return false;
                }
            }
            Err(e) => eprintln!("Error executing nmcli: {}", e),
        }

        return true;
    }

    pub fn disconnect_from_network(&mut self) {
        let ssid = &self.get_selected_network().as_ref().unwrap().ssid;

        let output = Command::new("nmcli")
            .arg("connection")
            .arg("down")
            .arg(ssid)
            .output()
            .expect("Failed to execute nmcli");

        
        if output.status.success() {
            println!("Successfully disconnected from '{}'", ssid);
        } else {
            eprintln!(
                "Failed to disconnect: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    pub fn scan_networks(&mut self) {
        if self.get_selected_network().is_none() {
            let output = Command::new("nmcli")
            .args(&["-t", "-f", "SSID, SIGNAL, SECURITY", "dev", "wifi", "list"]) // maybe add , "list"
            .output()
            .expect("Failed to execute nmcli");
    
            if output.status.success() {
                self.clear_available_networks();
                let networks = String::from_utf8_lossy(&output.stdout);
                if networks.trim().is_empty() {
                    print!("Could not find any networks");
                } else {
                    for network in networks.lines() {
                        const NUM_OF_ARGS: usize = 3;
                        let mut parts = network.splitn(NUM_OF_ARGS, ':'); // Split SSID and SIGNAL at the colon
                        if let (Some(ssid), Some(signal), Some(security)) = (parts.next(), parts.next(), parts.next()) {
                            if ssid != "" {
    
                                let mut sec: Option<String> = None;
    
                                if security != "" {
                                    sec = Some(security.parse().unwrap());
                                }
    
                                let network = Network::new(ssid.parse().unwrap(), signal.parse().unwrap(), sec);
    
                                self.available_networks.push(network);
                                
                            }
                        }
                    }
                }
            } else {
                eprintln!("Error running nmcli: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
    }

    pub fn ping_network(&self, sample_scale: u16, sample_length: u64) -> NetInfo {
        // return NetInfo { measured_power: Some(-38.0), tx_power: Some(15.0) };
        // Execute the nmcli command to get both Tx Power and Signal Level

        let sample_length = time::Duration::from_millis(sample_length);

        let mut signal_strength = 0.0;
        let mut tx_power = 0.0;
        
        for _ in 0..sample_scale {

            let output = Command::new("iwconfig")
                .output()
                .expect("Failed to execute iwconfig");

            if !output.status.success() {
                println!("Failed to execute iwconfig");
            }

            let output_str = str::from_utf8(&output.stdout).unwrap();
            
            for line in output_str.lines() {
                if line.contains("Signal level=") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    for part in parts {
                        if part.starts_with("level=") {
                            if let Some(value_str) = part.strip_prefix("level=") {
                                if let Ok(value) = value_str.replace("dBm", "").parse::<f32>() {
                                    signal_strength += value;
                                }
                            }
                        }
                    }
                }
                if line.contains("Tx-Power=") {
                    if let Some(start) = line.find("Tx-Power=") {
                        let tx_power_part = &line[start..];
                        let parts: Vec<&str> = tx_power_part.split_whitespace().collect();
                        if parts.len() > 1 {
                            if let Ok(value) = parts[0].split('=').nth(1).unwrap_or("0").parse::<f32>() {
                                tx_power = value;
                            }
                        }
                    }
                }
            }

            thread::sleep(sample_length);
        }

        return NetInfo { measured_power: Some(signal_strength / f32::from(sample_scale)), tx_power: Some(tx_power / f32::from(sample_scale)) };
    }
}

pub struct Network {
    pub ssid: String,
    pub measured_power: u32,
    pub security: Option<String>
}

impl Network {
    pub fn new(ssid: String, measured_power: u32, security: Option<String>) -> Network {
        return Network { ssid, measured_power, security };
    }

    pub fn from(network: &Network) -> Network {
        return Network { ssid: network.ssid.clone(), measured_power: network.measured_power, security: network.security.clone() };
    }
}
