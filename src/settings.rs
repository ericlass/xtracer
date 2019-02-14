use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use json::JsonValue;
use linear;
use linear::Vector4F;
use linear::Vertex4F;
use linear::Intersection;
use obj;
use octree;
use octree::OctreeNode;
use stopwatch::StopWatch;

pub struct Color {
  pub r: f64,
  pub g: f64,
  pub b: f64,
}

impl Color {
  pub fn new(r: f64, g: f64, b: f64) -> Color {
    Color {r, g, b}
  }

  pub fn black() -> Color {
    Color {r: 0.0, g: 0.0, b: 0.0}
  }

  pub fn clone(&self) -> Color {
    Color {
      r: self.r,
      g: self.g,
      b: self.b,
    }
  }
}

impl Display for Color {
  fn fmt(&self, f: &mut Formatter) -> Result {
    write!(f, "[{},{},{}]", self.r, self.g, self.b)
  }
}

pub struct Material {
  pub id: String,
  pub color: Color,
  pub reflect: f64,
  pub refract: f64,
  pub ior: f64,
  pub roughness: f64
}

pub trait Intersectable {
  fn intersect(&self, rorg: &Vector4F, rdir: &Vector4F, min_t: f64) -> Option<Intersection>;
  fn material(&self) -> String;
}

pub struct Sphere {
  pub center: Vector4F,
  pub radius: f64,
  pub material: String,
}

impl Intersectable for Sphere {
  fn intersect(&self, rorg: &Vector4F, rdir: &Vector4F, min_t: f64) -> Option<Intersection> {
    linear::intersect_ray_sphere(rorg, rdir, &self.center, self.radius, min_t)
  }

  fn material(&self) -> String {
    self.material.clone()
  }
}

pub struct Triangle {
  pub v1: Vertex4F,
  pub v2: Vertex4F,
  pub v3: Vertex4F,
}

pub struct Mesh {
  pub triangles: Vec<Triangle>,
  pub translation: Vector4F,
  pub rotation: Vector4F,
  pub scale: Vector4F,
  pub material: String,
  pub octree: OctreeNode
}

impl Intersectable for Mesh {
  fn intersect(&self, rorg: &Vector4F, rdir: &Vector4F, min_t: f64) -> Option<Intersection> {
    let candidates = self.octree.intersection_candidates(rorg, &rdir.normalize());

    let mut closest = None;
    let mut lmin_t = min_t;

    for t in candidates {
        let tri = &self.triangles[t];

        let intersection = linear::intersect_ray_triangle(
            &rorg,
            &rdir,
            &tri.v1,
            &tri.v2,
            &tri.v3,
            lmin_t
        );

        if intersection.is_some() {
            let inter = intersection.unwrap();
            if inter.ray_t < lmin_t {
                lmin_t = inter.ray_t;
                closest = Some(inter);
            }
        }
    }

    closest
  }

  fn material(&self) -> String {
    self.material.clone()
  }
}

pub enum LightType {
  Point,
  Sphere,
}

pub struct Light {
  pub ltype: LightType,
  pub position: Vector4F,
  pub color: Color,
  pub visible: bool,
  pub radius: f64,
  pub samples: u32,
  pub intensity: f64
}

pub struct Scene {
  pub materials: Vec<Material>,
  pub spheres: Vec<Sphere>,
  pub meshes: Vec<Mesh>,
  pub lights: Vec<Light>,
  pub skycolor: Color,
  pub max_depth: u32,
  pub path_samples: u32
}

impl Scene {
  pub fn objects<'a>(&'a self) -> Vec<&'a Intersectable> {
    let mut result = Vec::with_capacity(self.spheres.len() + self.meshes.len());
    for sp in &self.spheres {
      result.push(sp as &Intersectable);
    }
    for mesh in &self.meshes {
      result.push(mesh as &Intersectable);
    }

    result
  }
}

pub struct Output {
  pub filename: String,
  pub width: u32,
  pub height: u32,
  pub samples: u32,
}

pub struct Settings {
  pub scene: Scene,
  pub output: Output,
}

impl Settings {
  pub fn from_json(json: JsonValue) -> Option<Settings> {
    if let JsonValue::Object(nodes) = json {
      let mut scene = None;
      let mut output = None;

      for node in nodes {
        if node.0 == "scene" {
          scene = read_scene(node.1);
        } else if node.0 == "output" {
          output = read_output(node.1);
        }
      }

      return Some(Settings {
        scene: scene.unwrap(),
        output: output.unwrap()
      })
    }

    None
  }
}

