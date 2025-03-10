use std::process::Command;

pub struct NetworkManager {
    available_networks: Vec<Network>, // Store networks in a vector
    selected_network: Option<Network>, // Store the currently selected network REPLACE WITH NETWORK STRUCT
    connected: bool, // Wether or not the user is currently connected to the desired network
}

impl NetworkManager {
    pub fn default() -> Self {
        Self {
            available_networks: Vec::new(),
            selected_network: None,
            connected: false,
        }
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

        //nmcli dev wifi connect "SSID"
        //nmcli dev wifi connect "SSID" password "YourPassword"

        println!("Connecting to '{}' with password: {}", &self.get_selected_network().as_ref().unwrap().ssid, password);

        let ssid = &self.get_selected_network().as_ref().unwrap().ssid;

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
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    println!("Successfully disconnected from '{}'", ssid);
                } else {
                    eprintln!(
                        "Failed to disconnect: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }
            Err(e) => eprintln!("Error executing nmcli: {}", e),
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
