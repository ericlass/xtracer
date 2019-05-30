use json::JsonValue;
use linear;
use linear::Intersection;
use linear::Vector4F;
use linear::Vertex4F;
use obj;
use octree;
use octree::OctreeNode;
use vox;
use std::clone::Clone;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use stopwatch::StopWatch;
use vox::VoxelObject;

pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b }
    }

    pub fn black() -> Color {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }
    }

    pub fn white() -> Color {
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        }
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

impl Clone for Color {
    fn clone(&self) -> Self {
        Color {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}

pub struct Material {
    pub id: String,
    pub color: Color,
    pub reflect: f64,
    pub refract: f64,
    pub ior: f64,
    pub roughness: f64,
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
    pub octree: OctreeNode,
}

impl Intersectable for Mesh {
    fn intersect(&self, rorg: &Vector4F, rdir: &Vector4F, min_t: f64) -> Option<Intersection> {
        let candidates = self.octree.intersection_candidates(rorg, &rdir.normalize());

        let mut closest = None;
        let mut lmin_t = min_t;

        for t in candidates {
            let tri = &self.triangles[t];

            let intersection =
                linear::intersect_ray_triangle(&rorg, &rdir, &tri.v1, &tri.v2, &tri.v3, lmin_t);

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
    pub intensity: f64,
}

pub struct Scene {
    pub materials: Vec<Material>,
    pub spheres: Vec<Sphere>,
    pub meshes: Vec<Mesh>,
    pub voxels: Vec<Voxels>,
    pub lights: Vec<Light>,
    pub skycolor: Color,
    pub max_depth: u32,
    pub path_samples: u32,
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
        for vox in &self.voxels {
            result.push(vox as &Intersectable);
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
                output: output.unwrap(),
            });
        }

        None
    }
}

pub struct Voxels {
    pub translation: Vector4F,
    pub rotation: Vector4F,
    pub scale: Vector4F,
    pub material: String,
    pub voxels: VoxelObject,
}

impl Intersectable for Voxels {
    fn intersect(&self, rorg: &Vector4F, rdir: &Vector4F, min_t: f64) -> Option<Intersection> {
        //Create inverted transforms to be able to go from world space to object space
        let inv_trans = &self.translation.invert();
        let inv_rot = &self.rotation.invert();
        let inv_scale = &Vector4F {
            x: 1.0 / self.scale.x,
            y: 1.0 / self.scale.y,
            z: 1.0 / self.scale.z,
            w: 1.0,
        };

        //Transform ray origin and direction into object space
        let rorg_obj_space = &(rorg + inv_trans) * inv_scale;

        let rdir_obj_space = rdir
            .rotate_x(inv_rot.x)
            .rotate_y(inv_rot.y)
            .rotate_z(inv_rot.z);

        let mut x: i32;
        let mut y: i32;
        let mut z: i32;

        let min = Vector4F::null();
        let max = Vector4F::new(
            self.voxels.width as f64,
            self.voxels.height as f64,
            self.voxels.depth as f64,
        );

        let intersection = linear::intersect_ray_aabb2(&rorg_obj_space, &rdir_obj_space, &min, &max);
        if intersection.is_some() {
            let inter = intersection.unwrap();

            let world_pos = &(&inter.pos * &self.scale) + &self.translation;
            let world_normal = inter.normal.rotate_x(self.rotation.x).rotate_y(self.rotation.y).rotate_z(self.rotation.z);

            //Need to recalc t with world coordinates
            let world_t = (world_pos.x - rorg.x) / rdir.x;

            //println!("ipos: {}", inter.pos);
            //println!("wpos: {}", world_pos);
            //println!("t: {}", world_t);

            return Some(Intersection {
                pos: world_pos,
                normal: world_normal,
                tex_u: 0.0,
                tex_v: 0.0,
                barycentric: Vector4F::null(),
                ray_t: world_t,
            });
        }
        else {
            return None;
        }

        if linear::point_in_aabb(&rorg_obj_space, &min, &max) {
            //If ray origin is inside voxel grid, just truncate coordinates to get starting voxel
            x = rorg_obj_space.x as i32;
            y = rorg_obj_space.y as i32;
            z = rorg_obj_space.z as i32;
        } else {
            //Find voxel where ray enters
            let inter = linear::intersect_ray_aabb(rorg, rdir, &min, &max);
            if inter.is_some() {
                let intersection = inter.unwrap();
                x = intersection.pos.x as i32;
                y = intersection.pos.y as i32;
                z = intersection.pos.z as i32;
                println!("x;y;z: {};{};{}", x, y, z);
            } else {
                return None;
            }
        }

        let step_x = rdir_obj_space.x.signum() as i32;
        let step_y = rdir_obj_space.y.signum() as i32;
        let step_z = rdir_obj_space.z.signum() as i32;

        let out_x: i32 = if step_x > 0 {self.voxels.width as i32} else {0};
        let out_y: i32 = if step_y > 0 {self.voxels.height as i32} else {0};
        let out_z: i32 = if step_z > 0 {self.voxels.depth as i32} else {0};

        let mut t_max_x = get_max_element(rorg_obj_space.x, rdir_obj_space.x, self.voxels.width as f64);
        let mut t_max_y = get_max_element(rorg_obj_space.y, rdir_obj_space.y, self.voxels.height as f64);
        let mut t_max_z = get_max_element(rorg_obj_space.z, rdir_obj_space.z, self.voxels.depth as f64);

        let t_delta_x = 1.0 / rdir_obj_space.x;
        let t_delta_y = 1.0 / rdir_obj_space.y;
        let t_delta_z = 1.0 / rdir_obj_space.z;

        let mut intersects = false;
        loop {
            if t_max_x < t_max_y {
                if t_max_x < t_max_z {
                    x = x + step_x;
                    if x == out_x {
                        break;
                    }
                    t_max_x = t_max_x + t_delta_x;
                } else {
                    z = z + step_z;
                    if z == out_z {
                        break;
                    }
                    t_max_z = t_max_z + t_delta_z;
                }
            } else {
                if t_max_y < t_max_z {
                    y = y + step_y;
                    if y == out_y {
                        break;
                    }
                    t_max_y = t_max_y + t_delta_y;
                } else {
                    z = z + step_z;
                    if z == out_z {
                        break;
                    }
                    t_max_z = t_max_z + t_delta_z;
                }
            }

            let voxel = self.voxels.get(x as u32, y as u32, z as u32);
            if voxel.is_some() {
                intersects = true;
                break;
            }
        }

        if intersects {
            let min = Vector4F::new(x as f64, y as f64, z as f64);
            let max = Vector4F::new((x + 1) as f64, (y + 1) as f64, (z + 1) as f64);
            return linear::intersect_ray_aabb(&rorg_obj_space, &rdir_obj_space, &min, &max);
        }

        None
    }

