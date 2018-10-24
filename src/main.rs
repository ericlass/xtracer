extern crate time;

mod json;
mod linear;
mod settings;
mod tga;

use linear::Vector4F;
use settings::Settings;
use std::fs::File;
use std::io::Read;

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

    let start = time::precise_time_ns();
    for iy in 0..img_h {
        for ix in 0..img_w {
            let pixel = Vector4F {
                x: img_plane_l + (ix as f64 * img_pix_inc_h),
                y: img_plane_b + (iy as f64 * img_pix_inc_v),
                z: img_plane_dist,
                w: 0.0,
            };

            let ray_dir = &pixel - &cam_pos;

            let mut closest: Option<linear::Intersection> = None;
            let mut min_t = 9999999999.99;

            for sphere in spheres {
                let intersects = linear::intersect_ray_sphere(
                    &cam_pos,
                    &ray_dir,
                    &sphere.center,
                    sphere.radius,
                    min_t,
                );

                if intersects.is_some() {
                    let inter = intersects.unwrap();

                    if closest.is_some() {
                        if inter.ray_t < min_t {
                            min_t = inter.ray_t;
                            closest = Some(inter);
                        }
                    }
                    else {
                        min_t = inter.ray_t;
                        closest = Some(inter);
                    }
                }
            }

            match closest {
                Some(inter) => {
                    let normal = inter.normal;
                    pixels.push(convert(normal.z));
                    pixels.push(convert(normal.y));
                    pixels.push(convert(normal.x));
                }
                _ => {
                    pixels.push(64);
                    pixels.push(64);
                    pixels.push(64);
                }
            };
        }
    }
    let end = time::precise_time_ns();
    let duration_ns = end as f64 - start as f64;
    let duration = duration_ns / 1000000.0;
    println!("Render time: {}ms", duration);

    tga::write_tga(settings.output.filename.as_str(), img_w as u16, img_h as u16, pixels.as_slice());
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
        //result = result * -1.0;
        result = 0.0;
    }

    if result > 1.0 {
        result = 1.0;
    }

    result = result * 255.0;

    result as u8
}
