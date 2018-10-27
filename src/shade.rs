use linear::Vector4F;

pub fn shade_lambert(l: &Vector4F, n: &Vector4F) -> f64 {
  f64::max(0.0, Vector4F::dot(&n.normalize(), &l.normalize()))
}