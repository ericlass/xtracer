extern crate time;

mod json;
mod linear;
mod settings;
mod tga;
mod shade;

use linear::Vector4F;
use settings::Settings;
use settings::Scene;
use settings::Color;
use settings::LightType;
use settings::Material;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

fn main() {
    let settings = load_settings();

    let cam_pos = Vector4F {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    let img_plane_dist = 1.0;

    let img_w = settings.output.width;
    let img_h = settings.output.height;

    //Calculate image plane dimensions
    let img_ratio = img_w as f64 / img_h as f64;
    let img_plane_w = img_plane_dist / 2.0;
    let img_plane_h = img_plane_w / img_ratio;
    let img_plane_l = cam_pos.x - (img_plane_w / 2.0);
    let img_plane_b = cam_pos.y - (img_plane_h / 2.0);

    //Calculate pixel vertical and horizontal increment
    let img_pix_inc_h = img_plane_w / img_w as f64;
    let img_pix_inc_v = img_plane_h / img_h as f64;

    let mut pixels: Vec<u8> = Vec::with_capacity(((img_w * img_h) * 3) as usize);
    let spheres = &settings.scene.spheres;

    //Create map of materials for faster lookup
    let mut mat_map = HashMap::new();
    for m in 0..settings.scene.materials.len() {
        mat_map.insert(settings.scene.materials[m].id.as_str(), &settings.scene.materials[m]);
    }

    let start = time::precise_time_ns();
    let mut py = img_plane_b;
    for _iy in 0..img_h {
        let mut px = img_plane_l;
        for _ix in 0..img_w {
            let pixel = Vector4F {
                x: px,
                y: py,
                z: img_plane_dist,
                w: 0.0,
            };

            let ray_dir = &pixel - &cam_pos;
            let pix_color = trace(&cam_pos, &ray_dir, &settings.scene, &mat_map);

            pixels.push(convert(pix_color.b));
            pixels.push(convert(pix_color.g));
            pixels.push(convert(pix_color.r));
            
            px = px + img_pix_inc_h;
        }

        py = py + img_pix_inc_v;
    }
    let end = time::precise_time_ns();
    let duration_ns = end as f64 - start as f64;
    let duration = duration_ns / 1000000.0;
    println!("Render time: {}ms", duration);

    tga::write_tga(
        settings.output.filename.as_str(),
        img_w as u16,
        img_h as u16,
        pixels.as_slice(),
    );
}

fn trace(ray_org: &Vector4F, ray_dir: &Vector4F, scene: &Scene, mat_map: &HashMap<&str, &Material>) -> Color {
    let mut result = Color::black();

    let spheres = &scene.spheres;
    let mut closest = None;
    let mut closest_index = 0;
    let mut min_t = 9999999999.99;

    for i in 0..spheres.len() {
        let sphere = &spheres[i];

        let intersection = linear::intersect_ray_sphere(
            &ray_org,
            &ray_dir,
            &sphere.center,
            sphere.radius,
            min_t,
        );

        if intersection.is_some() {
            let inter = intersection.unwrap();

            if inter.ray_t < min_t {
                min_t = inter.ray_t;
                closest = Some(inter);
                closest_index = i;
            }
        }
    }

    if closest.is_some() {
        let sp = &spheres[closest_index];
        let inter = closest.unwrap();

        let material = mat_map.get(sp.material.as_str());
        if material.is_some() {
            let mut lcolor = Color {r: 0.0, g: 0.0, b: 0.0};
            for light in &scene.lights {
                if let LightType::Point = light.ltype {
                    let ldir = &light.position - &inter.pos;
                    let mut is_in_shadow = false;

                    for l in 0..spheres.len() {
                        if l != closest_index {
                            let ssp = &spheres[l];
                            if linear::ray_intersects_sphere(&inter.pos, &ldir, &ssp.center, ssp.radius) {
                                is_in_shadow = true;
                                break;
                            }
                        }
                    }

                    if !is_in_shadow {
                        let s = shade::shade_lambert(&ldir, &inter.normal);

                        lcolor.r = lcolor.r + (s * light.color.r);
                        lcolor.g = lcolor.g + (s * light.color.g);
                        lcolor.b = lcolor.b + (s * light.color.b);
                    }
                }
                else if let LightType::Sphere = light.ltype {
                }
            }

            let mat = material.unwrap();
            result.r = mat.color.r * lcolor.r;
            result.g = mat.color.g * lcolor.g;
            result.b = mat.color.b * lcolor.b;
        } else {
            //If no material could be found, color is black
            result.r = 0.0;
            result.g = 0.0;
            result.b = 0.0;
        }
    } else {
        result.r = scene.skycolor.r;
        result.g = scene.skycolor.g;
        result.b = scene.skycolor.b;
    }

    result
}

fn load_settings() -> Settings {
    let args: Vec<_> = std::env::args().collect();
    let mut filename = "settings.json";
    if args.len() > 1 {
        filename = args[1].as_str();
    }

    let mut file = File::open(filename).unwrap();
    let mut json = String::new();
    file.read_to_string(&mut json).unwrap();

    let json_object = json::parse_json(&json);
    if let Some(object) = json_object {
        return Settings::from_json(object).unwrap();
    }

    panic!("Unable to read settings!");
}

fn convert(v: f64) -> u8 {
    let mut result = v;
    if result < 0.0 {
        result = 0.0;
    }
    if result > 1.0 {
        result = 1.0;
    }
    result = result * 255.0;

    result as u8
}
