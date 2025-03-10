use nalgebra::{Vector3, Matrix3};

#[derive(Clone)] 
pub struct NetInfo {
    pub tx_power: Option<f32>,
    pub measured_power: Option<f32>,
}

#[derive(Clone)] 
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub net_info: Option<NetInfo>,
}

impl Point {
    pub fn new(x: f32, y: f32, net_info: Option<NetInfo>) -> Point {
        return Point { x, y, net_info };
    }
}

pub struct Location {
    pub x: f32,
    pub y: f32,
}

pub struct TrilaterationCalculator {
    path_loss_exponent: f32,
}

impl Default for TrilaterationCalculator {
    fn default() -> Self {
        Self {
            path_loss_exponent: 3.0,
        }
    }
}

impl TrilaterationCalculator {
    pub fn get_location(&self, point_one: &Point, point_two: &Point, point_three: &Point) -> Location {
        // """
        // Calculates the estimated location based on the measured power in dBm and transmit power in dBm
        // and the known positions of three test points.

        // Args:
        //     point_one (borrowed Point): The Point struct (Measured Power, Transmit Power, X, Y) of test point one.
        //     point_two (borrowed Point): The Point struct (Measured Power, Transmit Power, X, Y) of test point two.
        //     point_three (borrowed Point): The Point struct (Measured Power, Transmit Power, X, Y) of test point three.

        // Returns:
        //     Location: The estimated position (X, Y)
        // """

        // Trilateration
        let estimated_location = self.trilaterate(&point_one, &point_two, &point_three);

        return estimated_location;
    }

    fn trilaterate(&self, point_one: &Point, point_two: &Point, point_three: &Point) -> Location {
        // """
        // Trilaterates the location (X, Y) given the distances, from the selected network, of three test points.

        // Args:
        //     d1 (float): distance from the first point to the unknown position.
        //     d2 (float): distance from the second point to the unknown position.
        //     d3 (float): distance from the third point to the unknown position.

        // Returns:
        //     Location: The (X, Y) coordinates of the selected network's, unknown position.
        // """

        // Use least squares to solve the equations
        let results = self.calculate_location(&point_one, &point_two, &point_three, self.path_loss_exponent).unwrap();

        // Return the estimated coordinates
        return Location {x: results.x, y: results.y};
    }

    fn get_distance(&self, net_info: Option<&NetInfo>, path_loss_exponent: f32) -> f32 {
        let base: f32 = 10.0;
    
        let network_info= net_info.unwrap();
    
        let tx_power = network_info.tx_power.unwrap();
        let measured_power = network_info.measured_power.unwrap();
    
        return base.powf((tx_power - measured_power) / (10.0 * path_loss_exponent));
    }
    
    // Trilateration with Linear Least Squares
    // System of quadratic distance equation
    fn calculate_location(&self,p1: &Point, p2: &Point, p3: &Point, path_loss_exponent: f32) -> Option<Location> {
        let x1 = p1.x;
        let y1 = p1.y;
        let r1 = self.get_distance(p1.net_info.as_ref(), path_loss_exponent);
        let x2 = p2.x;
        let y2 = p2.y;
        let r2 = self.get_distance(p2.net_info.as_ref(), path_loss_exponent);
        let x3 = p3.x;
        let y3 = p3.y;
        let r3 = self.get_distance(p3.net_info.as_ref(), path_loss_exponent);
    
        println!("d:{} d:{} d:{}", r1, r2, r3);
    
        // Constructing the matrix system
        let a = 2.0 * (x2 - x1);
        let b = 2.0 * (y2 - y1);
        let c = r1.powi(2) - r2.powi(2) - x1.powi(2) + x2.powi(2) - y1.powi(2) + y2.powi(2);
        
        let d = 2.0 * (x3 - x1);
        let e = 2.0 * (y3 - y1);
        let f = r1.powi(2) - r3.powi(2) - x1.powi(2) + x3.powi(2) - y1.powi(2) + y3.powi(2);
    
        let matrix = Matrix3::new(a, b, 0.0, d, e, 0.0, 0.0, 0.0, 1.0);
        let vector = Vector3::new(c, f, 0.0);
    
        if let Some(solution) = matrix.try_inverse().map(|inv| inv * vector) {
            return Some(Location{ x: solution.x, y: solution.y });
        }
    
        None
    }
}
