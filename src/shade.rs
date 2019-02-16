use linear::Vector4F;

const PI: f64 = 3.1415926535897932384626433;
const E: f64 = 2.718281828459045;

pub fn shade_lambert(l: &Vector4F, n: &Vector4F) -> f64 {
  f64::max(0.0, Vector4F::dot(&n, &l))
}

fn saturate(v: f64) -> f64 {
  let mut result = v;
  if result < 0.0 {
    result = 0.0;
  }
  if result > 1.0 {
    result = 1.0;
  }  

  result
}

//Taken from: http://shaderjvo.blogspot.com/2011/08/van-ouwerkerks-rewrite-of-oren-nayar.html
pub fn shade_oren_nayar(l: &Vector4F, n: &Vector4F, v: &Vector4F, rough: f64, alb: f64) -> f64 {
  let r2 = rough * rough;
  let onf_x = r2 / (r2 + 0.33);
  let onf_y = r2 / (r2 + 0.09);
  let on_x = 1.0 + -0.5 * onf_x;
  let on_y = 0.0 + 0.45 * onf_y;

  let ndotl = Vector4F::dot(n, l);
  let ndotv = Vector4F::dot(n, v);
  let ct_x = saturate(ndotl);
  let ct_y = saturate(ndotv);
  let ct_x2 = ct_x * ct_x;
  let ct_y2 = ct_y * ct_y;
  let st = ((1.0 - ct_x2) * (1.0 - ct_y2)).sqrt();

  let lp = Vector4F {
    x: l.x - ct_x * n.x,
    y: l.y - ct_x * n.y,
    z: l.z - ct_x * n.z,
    w: 1.0
  };

  let vp = Vector4F {
    x: v.x - ct_y * n.x,
    y: v.y - ct_y * n.y,
    z: v.z - ct_y * n.z,
    w: 1.0
  };

  let cp = saturate(Vector4F::dot(&lp, &vp));

  //Avoid div by 0
  if ct_x == 0.0 || ct_y == 0.0 {
    return 0.0;
  }

  let don = cp * st / f64::max(ct_x, ct_y);
  let dif = ct_x * (on_x + on_y * don);

  dif
}


fn beckmann(x: f64, rough: f64) -> f64 {
  let ndoth = f64::max(x, 0.0001);
  let cos2alpha = ndoth * ndoth;
  let tan2alpha = (cos2alpha - 1.0) / cos2alpha;
  let rough2 = rough * rough;
  let denom = PI * rough2 * cos2alpha * cos2alpha;

  E.powf(tan2alpha / rough2) / denom
}

pub fn shade_cook_torrance(l: &Vector4F, v: &Vector4F, n: &Vector4F, rough: f64, fresnel: f64) -> f64 {
  let vdotn = f64::max(0.0, Vector4F::dot(v, n));
  let ldotn = f64::max(0.0, Vector4F::dot(l, n));
  let h = Vector4F::half(l, v);

  let ndoth = f64::max(0.0, Vector4F::dot(n, &h));
  let vdoth = f64::max(0.000001, Vector4F::dot(v, &h));
  let x = 2.0 * ndoth / vdoth;
  let g = f64::min(1.0, f64::min(x * vdotn, x * ldotn));

  let d = beckmann(ndoth, rough);

  let f = (1.0 - vdotn).powf(fresnel);

  g * f * d / f64::max(PI * vdotn * ldotn, 0.000001)
}