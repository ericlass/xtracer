use linear;
use linear::Vector4F;
use settings::Triangle;

pub struct OctreeNode {
  pub children: Vec<OctreeNode>,
  pub tris: Vec<usize>,
  pub min: Vector4F,
  pub max: Vector4F,
}

impl OctreeNode {
  pub fn new() -> OctreeNode {
    OctreeNode {
      children: Vec::new(),
      tris: Vec::new(),
      min: Vector4F::null(),
      max: Vector4F::null(),
    }
  }

  pub fn intersection_candidates(&self, rorg: &Vector4F, rdir: &Vector4F) -> Vec<usize> {
    let mut result = Vec::new();
    self.intersection_candidates_rec(rorg, rdir, &mut result);
    result
  }

  fn intersection_candidates_rec(&self, rorg: &Vector4F, rdir: &Vector4F, candidates: &mut Vec<usize>) {
    if linear::ray_intersects_aabb(rorg, rdir, &self.min, &self.max) {
      if self.children.len() > 0 {
        for child in &self.children {
          child.intersection_candidates_rec(rorg, rdir, candidates);
        }
      }
      else {
        for tri in &self.tris {
          candidates.push(*tri);
        }
      }
    }
  }
}

fn qmin(v1: f64, v2: f64, v3: f64, v4: f64) -> f64 {
  f64::min(f64::min(f64::min(v1, v2), v3), v4)
}

fn qmax(v1: f64, v2: f64, v3: f64, v4: f64) -> f64 {
  f64::max(f64::max(f64::max(v1, v2), v3), v4)
}

pub fn build_octree(triangles: &Vec<Triangle>) -> OctreeNode {
  let mut result = OctreeNode::new();

  let mut min = Vector4F {
    x: std::f64::MAX,
    y: std::f64::MAX,
    z: std::f64::MAX,
    w: 1.0
  };

  let mut max = Vector4F {
    x: std::f64::MIN,
    y: std::f64::MIN,
    z: std::f64::MIN,
    w: 1.0
  };

  for tri in triangles {
    min.x = qmin(min.x, tri.v1.pos.x, tri.v2.pos.x, tri.v3.pos.x);
    min.y = qmin(min.y, tri.v1.pos.y, tri.v2.pos.y, tri.v3.pos.y);
    min.z = qmin(min.z, tri.v1.pos.z, tri.v2.pos.z, tri.v3.pos.z);

    max.x = qmax(max.x, tri.v1.pos.x, tri.v2.pos.x, tri.v3.pos.x);
    max.y = qmax(max.y, tri.v1.pos.y, tri.v2.pos.y, tri.v3.pos.y);
    max.z = qmax(max.z, tri.v1.pos.z, tri.v2.pos.z, tri.v3.pos.z);
  }

  result.min = min;
  result.max = max;
  build_octree_rec(&mut result, triangles, 1, 4);

  result
}

fn build_octree_rec(node: &mut OctreeNode, triangles: &Vec<Triangle>, depth: u32, max_depth: u32) {
  let min = &node.min;
  let max = &node.max;

  //Save intersecting tris only for leave nodes
  if depth >= max_depth {
    let mut tris = Vec::new();
    for t in 0..triangles.len() {
      let tri = &triangles[t];
      if linear::triangle_aabb_overlap(&tri.v1.pos, &tri.v2.pos, &tri.v3.pos, min, max) {
        tris.push(t);
      }
    }
    node.tris = tris;

    //Maximum level reached, stop recursion
    return;
  }

  let half_x = (max.x - min.x) / 2.0;
  let half_y = (max.y - min.y) / 2.0;
  let half_z = (max.z - min.z) / 2.0;

  let mut x = min.x;
  for _x in 0..2 {
    let mut y = min.y;
    for _y in 0..2 {
      let mut z = min.z;
      for _z in 0..2 {
        let nmin = Vector4F::new(x, y, z);
        let nmax = Vector4F::new(x + half_x, y + half_y, z + half_z);
        let mut nnode = OctreeNode::new();
        nnode.min = nmin;
        nnode.max = nmax;
        
        build_octree_rec(&mut nnode, triangles, depth + 1, max_depth);
        node.children.push(nnode);

        z += half_z;
      }
      y += half_y;
    }
    x += half_x;
  }
}