fn read_scene(scene: JsonValue) -> Option<Scene> {
  if let JsonValue::Object(fields) = scene {
    let mut materials = Vec::new();
    let mut spheres = Vec::new();
    let mut meshes = Vec::new();
    let mut lights = Vec::new();
    let mut skycolor = Color {r: 0.0, g: 0.0, b: 0.0};
    let mut max_depth = 5;
    let mut path_samples = 1;

    for f in fields {
      if f.0 == "skycolor" {
        let v = read_number_triplet(&f.1).unwrap();
        skycolor.r = v.0;
        skycolor.g = v.1;
        skycolor.b = v.2;
      }
      else if f.0 == "max_trace_depth" {
        if let JsonValue::Number(md) = f.1 {
          max_depth = md as u32;
        }
      }
      else if f.0 == "path_samples" {
        if let JsonValue::Number(ps) = f.1 {
          path_samples = ps as u32;
        }
      }      
      else if let JsonValue::Array(values) = f.1 {
        if f.0 == "materials" {
          materials = read_materials(values);
        } else if f.0 == "spheres" {
          spheres = read_spheres(values);
        } else if f.0 == "meshes" {
          meshes = read_meshes(values);
        } else if f.0 == "lights" {
          lights = read_lights(values);
        }
      }
    }

    if max_depth <= 0 {
      path_samples = 0;
    }

    return Some(Scene {
      materials,
      spheres,
      meshes,
      lights,
      skycolor,
      max_depth,
      path_samples
    });
  }

  None
}

fn read_materials(materials: Vec<JsonValue>) -> Vec<Material> {
  let mut result = Vec::new();

  for mat in materials {
    if let JsonValue::Object(fields) = mat {
      let mut id: Option<String> = None;
      let mut color = Color::black();
      let mut reflect = 0.0;
      let mut refract = 0.0;
      let mut ior = 1.0;
      let mut roughness = 0.001;

      for f in fields {
        if f.0 == "id" {
          if let JsonValue::String(idstr) = f.1 {
            id = Some(idstr);
          }
        } else if f.0 == "color" {
          let values = read_number_triplet(&f.1).unwrap();
          color = Color::new(values.0, values.1, values.2);
        } else if f.0 == "refract" {
          if let JsonValue::Number(refr) = f.1 {
            refract = refr;
          }
        } else if f.0 == "reflect" {
          if let JsonValue::Number(refl) = f.1 {
            reflect = refl;
          }
        } else if f.0 == "ior" {
          if let JsonValue::Number(iorv) = f.1 {
            ior = iorv;
          }
        } else if f.0 == "roughness" {
          if let JsonValue::Number(rgv) = f.1 {
            roughness = rgv;
          }
        }
      }

      result.push(Material {
        id: id.unwrap(),
        color,
        reflect,
        refract,
        ior,
        roughness
      });
    }
  }

  result
}

fn read_spheres(spheres: Vec<JsonValue>) -> Vec<Sphere> {
  let mut result = Vec::new();

  for sp in spheres {
    if let JsonValue::Object(fields) = sp {
      let mut center = Vector4F {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
      };
      let mut radius = 1.0;
      let mut mat_id = String::from("_default");

      for f in fields {
        if f.0 == "center" {
          let values = read_number_triplet(&f.1).unwrap();
          center = Vector4F {
            x: values.0,
            y: values.1,
            z: values.2,
            w: 1.0,
          };
        } else if f.0 == "radius" {
          if let JsonValue::Number(rad) = f.1 {
            radius = rad;
          }
        } else if f.0 == "material" {
          if let JsonValue::String(matid) = f.1 {
            mat_id = matid;
          }
        }
      }

      result.push(Sphere {
        center: center,
        radius: radius,
        material: mat_id,
      });
    }
  }

  result
}

fn read_meshes(meshes: Vec<JsonValue>) -> Vec<Mesh> {
  let mut result = Vec::new();

  for mesh in meshes {
    if let JsonValue::Object(fields) = mesh {
      let mut vertices = Vec::new();
      let mut translation = Vector4F::null();
      let mut rotation = Vector4F::null();
      let mut scale = Vector4F::new(1.0, 1.0, 1.0);
      let mut material = String::new();

      for f in fields {
        if f.0 == "file" {
          if let JsonValue::String(s) = f.1 {
            println!("Loading mesh: '{}'", s);
            vertices = obj::load_obj(s.as_str());
            println!("Loaded {} vertices, {} triangles", vertices.len(), vertices.len() / 3);
          }
        }
        else if f.0 == "translation" {
          let values = read_number_triplet(&f.1).unwrap();
          translation = Vector4F {
            x: values.0,
            y: values.1,
            z: values.2,
            w: 1.0,
          };
        }
        else if f.0 == "scale" {
          let values = read_number_triplet(&f.1).unwrap();
          scale = Vector4F {
            x: values.0,
            y: values.1,
            z: values.2,
            w: 1.0,
          };
        }
        else if f.0 == "rotation" {
          let values = read_number_triplet(&f.1).unwrap();
          rotation = Vector4F {
            x: values.0,
            y: values.1,
            z: values.2,
            w: 1.0,
          };
        }
        else if f.0 == "material" {
          if let JsonValue::String(s) = f.1 {
            material = s;
          }
        }
      }

      let mut stopwatch = StopWatch::new();

      //Apply transform to position AND normals
      stopwatch.start();
      for vert in &mut vertices {
        let new_pos = vert.pos.rotate_x(rotation.x).rotate_y(rotation.y).rotate_z(rotation.z);
        vert.pos = &(&new_pos * &scale) + &translation;

        let new_norm = vert.normal.rotate_x(rotation.x).rotate_y(rotation.y).rotate_z(rotation.z);
        vert.normal = new_norm;
      }
      stopwatch.stop();
      println!("Transforming vertices took {}ms", stopwatch.get_millis());

      stopwatch.start();
      let triangles = create_triangles(&mut vertices);
      stopwatch.stop();
      println!("Creating triangles took {}ms", stopwatch.get_millis());

      stopwatch.start();
      let octree = octree::build_octree(&triangles);
      stopwatch.stop();
      println!("Building octree took {}ms", stopwatch.get_millis());

      let mut m = Mesh {
        triangles,
        translation,
        rotation,
        scale,
        material,
        octree
      };

      result.push(m);
    }
  }

  result
}

