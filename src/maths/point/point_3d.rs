pub struct Point3d {
    x: f64,
    y: f64,
    z: f64,   
}

impl Point3d {
    pub fn new(x: f64, y: f64, z: f64) -> Point3d {
        Point3d { x, y, z }
    }
    
}