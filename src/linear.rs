use std::cmp::PartialEq;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::ops::Add;
use std::ops::Sub;

#[repr(C, packed)]
pub struct Vector4F {
  pub x: f64,
  pub y: f64,
  pub z: f64,
  pub w: f64
}

impl Vector4F {
  pub fn null() -> Vector4F {
    Vector4F{
      x: 0.0,
      y: 0.0,
      z: 0.0,
      w: 0.0,
    }
  }

  pub fn add(v1: &Vector4F, v2: &Vector4F) -> Vector4F {
    Vector4F{
      x: v1.x + v2.x,
      y: v1.y + v2.y,
      z: v1.z + v2.z,
      w: v1.w + v2.w,
    }
  }

  pub fn sub(v1: &Vector4F, v2: &Vector4F) -> Vector4F {
    Vector4F{
      x: v1.x - v2.x,
      y: v1.y - v2.y,
      z: v1.z - v2.z,
      w: v1.w - v2.w,
    }
  }

  pub fn dot(v1: &Vector4F, v2: &Vector4F) -> f64 {
    v1.x * v2.x +
    v1.y * v2.y +
    v1.z * v2.z
  }

  pub fn cross(v1: &Vector4F, v2: &Vector4F) -> Vector4F {
    Vector4F {
      x: v1.y * v2.z - v2.y * v1.z,
      y: v1.z * v2.x - v2.z * v1.x,
      z: v1.x * v2.y - v2.x * v1.y,
      w: 1.0
    }
  }

  pub fn project_scalar(v1: &Vector4F, v2: &Vector4F) -> f64 {
    Vector4F::dot(v1, v2) / v2.sqr_len()
  }

  pub fn project(v1: &Vector4F, v2: &Vector4F) -> Vector4F {
    let ps = Vector4F::project_scalar(v1, v2);

    Vector4F{
      x: v2.x * ps,
      y: v2.y * ps,
      z: v2.z * ps,
      w: v2.w,
    }
  }

  pub fn reflect(i: &Vector4F, n: &Vector4F) -> Vector4F {
    let dot = Vector4F::dot(n, i);

    Vector4F{
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
      Vector4F { x: 0.0, y: 0.0, z: 0.0, w: 0.0 }
    }
    else {
      let cost2sqrt = cost2.sqrt();
      Vector4F {
        x: eta * i.x - ((eta * cosi + cost2sqrt) * n.x),
        y: eta * i.y - ((eta * cosi + cost2sqrt) * n.y),
        z: eta * i.z - ((eta * cosi + cost2sqrt) * n.z),
        w: 1.0
      }
    }
  }

  pub fn normalize(&self) -> Vector4F {
    let len = self.len();

    Vector4F{
      x: self.x / len,
      y: self.y / len,
      z: self.z / len,
      w: self.w
    }
  }

  pub fn len(&self) -> f64 {
    self.sqr_len().sqrt()
  }