    fn material(&self) -> String {
        self.material.clone()
    }
}

fn get_max_element(org: f64, dir: f64, out: f64) -> f64 {
    let mut max = if dir > 0.0 {std::f64::INFINITY} else {std::f64::NEG_INFINITY};

    if dir > 0.0 {
        if org < 0.0 {
            max = dir / -org;
        } else if org >= 0.0 && org <= out {
            max = dir / (org.trunc() + 1.0);
        } else {
            panic!("Ray starts outside of voxel model, this should not happen!");
        }
    } else if dir < 0.0 {
        if org > out {
            max = -dir / (org - out);
        } else if org >= 0.0 && org <= out {
            max = -dir / (org.trunc());
        } else {
            panic!("Ray starts outside of voxel model, this should not happen!");
        }
    }

    max
}

fn read_scene(scene: JsonValue) -> Option<Scene> {
    if let JsonValue::Object(fields) = scene {
        let mut materials = Vec::new();
        let mut spheres = Vec::new();
        let mut meshes = Vec::new();
        let mut voxels = Vec::new();
        let mut lights = Vec::new();
        let mut skycolor = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        let mut max_depth = 5;
        let mut path_samples = 1;

        for f in fields {
            if f.0 == "skycolor" {
                let v = read_number_triplet(&f.1).unwrap();
                skycolor.r = v.0 as f32;
                skycolor.g = v.1 as f32;
                skycolor.b = v.2 as f32;
            } else if f.0 == "max_trace_depth" {
                if let JsonValue::Number(md) = f.1 {
                    max_depth = md as u32;
                }
            } else if f.0 == "path_samples" {
                if let JsonValue::Number(ps) = f.1 {
                    path_samples = ps as u32;
                }
            } else if let JsonValue::Array(values) = f.1 {
                if f.0 == "materials" {
                    materials = read_materials(values);
                } else if f.0 == "spheres" {
                    spheres = read_spheres(values);
                } else if f.0 == "meshes" {
                    meshes = read_meshes(values);
                } else if f.0 == "lights" {
                    lights = read_lights(values);
                } else if f.0 == "voxels" {
                    voxels = read_voxels(values);
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
            voxels,
            lights,
            skycolor,
            max_depth,
            path_samples,
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
                    color = Color::new(values.0 as f32, values.1 as f32, values.2 as f32);
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
                roughness,
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
                        println!(
                            "Loaded {} vertices, {} triangles",
                            vertices.len(),
                            vertices.len() / 3
                        );
                    }
                } else if f.0 == "translation" {
                    let values = read_number_triplet(&f.1).unwrap();
                    translation = Vector4F {
                        x: values.0,
                        y: values.1,
                        z: values.2,
                        w: 1.0,
                    };
                } else if f.0 == "scale" {
                    let values = read_number_triplet(&f.1).unwrap();
                    scale = Vector4F {
                        x: values.0,
                        y: values.1,
                        z: values.2,
                        w: 1.0,
                    };
                } else if f.0 == "rotation" {
                    let values = read_number_triplet(&f.1).unwrap();
                    rotation = Vector4F {
                        x: values.0,
                        y: values.1,
                        z: values.2,
                        w: 1.0,
                    };
                } else if f.0 == "material" {
                    if let JsonValue::String(s) = f.1 {
                        material = s;
                    }
                }
            }

            let mut stopwatch = StopWatch::new();

            //Apply transform to position AND normals
            stopwatch.start();
            for vert in &mut vertices {
                let new_pos = vert
                    .pos
                    .rotate_x(rotation.x)
                    .rotate_y(rotation.y)
                    .rotate_z(rotation.z);
                vert.pos = &(&new_pos * &scale) + &translation;

                let new_norm = vert
                    .normal
                    .rotate_x(rotation.x)
                    .rotate_y(rotation.y)
                    .rotate_z(rotation.z);
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
                octree,
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

        result.push(Triangle {
            v1: verts[i1].clone(),
            v2: verts[i1 + 1].clone(),
            v3: verts[i1 + 2].clone(),
        });
    }

    result
}

fn read_voxels(voxels: Vec<JsonValue>) -> Vec<Voxels> {
    let mut result = Vec::new();

    for vox in voxels {
        if let JsonValue::Object(fields) = vox {
            let mut voxels = None;
            let mut translation = Vector4F::null();
            let mut rotation = Vector4F::null();
            let mut scale = Vector4F::new(1.0, 1.0, 1.0);
            let mut material = String::new();

            for f in fields {
                if f.0 == "file" {
                    if let JsonValue::String(s) = f.1 {
                        println!("Loading voxel mesh: '{}'", s);
                        voxels = vox::read_voxels(s.as_str());
                    }
                } else if f.0 == "translation" {
                    let values = read_number_triplet(&f.1).unwrap();
                    translation = Vector4F {
                        x: values.0,
                        y: values.1,
                        z: values.2,
                        w: 1.0,
                    };
                } else if f.0 == "scale" {
                    let values = read_number_triplet(&f.1).unwrap();
                    scale = Vector4F {
                        x: values.0,
                        y: values.1,
                        z: values.2,
                        w: 1.0,
                    };
                } else if f.0 == "rotation" {
                    let values = read_number_triplet(&f.1).unwrap();
                    rotation = Vector4F {
                        x: values.0,
                        y: values.1,
                        z: values.2,
                        w: 1.0,
                    };
                } else if f.0 == "material" {
                    if let JsonValue::String(s) = f.1 {
                        material = s;
                    }
                }
            }

            let voxels = voxels.unwrap();
            println!("Loaded {} voxels", voxels.data.len());

            let mut v = Voxels {
                translation,
                rotation,
                scale,
                material,
                voxels
            };

            result.push(v);
        }
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
                        r: values.0 as f32,
                        g: values.1 as f32,
                        b: values.2 as f32,
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
                intensity,
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
            } else if f.0 == "width" {
                if let JsonValue::Number(num) = f.1 {
                    width = num as u32;
                }
            } else if f.0 == "height" {
                if let JsonValue::Number(num) = f.1 {
                    height = num as u32;
                }
            } else if f.0 == "samples" {
                if let JsonValue::Number(num) = f.1 {
                    samples = num as u32;
                }
            }
        }

        return Some(Output {
            filename,
            width,
            height,
            samples,
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
