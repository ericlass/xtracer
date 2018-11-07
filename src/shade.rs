use linear::Vector4F;

pub fn shade_lambert(l: &Vector4F, n: &Vector4F) -> f64 {
  f64::max(0.0, Vector4F::dot(&n.normalize(), &l.normalize()))
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

pub fn shade_oren_nayar(l: &Vector4F, n: &Vector4F, v: &Vector4F, rough: f64, alb: f64) -> f64 {
  let nn = n.normalize();
  let ln = l.normalize();
  let vn = v.normalize();

  let r2 = rough * rough;
  let onf_x = r2 / (r2 + 0.33);
  let onf_y = r2 / (r2 + 0.09);
  let on_x = 1.0 + -0.5 * onf_x;
  let on_y = 0.0 + 0.45 * onf_y;

  let ndotl = Vector4F::dot(&nn, &ln);
  let ndotv = Vector4F::dot(&nn, &vn);
  let ct_x = saturate(ndotl);
  let ct_y = saturate(ndotv);
  let ct_x2 = ct_x * ct_x;
  let ct_y2 = ct_y * ct_y;
  let st = ((1.0 - ct_x2) * (1.0 - ct_y2)).sqrt();

  let lp = Vector4F {
    x: ln.x - ct_x * nn.x,
    y: ln.y - ct_x * nn.y,
    z: ln.z - ct_x * nn.z,
    w: 1.0
  };

  let vp = Vector4F {
    x: vn.x - ct_y * nn.x,
    y: vn.y - ct_y * nn.y,
    z: vn.z - ct_y * nn.z,
    w: 1.0
  };

  let cp = saturate(Vector4F::dot(&lp, &vp));

  let don = cp * st / f64::max(ct_x, ct_y);
  let dif = ct_x * (on_x + on_y * don);

  dif
}