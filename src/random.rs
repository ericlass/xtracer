use linear::Vector4F;
use rand::Rng;

const PI: f64 = 3.1415926535897932384626433;

pub struct Random {
}

impl Random {
    pub fn new() -> Random {
        Random {}
    }

    //Crete random number in range 0...u32.MAX
    pub fn random(&mut self) -> u32 {
        rand::thread_rng().gen()
    }

    //Create random number in range 0.0...1.0
    pub fn random_f(&mut self) -> f64 {
        rand::thread_rng().gen()
    }

    fn random_samples(&mut self, num_samples: u32) -> Vec<(f64, f64)> {
        let sample_width = 1.0 / num_samples as f64;
        let half_width = sample_width * 0.5;

        let mut result = Vec::with_capacity((num_samples * num_samples) as usize);
        for y in 0..num_samples {
            for x in 0..num_samples {
                let scatter = half_width * (self.random_f() - 0.5);
                let offset = half_width + scatter;
                let vx = (x as f64 * sample_width) + offset;
                let vy = (y as f64 * sample_width) + offset;

                result.push((vx, vy));
            }
        }

        result
    }

    pub fn random_directions_in_hemisphere(&mut self, num_samples: u32, n: &Vector4F) -> Vec<Vector4F> {
        //TODO: Do this inline and get &mut vec from outside to fill to avoid creating millions of vecs
        let samples = self.random_samples(num_samples);

        let mut result = Vec::new();
        for sample in samples {
            let u = sample.0;
            let v = sample.1;
            let theta = 2.0 * PI * u;
            let phi = (2.0 * v - 1.0).acos();
            let sin_phi = phi.sin();

            let dir = Vector4F {
                x: sin_phi * theta.cos(),
                y: sin_phi * theta.sin(),
                z: phi.cos(),
                w: 1.0,
            };

            let pdotn = Vector4F::dot(&dir, n);
            if pdotn < 0.0 {
                result.push(dir.invert());
            }
            else {
                result.push(dir);
            }
        }

        result
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
            w: 1.0,
        }
    }

    pub fn random_direction(&mut self) -> Vector4F {
        self.random_point_on_unit_sphere()
    }

    //Create point on sphere centered at given pos and with given radius
    pub fn random_point_on_sphere(&mut self, pos: &Vector4F, radius: f64) -> Vector4F {
        let usp = self.random_point_on_unit_sphere();

        Vector4F {
            x: pos.x + (radius * usp.x),
            y: pos.y + (radius * usp.y),
            z: pos.z + (radius * usp.z),
            w: 1.0,
        }
    }

    //Create point on hemisphere centered at (0,0,0) and with radius 1.0. The direction of the top of the hemisphere is given by n.
    pub fn random_point_on_hemisphere(&mut self, n: &Vector4F) -> Vector4F {
        let usp = self.random_point_on_unit_sphere();
        let pdotn = Vector4F::dot(&usp, n);
        if pdotn < 0.0 {
            usp.invert()
        } else {
            usp
        }
    }
}