  pub fn sqr_len(&self) -> f64 {
    (self.x * self.x + self.y * self.y + self.z * self.z)
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
    Vector4F{
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
    Vector4F{
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
    Vector4F{
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
    Vector4F{
      x: self.x - other.x,
      y: self.y - other.y,
      z: self.z - other.z,
      w: self.w - other.w,
    }
  }
}

impl PartialEq for Vector4F {
  fn eq(&self, other: &Vector4F) -> bool {
    self.x == other.x &&
    self.y == other.y &&
    self.z == other.z &&
    self.w == other.w
  }
}

//############################# VERTEX #############################

#[repr(C, packed)]
pub struct Vertex4F {
  pub pos: Vector4F,
  pub normal: Vector4F,
  pub tex: Vector4F,
  pub color: Vector4F
}

//############################# INTERSECTIONS #############################

struct PluckerCoords {
  p0: f64,
  p1: f64,
  p2: f64,
  p3: f64,
  p4: f64,
  p5: f64
}

pub struct Intersection {
  pub pos: Vector4F,
  pub normal: Vector4F,
  pub tex: Vector4F,
  pub barycentric: Vector4F,
  pub ray_t: f64
}

fn plucker(start: &Vector4F, end: &Vector4F) -> PluckerCoords {
  PluckerCoords {
    p0: start.x * end.y - end.x * start.y,
    p1: start.x * end.z - end.x * start.z,
    p2: start.x - end.x,
    p3: start.y * end.z - end.y * start.z,
    p4: start.z - end.z,
    p5: end.y - start.y
  }
}

fn side(a: &PluckerCoords, b: &PluckerCoords) -> f64 {
  a.p0 * b.p4 + a.p1 * b.p5 + a.p2 * b.p3 + a.p3 * b.p2 + a.p4 * b.p0 + a.p5 * b.p1
}

struct SideProducts {
  s1: f64,
  s2: f64,
  s3: f64
}

// Calculates side products of ray and triangle.
//
// rorg: ray origin
// rdir: ray direction, scaled by ray length
// t1: first point of triangle
// t2: second point of triangle
// t3: third point of triangle
fn ray_triangle_side_products(rorg: &Vector4F, rdir: &Vector4F, t1: &Vertex4F, t2: &Vertex4F, t3: &Vertex4F) -> SideProducts {
  let ta = &t1.pos;
  let tb = &t2.pos;
  let tc = &t3.pos;

  let tab = &plucker(ta, tb);

  //Calculate barycentric coordinates
  let raypluck = &plucker(rorg, rdir);
  let tbc = &plucker(tb, tc);
  let tca = &plucker(tc, ta);

  let s1 = side(raypluck, tab);
  let s2 = side(raypluck, tbc);
  let s3 = side(raypluck, tca);

  SideProducts {
    s1,
    s2,
    s3
  }
}

const EPS: f64 = 0.0000001;
const NEPS: f64 = -0.0000001;

// Checks if ray intersects with triangle using plucker coordinates. Does not provide additional information about the intersection, onyl if it
// intersects or not. If you required more information, like intersection point, normal... use "intersect_ray_triangle" instead.
//
// rorg: ray origin
// rdir: ray direction, scaled by ray length
// t1: first point of triangle
// t2: second point of triangle
// t3: third point of triangle
pub fn ray_intersects_triangle(rorg: &Vector4F, rdir: &Vector4F, t1: &Vertex4F, t2: &Vertex4F, t3: &Vertex4F) -> bool {
  let sides = ray_triangle_side_products(rorg, rdir, t1, t2, t3);

  let s1 = sides.s1;
  let s2 = sides.s2;
  let s3 = sides.s3;

  (s1 > NEPS && s2 > NEPS && s3 > NEPS) || (s1 < EPS && s2 < EPS && s3 < EPS)
}

// Intersects ray with triangle using plucker coordinates and returns all kinds of intersection information, which takes some time to compute.
// Use ray_intersects_triangle if you only need to to know if the ray intersects the triangle or not.
//
// rorg: ray origin
// rdir: ray direction, scaled by ray length
// t1: first point of triangle
// t2: second point of triangle
// t3: third point of triangle
// mint_t: minimum T value of ray. If intersection is bigger than this None is returned
pub fn intersect_ray_triangle(rorg: &Vector4F, rdir: &Vector4F, t1: &Vertex4F, t2: &Vertex4F, t3: &Vertex4F, min_t: f64) -> Option<Intersection> {
  let ta = &t1.pos;
  let tb = &t2.pos;
  let tc = &t3.pos;

  let sides = ray_triangle_side_products(rorg, rdir, t1, t2, t3);

  let mut s1 = sides.s1;
  let mut s2 = sides.s2;
  let mut s3 = sides.s3;

  if (s1 > NEPS && s2 > NEPS && s3 > NEPS) || (s1 < EPS && s2 < EPS && s3 < EPS) {
    //Side products are proportional to the signed area
    //of the barycentric triangles. So scale them to sum
    //up to 1 to get barycentric coordinates.
    let sum = s1 + s2 + s3;
    s1 /= sum; //Barycentric C
    s2 /= sum; //Barycentric A
    s3 /= sum; //Barycentric B

    // Calculate using barycentric coordinated because t value is not correct. This is even faster!
    let point = Vector4F {
      x: ta.x * s2 + tb.x * s3 + tc.x * s1,
      y: ta.y * s2 + tb.y * s3 + tc.y * s1,
      z: ta.z * s2 + tb.z * s3 + tc.z * s1,
      w: 1.0
    };

    let real_t = (&point - rorg).sqr_len() / rdir.sqr_len();

    if real_t > min_t {
      return None;
    }

    //Interpolate normal
    let na = &t1.normal;
    let nb = &t2.normal;
    let nc = &t3.normal;

    let normal = Vector4F {
      x: na.x * s2 + nb.x * s3 + nc.x * s1,
      y: na.y * s2 + nb.y * s3 + nc.y * s1,
      z: na.z * s2 + nb.z * s3 + nc.z * s1,
      w: 1.0
    };

    //Interpolate texture coordinates
    let ta = &t1.tex;
    let tb = &t2.tex;
    let tc = &t3.tex;

    let tex = Vector4F {
      x: ta.x * s2 + tb.x * s3 + tc.x * s1,
      y: ta.y * s2 + tb.y * s3 + tc.y * s1,
      z: ta.z * s2 + tb.z * s3 + tc.z * s1,
      w: 1.0
    };

    //Fill other intersection info
    let result = Intersection {
      pos: point,
      normal: normal,
      tex: tex,
      barycentric: Vector4F {x: s2, y: s3, z: s1, w: 1.0},
      ray_t: real_t
    };

    return Some(result);
  }

  return None;
}

pub fn ray_intersects_sphere(p0: &Vector4F, d: &Vector4F, c: &Vector4F, r: f64) -> bool {
  let dnorm = d.normalize();

  let e = c - p0;
  let le = e.len();
  let a = Vector4F::dot(&e, &dnorm);
  let f = (r * r - le * le + a * a).sqrt();

  if f >= 0.0 {
    let t = a - f;
    if t >= 0.0 {
      return true;
    }
  }

  false
}

// Intersects ray with sphere.
//
// p0: ray origin
// d: ray direction, scaled by ray length
// c: sphere center
// r: sphere radius
// mint_t: minimum T value of ray. If intersection is bigger than this None is returned
pub fn intersect_ray_sphere(p0: &Vector4F, d: &Vector4F, c: &Vector4F, r: f64, min_t: f64) -> Option<Intersection> {
  let dnorm = d.normalize();

  let e = c - p0;
  let le = e.len();
  let a = Vector4F::dot(&e, &dnorm);
  let f = (r * r - le * le + a * a).sqrt();

  //No intersection
  if f < 0.0 {
    return None;
  }

  let t = a - f;

  if t < 0.0 || t > min_t {
    return None;
  }

  let point = Vector4F {
    x: p0.x + dnorm.x * t,
    y: p0.y + dnorm.y * t,
    z: p0.z + dnorm.z * t,
    w: 1.0
  };

  let normal = (&point - c).normalize();

  let result = Intersection {
    pos: point,
    normal: normal,
    tex: Vector4F { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
    barycentric: Vector4F { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
    ray_t: t
  };

  Some(result)
}