fn create_triangles(verts: &mut Vec<Vertex4F>) -> Vec<Triangle> {
  let num_tris = verts.len() / 3;
  let mut result = Vec::with_capacity(num_tris);

  for i in 0..num_tris {
    let i1 = i * 3;

    result.push(
      Triangle {
        v1: verts[i1].clone(),
        v2: verts[i1 + 1].clone(),
        v3: verts[i1 + 2].clone()
      }
    );
  }

  result
}

fn read_lights(lights: Vec<JsonValue>) -> Vec<Light> {
  let mut result = Vec::new();

  for light in lights {
    if let JsonValue::Object(fields) = light {
      let mut ltype = LightType::Point;
      let mut position = Vector4F {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
      };
      let mut color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
      };
      let mut radius = 1.0;
      let mut visible = false;
      let mut samples = 1;
      let mut intensity = 1.0;

      for f in fields {
        if f.0 == "type" {
          if let JsonValue::String(t) = f.1 {
            let ts = t.trim().to_lowercase();
            if ts == "point" {
              ltype = LightType::Point;
            } else if ts == "sphere" {
              ltype = LightType::Sphere;
            } else {
              let mut message = String::new();
              message.push_str("Unknown light type: ");
              message.push_str(ts.as_str());
              panic!(message);
            }
          }
        } else if f.0 == "position" {
          let values = read_number_triplet(&f.1).unwrap();
          position = Vector4F {
            x: values.0,
            y: values.1,
            z: values.2,
            w: 1.0,
          };
        } else if f.0 == "color" {
          let values = read_number_triplet(&f.1).unwrap();
          color = Color {
            r: values.0,
            g: values.1,
            b: values.2,
          }
        } else if f.0 == "radius" {
          if let JsonValue::Number(rad) = f.1 {
            radius = rad;
          }
        } else if f.0 == "samples" {
          if let JsonValue::Number(sm) = f.1 {
            samples = sm as u32;
          }
        } else if f.0 == "visible" {
          if let JsonValue::Boolean(b) = f.1 {
            visible = b;
          }
        } else if f.0 == "intensity" {
          if let JsonValue::Number(int) = f.1 {
            intensity = int;
          }
        }
      }

      result.push(Light {
        ltype,
        position,
        color,
        visible,
        radius,
        samples,
        intensity
      });

    }
  }

  result
}

fn read_output(output: JsonValue) -> Option<Output> {
  if let JsonValue::Object(fields) = output {
    let mut filename = String::from("render.tga");
    let mut width = 1920;
    let mut height = 1080;
    let mut samples = 1;

    for f in fields {
      if f.0 == "file" {
        if let JsonValue::String(st) = f.1 {
          filename = st;
        }
      }
      else if f.0 == "width" {
        if let JsonValue::Number(num) = f.1 {
          width = num as u32;
        }
      }
      else if f.0 == "height" {
        if let JsonValue::Number(num) = f.1 {
          height = num as u32;
        }
      }
      else if f.0 == "samples" {
        if let JsonValue::Number(num) = f.1 {
          samples = num as u32;
        }
      }
    }

    return Some(Output {
      filename,
      width,
      height,
      samples
    });
  }

  None
}

fn read_number_triplet(array: &JsonValue) -> Option<(f64, f64, f64)> {
  if let JsonValue::Array(values) = array {
    let mut v1 = 0.0;
    let mut v2 = 0.0;
    let mut v3 = 0.0;

    if let JsonValue::Number(n1) = values[0] {
      v1 = n1;
    }
    if let JsonValue::Number(n2) = values[1] {
      v2 = n2;
    }
    if let JsonValue::Number(n3) = values[2] {
      v3 = n3;
    }

    return Some((v1, v2, v3));
  }

  None
}
