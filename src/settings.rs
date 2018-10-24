use json::JsonValue;
use linear::Vector4F;
use linear::Vertex4F;

pub struct Color {
  pub r: f64,
  pub g: f64,
  pub b: f64,
}

pub struct Material {
  pub id: String,
  pub color: Color,
  pub reflect: f64,
  pub refract: f64,
  pub ior: f64,
}

pub struct Sphere {
  pub center: Vector4F,
  pub radius: f64,
  pub material: String,
}

pub struct Mesh {
  pub vertices: Vec<Vertex4F>,
  pub translation: Vector4F,
  pub rotation: Vector4F,
  pub scale: Vector4F,
  pub material: String,
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
}

pub struct Scene {
  pub materials: Vec<Material>,
  pub spheres: Vec<Sphere>,
  pub meshes: Vec<Mesh>,
  pub lights: Vec<Light>,
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

    for f in fields {
      if let JsonValue::Array(values) = f.1 {
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

    return Some(Scene {
      materials,
      spheres,
      meshes,
      lights
    });
  }

  None
}

fn read_materials(materials: Vec<JsonValue>) -> Vec<Material> {
  let mut result = Vec::new();

  for mat in materials {
    if let JsonValue::Object(fields) = mat {
      let mut id: Option<String> = None;
      let mut color: Option<Color> = Some(Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
      });
      let mut reflect: Option<f64> = Some(0.0);
      let mut refract: Option<f64> = Some(0.0);
      let mut ior: Option<f64> = Some(1.0);

      for f in fields {
        if f.0 == "id" {
          if let JsonValue::String(idstr) = f.1 {
            id = Some(idstr);
          }
        } else if f.0 == "color" {
          let values = read_number_triplet(f.1).unwrap();
          color = Some(Color {
            r: values.0,
            g: values.1,
            b: values.2,
          });
        } else if f.0 == "refract" {
          if let JsonValue::Number(refr) = f.1 {
            refract = Some(refr);
          }
        } else if f.0 == "reflect" {
          if let JsonValue::Number(refl) = f.1 {
            reflect = Some(refl);
          }
        } else if f.0 == "ior" {
          if let JsonValue::Number(iorv) = f.1 {
            ior = Some(iorv);
          }
        }
      }

      result.push(Material {
        id: id.unwrap(),
        color: color.unwrap(),
        reflect: reflect.unwrap(),
        refract: refract.unwrap(),
        ior: ior.unwrap(),
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
          let values = read_number_triplet(f.1).unwrap();
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
  Vec::new()
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
      let mut radius = 0.0;
      let mut visible = false;
      let mut samples = 1;

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
          let values = read_number_triplet(f.1).unwrap();
          position = Vector4F {
            x: values.0,
            y: values.1,
            z: values.2,
            w: 1.0,
          };
        } else if f.0 == "color" {
          let values = read_number_triplet(f.1).unwrap();
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
        }
      }

      result.push(Light {
        ltype,
        position,
        color,
        visible,
        radius,
        samples
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

fn read_number_triplet(array: JsonValue) -> Option<(f64, f64, f64)> {
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
