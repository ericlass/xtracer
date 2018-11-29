use linear::Vector4F;

const PI: f64 = 3.1415926535897932384626433;

pub struct Random {
    rand_seed: u32
}

impl Random {
    pub fn new(seed: u32) -> Random {
        Random {rand_seed: seed}
    }

    //Crete random number in range 0...u32.MAX
    pub fn random (&mut self) -> u32 {
        let mut x = self.rand_seed;
        x = x ^ (x << 13);
        x = x ^ (x >> 17);
        x = x ^ (x << 5);
        self.rand_seed = x;

        x
    }

    //Create random number in range 0.0...1.0
    pub fn random_f(&mut self) -> f64 {
        self.random() as f64 * 2.3283064370807973754314699618685e-10        
    }

    //Creates point on unit sphere centered at (0,0,0) with radius 1.0.
    fn random_point_on_unit_sphere(&mut self) -> Vector4F {
        let u = self.random_f();
        let v = self.random_f();
        let theta = 2.0 * PI * u;
        let phi = (2.0 * v - 1.0).acos();
        let sin_phi = phi.sin();

        Vector4F {
            x: sin_phi * theta.cos(),
            y: sin_phi * theta.sin(),
            z: phi.cos(),
            w: 1.0
        }
    }

    //Create point on sphere centered at given pos and with given radius
    pub fn random_point_on_sphere(&mut self, pos: &Vector4F, radius: f64) -> Vector4F {
        let usp = self.random_point_on_unit_sphere();

        Vector4F {
            x: pos.x + (radius * usp.x),
            y: pos.y + (radius * usp.y),
            z: pos.z + (radius * usp.z),
            w: 1.0
        }
    }

    //Create point on hemisphere centered at (0,0,0) and with radius 1.0. The direction of the top of the hemisphere is given by n.
    pub fn random_point_on_hemisphere(&mut self, n: &Vector4F) -> Vector4F {
        let usp = self.random_point_on_unit_sphere();
        let pdotn = Vector4F::dot(&usp, n);
        if pdotn < 0.0 {usp.invert()} else {usp}
    }
}