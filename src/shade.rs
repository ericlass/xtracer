use linear::Vector4F;

pub fn shade_lambert(l: &Vector4F, n: &Vector4F) -> f64 {
  let ln = l.normalize();
  let nn = n.normalize();

  Vector4F::dot(&nn, &ln)
}