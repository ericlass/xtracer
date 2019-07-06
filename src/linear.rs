use settings::Color;
use std::cmp::PartialEq;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::ops::Add;
use std::ops::Mul;
use std::ops::Sub;

const PI: f64 = 3.1415926535897932384626433;

#[repr(C, packed)]
pub struct Vector4F {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Vector4F {
    pub fn new(x: f64, y: f64, z: f64) -> Vector4F {
        Vector4F { x, y, z, w: 1.0 }
    }

    pub fn null() -> Vector4F {
        Vector4F {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        }
    }

    pub fn copy(v: &Vector4F) -> Vector4F {
        Vector4F {
            x: v.x,
            y: v.y,
            z: v.z,
            w: v.w,
        }
    }

    pub fn add(v1: &Vector4F, v2: &Vector4F) -> Vector4F {
        Vector4F {
            x: v1.x + v2.x,
            y: v1.y + v2.y,
            z: v1.z + v2.z,
            w: v1.w + v2.w,
        }
    }

    pub fn sub(v1: &Vector4F, v2: &Vector4F) -> Vector4F {
        Vector4F {
            x: v1.x - v2.x,
            y: v1.y - v2.y,
            z: v1.z - v2.z,
            w: v1.w - v2.w,
        }
    }

    pub fn dot(v1: &Vector4F, v2: &Vector4F) -> f64 {
        v1.x * v2.x + v1.y * v2.y + v1.z * v2.z
    }

    pub fn cross(v1: &Vector4F, v2: &Vector4F) -> Vector4F {
        Vector4F {
            x: v1.y * v2.z - v2.y * v1.z,
            y: v1.z * v2.x - v2.z * v1.x,
            z: v1.x * v2.y - v2.x * v1.y,
            w: 1.0,
        }
    }

    pub fn half(v1: &Vector4F, v2: &Vector4F) -> Vector4F {
        (v1 + v2).normalize()
    }

    pub fn project_scalar(v1: &Vector4F, v2: &Vector4F) -> f64 {
        Vector4F::dot(v1, v2) / v2.sqr_len()
    }

    pub fn project(v1: &Vector4F, v2: &Vector4F) -> Vector4F {
        let ps = Vector4F::project_scalar(v1, v2);

        Vector4F {
            x: v2.x * ps,
            y: v2.y * ps,
            z: v2.z * ps,
            w: v2.w,
        }
    }

    pub fn reflect(i: &Vector4F, n: &Vector4F) -> Vector4F {
        let dot = Vector4F::dot(n, i);

        Vector4F {
            x: i.x - (2.0 * n.x * dot),
            y: i.y - (2.0 * n.y * dot),
            z: i.z - (2.0 * n.z * dot),
            w: 1.0,
        }
    }

    pub fn refract(i: &Vector4F, n: &Vector4F, eta: f64) -> Vector4F {
        let cosi = Vector4F::dot(n, i);
        let cost2 = 1.0 - eta * eta * (1.0 - cosi * cosi);

        if cost2 < 0.0 {
            Vector4F {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 0.0,
            }
        } else {
            let cost2sqrt = cost2.sqrt();
            Vector4F {
                x: eta * i.x - ((eta * cosi + cost2sqrt) * n.x),
                y: eta * i.y - ((eta * cosi + cost2sqrt) * n.y),
                z: eta * i.z - ((eta * cosi + cost2sqrt) * n.z),
                w: 1.0,
            }
        }
    }

    pub fn normalize(&self) -> Vector4F {
        let len = self.len();

        //Avoid division by 0
        if len == 0.0 {
            return Vector4F::null();
        }

        Vector4F {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
            w: self.w,
        }
    }

    pub fn invert(&self) -> Vector4F {
        Vector4F {
            x: self.x * -1.0,
            y: self.y * -1.0,
            z: self.z * -1.0,
            w: self.w,
        }
    }

    pub fn rotate_x(&self, angle: f64) -> Vector4F {
        let rads = (angle / 180.0) * PI;
        let sin = rads.sin();
        let cos = rads.cos();

        Vector4F {
            x: self.x,
            y: self.y * cos - self.z * sin,
            z: self.z * cos + self.y * sin,
            w: self.w,
        }
    }

    pub fn rotate_y(&self, angle: f64) -> Vector4F {
        let rads = (angle / 180.0) * PI;
        let sin = rads.sin();
        let cos = rads.cos();

        Vector4F {
            x: self.x * cos - self.z * sin,
            y: self.y,
            z: self.z * cos + self.x * sin,
            w: self.w,
        }
    }

