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

// Intersects ray with triangle using plucker coordinates.
//
// rorg: ray origin
// rdir: ray direction, scaled by ray length
// t1: first point of triangle
// t1: second point of triangle
// t1: third point of triangle
// mint_t: minimum T value of ray. If intersection is bigger than this None is returned
pub fn intersect_ray_triangle(rorg: &Vector4F, rdir: &Vector4F, t1: &Vertex4F, t2: &Vertex4F, t3: &Vertex4F, min_t: f64) -> Option<Intersection> {
  //Used to test values for beeing close to zero because of limited precision
  let eps = 0.0000001;
  let neps = -0.0000001;

  let ta = &t1.pos;
  let tb = &t2.pos;
  let tc = &t3.pos;

  let ray_end = &(rorg + rdir);

  let tab = &plucker(ta, tb);
  let tcro = &plucker(tc, rorg);
  let tcre = &plucker(tc, ray_end);

  //Calculate t of intersection of ray with triangle plane
  //WARNING: Somehow this is not 100% correct. Don't use for further calculation
  let g1 = side(tab, tcro);
  let g2 = side(tab, tcre);
  let t = g1 / (-g2);

  // If there already was an intersection closer to the camera then return null
  if t < eps || t > min_t {
    return None;
  }

  //Calculate barycentric coordinates
  let raypluck = &plucker(rorg, rdir);
  let tbc = &plucker(tb, tc);
  let tca = &plucker(tc, ta);

  let mut s1 = side(raypluck, tab);
  let mut s2 = side(raypluck, tbc);
  let mut s3 = side(raypluck, tca);

  if (s1 > neps && s2 > neps && s3 > neps) || (s1 < eps && s2 < eps && s3 < eps) {
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

    let real_t = (&point - rorg).sqr_len() / rdir.sqr_len();

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

pub fn intersect_ray_sphere(rorg: &Vector4F, rdir: &Vector4F, sc: &Vector4F, sr: f64, min_t: f64) -> Option<Intersection> {
  let a = rdir.sqr_len();

  let relx = rorg.x - sc.x;
  let rely = rorg.y - sc.y;
  let relz = rorg.z - sc.z;

  let b = 2.0 * (rdir.x * relx + rdir.y * rely + rdir.z * relz);
  let c = relx * relx + rely * rely + relz * relz - sr * sr;

  let discriminant = (b * b) - (4.0 * a * c);
  if discriminant < 0.0 {
    return None;
  }

  let m = b * b - 4.0 * c;
  let t = (-b - m.sqrt()) / 2.0;
  if t < 0.0 || t > min_t {
    return None;
  }

  let point = Vector4F {
    x: rorg.x + rdir.x * t,
    y: rorg.y + rdir.y * t,
    z: rorg.z + rdir.z * t,
    w: 1.0
  };

  let normal = (sc - &point).normalize();

  let result = Intersection {
    pos: point,
    normal: normal,
    tex: Vector4F { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
    barycentric: Vector4F { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
    ray_t: t
  };

  return Some(result);
}