    pub fn rotate_z(&self, angle: f64) -> Vector4F {
        let rads = (angle / 180.0) * PI;
        let sin = rads.sin();
        let cos = rads.cos();

        Vector4F {
            x: self.x * cos - self.y * sin,
            y: self.y * cos + self.x * sin,
            z: self.z,
            w: self.w,
        }
    }

    pub fn len(&self) -> f64 {
        self.sqr_len().sqrt()
    }

    pub fn sqr_len(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z)
    }

    pub fn clone(&self) -> Vector4F {
        Vector4F {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}

impl Display for Vector4F {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let x = self.x;
        let y = self.y;
        let z = self.z;
        let w = self.w;
        write!(f, "[{},{},{},{}]", x, y, z, w)
    }
}

impl Add for Vector4F {
    type Output = Vector4F;

    fn add(self, other: Vector4F) -> Vector4F {
        Vector4F {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl<'a, 'b> Add<&'b Vector4F> for &'a Vector4F {
    type Output = Vector4F;

    fn add(self, other: &'b Vector4F) -> Vector4F {
        Vector4F {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl Sub for Vector4F {
    type Output = Vector4F;

    fn sub(self, other: Vector4F) -> Vector4F {
        Vector4F {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

impl<'a, 'b> Sub<&'b Vector4F> for &'a Vector4F {
    type Output = Vector4F;

    fn sub(self, other: &'b Vector4F) -> Vector4F {
        Vector4F {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

impl Mul for Vector4F {
    type Output = Vector4F;

    fn mul(self, other: Vector4F) -> Vector4F {
        Vector4F {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
            w: self.w * other.w,
        }
    }
}

impl<'a, 'b> Mul<&'b Vector4F> for &'a Vector4F {
    type Output = Vector4F;

    fn mul(self, other: &'b Vector4F) -> Vector4F {
        Vector4F {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
            w: self.w * other.w,
        }
    }
}

impl PartialEq for Vector4F {
    fn eq(&self, other: &Vector4F) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z && self.w == other.w
    }
}

//############################# VERTEX #############################

pub struct Vertex4F {
    pub pos: Vector4F,
    pub normal: Vector4F,
    pub tex_u: f64,
    pub tex_v: f64,
    pub color: Color,
}

impl Vertex4F {
    pub fn new() -> Vertex4F {
        Vertex4F {
            pos: Vector4F::null(),
            normal: Vector4F::null(),
            tex_u: 0.0,
            tex_v: 0.0,
            color: Color::black(),
        }
    }

    pub fn clone(&self) -> Vertex4F {
        Vertex4F {
            pos: self.pos.clone(),
            normal: self.normal.clone(),
            tex_u: self.tex_u,
            tex_v: self.tex_v,
            color: self.color.clone(),
        }
    }
}

//############################# INTERSECTIONS #############################

pub struct Intersection {
    pub pos: Vector4F,
    pub normal: Vector4F,
    pub tex_u: f64,
    pub tex_v: f64,
    pub barycentric: Vector4F,
    pub ray_t: f64,
}

// Intersects ray with sphere.
//
// p0: ray origin
// d: ray direction, scaled by ray length
// c: sphere center
// r: sphere radius
// mint_t: minimum T value of ray. If intersection is bigger than this None is returned
pub fn intersect_ray_sphere(
    p0: &Vector4F,
    d: &Vector4F,
    c: &Vector4F,
    r: f64,
    min_t: f64,
) -> Option<Intersection> {
    let dnorm = d.normalize();

    let e = c - p0;

    let le = e.len();
    let a = Vector4F::dot(&e, &dnorm);
    let k = r * r - le * le + a * a;

    //No intersection
    if k < 0.0 {
        return None;
    }

    let f = k.sqrt();

    //No intersection
    if f < 0.0 {
        return None;
    }

    let t = a - f;

    if t <= 0.0 || t > min_t {
        return None;
    }

    let point = Vector4F {
        x: p0.x + dnorm.x * t,
        y: p0.y + dnorm.y * t,
        z: p0.z + dnorm.z * t,
        w: 1.0,
    };

    let normal = (&point - c).normalize();

    let result = Intersection {
        pos: point,
        normal: normal,
        tex_u: 0.0,
        tex_v: 0.0,
        barycentric: Vector4F {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        ray_t: t,
    };

    Some(result)
}

// Intersects ray with triangle.
//
// p0: ray origin
// d: ray direction, scaled by ray length
// v1: first vertex of triangle
// v2: second vertex of triangle
// v3: third vertex of triangle
// mint_t: minimum T value of ray. If intersection is bigger than this None is returned
pub fn intersect_ray_triangle(
    rorg: &Vector4F,
    rdir: &Vector4F,
    t0: &Vertex4F,
    t1: &Vertex4F,
    t2: &Vertex4F,
    min_t: f64,
) -> Option<Intersection> {
    let p0 = &t0.pos;
    let p1 = &t1.pos;
    let p2 = &t2.pos;

    let e1 = p1 - p0;
    let e2 = p2 - p1;
    let n = Vector4F::cross(&e1, &e2);
    let dot = Vector4F::dot(&n, rdir);

    if !(dot < 0.0) {
        return None;
    }

    let d = Vector4F::dot(&n, &p0);
    let mut t = d - Vector4F::dot(&n, rorg);

    if !(t <= 0.0) {
        return None;
    }

    if !(t >= dot * min_t) {
        return None;
    }

    t = t / dot;

    assert!(t >= 0.0);
    //assert!(t <= min_t);

    let p = Vector4F {
        x: rorg.x + (rdir.x * t),
        y: rorg.y + (rdir.y * t),
        z: rorg.z + (rdir.z * t),
        w: 1.0,
    };

    let u0;
    let u1;
    let u2;

    let v0;
    let v1;
    let v2;

    let absx = n.x.abs();
    let absy = n.y.abs();
    let absz = n.z.abs();

    if absx > absy {
        if absx > absz {
            u0 = p.y - p0.y;
            u1 = p1.y - p0.y;
            u2 = p2.y - p0.y;

            v0 = p.z - p0.z;
            v1 = p1.z - p0.z;
            v2 = p2.z - p0.z;
        } else {
            u0 = p.x - p0.x;
            u1 = p1.x - p0.x;
            u2 = p2.x - p0.x;

            v0 = p.y - p0.y;
            v1 = p1.y - p0.y;
            v2 = p2.y - p0.y;
        }
    } else {
        if absy > absz {
            u0 = p.x - p0.x;
            u1 = p1.x - p0.x;
            u2 = p2.x - p0.x;

            v0 = p.z - p0.z;
            v1 = p1.z - p0.z;
            v2 = p2.z - p0.z;
        } else {
            u0 = p.x - p0.x;
            u1 = p1.x - p0.x;
            u2 = p2.x - p0.x;

            v0 = p.y - p0.y;
            v1 = p1.y - p0.y;
            v2 = p2.y - p0.y;
        }
    }

    let mut temp = u1 * v2 - v1 * u2;

    if !(temp != 0.0) {
        return None;
    }

    temp = 1.0 / temp;

    let alpha = (u0 * v2 - v0 * u2) * temp;
    if !(alpha >= 0.0) {
        return None;
    }

    let beta = (u1 * v0 - v1 * u0) * temp;
    if !(beta >= 0.0) {
        return None;
    }

    let gamma = 1.0 - alpha - beta;
    if !(gamma >= 0.0) {
        return None;
    }

    let n0 = &t0.normal;
    let n1 = &t0.normal;
    let n2 = &t0.normal;

    let normal = Vector4F {
        x: n0.x * alpha + n1.x * beta + n2.x * gamma,
        y: n0.y * alpha + n1.y * beta + n2.y * gamma,
        z: n0.z * alpha + n1.z * beta + n2.z * gamma,
        w: 1.0,
    };

    let result = Intersection {
        pos: p,
        normal: normal.normalize(),
        tex_u: 0.0,
        tex_v: 0.0,
        barycentric: Vector4F::new(alpha, beta, gamma),
        ray_t: t,
    };

    Some(result)
}

pub fn ray_intersects_aabb(
    rorg: &Vector4F,
    rdir: &Vector4F,
    min: &Vector4F,
    max: &Vector4F,
) -> bool {
    //Source: https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-box-intersection
    let mut tmin = (min.x - rorg.x) / rdir.x;
    let mut tmax = (max.x - rorg.x) / rdir.x;

    if tmin > tmax {
        let tmp = tmin;
        tmin = tmax;
        tmax = tmp;
    }

    let mut tymin = (min.y - rorg.y) / rdir.y;
    let mut tymax = (max.y - rorg.y) / rdir.y;

    if tymin > tymax {
        let tmp = tymin;
        tymin = tymax;
        tymax = tmp;
    }

    if tmin > tymax || tymin > tmax {
        return false;
    }

    if tymin > tmin {
        tmin = tymin;
    }

    if tymax < tmax {
        tmax = tymax;
    }

    let mut tzmin = (min.z - rorg.z) / rdir.z;
    let mut tzmax = (max.z - rorg.z) / rdir.z;

    if tzmin > tzmax {
        let tmp = tzmin;
        tzmin = tzmax;
        tzmax = tmp;
    }

    if tmin > tzmax || tzmin > tmax {
        return false;
    }
    /*
    if tzmin > tmin {
      tmin = tzmin;
    }

    if tzmax < tmax {
      tmax = tzmax;
    }
    */
    true
}

pub fn intersect_ray_aabb2(
    rorg: &Vector4F,
    rdir: &Vector4F,
    min: &Vector4F,
    max: &Vector4F,
) -> Option<Intersection> {
    let mut inside = true;

    let mut xt;
    let mut xn = 0.0;
    if rorg.x < min.x {
        xt = min.x - rorg.x;
        /*if xt > rdir.x {
            return None;
        }*/
        xt /= rdir.x;
        inside = false;
        xn = -1.0;
    }
    else if rorg.x > max.x {
        xt = max.x - rorg.x;
        /*if xt < rdir.x {
            return None;
        }*/
        xt /= rdir.x;
        inside = false;
        xn = 1.0;
    }
    else {
        xt = -1.0;
    }

    let mut yt;
    let mut yn = 0.0;
    if rorg.y < min.y {
        yt = min.y - rorg.y;
        /*if yt > rdir.y {
            return None;
        }*/
        yt /= rdir.y;
        inside = false;
        yn = -1.0;
    }
    else if rorg.y > max.y {
        yt = max.y - rorg.y;
        /*if yt < rdir.y {
            return None;
        }*/
        yt /= rdir.y;
        inside = false;
        yn = 1.0;
    }
    else {
        yt = -1.0;
    }

    let mut zt;
    let mut zn = 0.0;
    if rorg.z < min.z {
        zt = min.z - rorg.z;
        /*if zt > rdir.z {
            return None;
        }*/
        zt /= rdir.z;
        inside = false;
        zn = -1.0;
    }
    else if rorg.z > max.z {
        zt = max.z - rorg.z;
        /*if zt < rdir.z {
            return None;
        }*/
        zt /= rdir.z;
        inside = false;
        zn = 1.0;
    }
    else {
        zt = -1.0;
    }

    if inside {
        return Some(Intersection {
            pos: rorg.clone(),
            normal: rdir.invert(),
            tex_u: 0.0,
            tex_v: 0.0,
            barycentric: Vector4F::null(),
            ray_t: 0.0,
        });
    }

    let mut which = 0;
    let mut t = xt;
    if yt > t {
        which = 1;
        t = yt;
    }
    if zt > t {
        which = 2;
        t = zt;
    }

    if t < 0.0 {
        return None;
    }

    let mut normal = Vector4F::null();
    match which {
        0 => {
            let y = rorg.y + (rdir.y * t);
            if y < min.y || y > max.y {
                return None;
            }
            let z = rorg.z + (rdir.z * t);
            if z < min.z || z > max.z {
                return None;
            }

            normal.x = xn;
            normal.y = 0.0;
            normal.z = 0.0;
        },
        1 => {
            let x = rorg.x + (rdir.x * t);
            if x < min.x || x > max.x {
                return None;
            }
            let z = rorg.z + (rdir.z * t);
            if z < min.z || z > max.z {
                return None;
            }

            normal.x = 0.0;
            normal.y = yn;
            normal.z = 0.0;
        },
        2 => {
            let x = rorg.x + (rdir.x * t);
            if x < min.x || x > max.x {
                return None;
            }
            let y = rorg.y + (rdir.y * t);
            if y < min.y || y > max.y {
                return None;
            }

            normal.x = 0.0;
            normal.y = 0.0;
            normal.z = zn;
        },
        _ => {
            panic!("Unexpected value!");
        }
    }

    //The 0.999... will offset the intersection point to be a bit outside the AABB, towards the ray origin. Otherwise, the AABB will shadow itself.
    let p = point_on_ray(&rorg, &rdir, t * 0.999999999999);

    let result = Intersection {
        pos: p,
        normal: normal,
        tex_u: 0.0,
        tex_v: 0.0,
        barycentric: Vector4F::null(),
        ray_t: t,
    };

    Some(result)
}

pub fn point_on_ray(rorg: &Vector4F, rdir: &Vector4F, t: f64) -> Vector4F {
    Vector4F::new(
        rorg.x + (rdir.x * t), 
        rorg.y + (rdir.y * t),
        rorg.z + (rdir.z * t)
    )
}

pub fn intersect_ray_aabb(
    rorg: &Vector4F,
    rdir: &Vector4F,
    min: &Vector4F,
    max: &Vector4F,
) -> Option<Intersection> {
    let ndir = Vector4F {
        x: 1.0 / rdir.x,
        y: 1.0 / rdir.y,
        z: 1.0 / rdir.z,
        w: 1.0,
    };

    let t1 = (min.x - rorg.x) * ndir.x;
    let t2 = (max.x - rorg.x) * ndir.x;

    let mut tmin = f64::min(t1, t2);
    let mut tmax = f64::max(t1, t2);

    let t1 = (min.y - rorg.y) * ndir.y;
    let t2 = (max.y - rorg.y) * ndir.y;

    tmin = f64::max(tmin, f64::min(f64::min(t1, t2), tmax));
    tmax = f64::min(tmax, f64::max(f64::max(t1, t2), tmin));

    let t1 = (min.z - rorg.z) * ndir.z;
    let t2 = (max.z - rorg.z) * ndir.z;

    tmin = f64::max(tmin, f64::min(f64::min(t1, t2), tmax));
    tmax = f64::min(tmax, f64::max(f64::max(t1, t2), tmin));

    if tmax < f64::max(tmin, 0.0) {
        return None;
    }

    let box_center = Vector4F {
        x: (min.x + max.x) * 0.5,
        y: (min.y + max.y) * 0.5,
        z: (min.z + max.z) * 0.5,
        w: 1.0,
    };

    let p = Vector4F {
        x: rorg.x + (rdir.x * tmin),
        y: rorg.y + (rdir.y * tmin),
        z: rorg.z + (rdir.z * tmin),
        w: 1.0,
    };

    let mut n = &p - &box_center;

    if n.x > n.y && n.x > n.z {
        n.x = 1.0;
        n.y = 0.0;
        n.z = 0.0;
    } else if n.y > n.x && n.y > n.z {
        n.x = 0.0;
        n.y = 1.0;
        n.z = 0.0;
    } else {
        n.x = 0.0;
        n.y = 0.0;
        n.z = 1.0;
    }

    let result = Intersection {
        pos: p,
        normal: n,
        tex_u: 0.0,
        tex_v: 0.0,
        barycentric: Vector4F::null(),
        ray_t: tmin,
    };

    Some(result)
}

pub fn triangle_aabb_overlap(
    t1: &Vector4F,
    t2: &Vector4F,
    t3: &Vector4F,
    min: &Vector4F,
    max: &Vector4F,
) -> bool {
    let (tmin, tmax) = triangle_to_aabb(t1, t2, t3);
    aabb_aabb_overlap(&tmin, &tmax, min, max)
}

pub fn triangle_to_aabb(t1: &Vector4F, t2: &Vector4F, t3: &Vector4F) -> (Vector4F, Vector4F) {
    let min = Vector4F {
        x: f64::min(f64::min(t1.x, t2.x), t3.x),
        y: f64::min(f64::min(t1.y, t2.y), t3.y),
        z: f64::min(f64::min(t1.z, t2.z), t3.z),
        w: 1.0,
    };

    let max = Vector4F {
        x: f64::max(f64::max(t1.x, t2.x), t3.x),
        y: f64::max(f64::max(t1.y, t2.y), t3.y),
        z: f64::max(f64::max(t1.z, t2.z), t3.z),
        w: 1.0,
    };

    (min, max)
}

pub fn aabb_aabb_overlap(
    min1: &Vector4F,
    max1: &Vector4F,
    min2: &Vector4F,
    max2: &Vector4F,
) -> bool {
    if min1.x > max2.x {
        return false;
    };
    if max1.x < min2.x {
        return false;
    };

    if min1.y > max2.y {
        return false;
    };
    if max1.y < min2.y {
        return false;
    };

    if min1.z > max2.z {
        return false;
    };
    if max1.z < min2.z {
        return false;
    };

    true
}

pub fn point_in_aabb(p: &Vector4F, min: &Vector4F, max: &Vector4F) -> bool {
    if p.x < min.x {
        return false;
    };
    if p.x > max.x {
        return false;
    };

    if p.y < min.y {
        return false;
    };
    if p.y > max.y {
        return false;
    };

    if p.z < min.z {
        return false;
    };
    if p.z > max.z {
        return false;
    };

    true
}
