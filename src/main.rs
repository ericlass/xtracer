extern crate time;
extern crate rand;
extern crate num_cpus;

mod json;
mod linear;
mod settings;
mod tga;
mod shade;
mod stopwatch;
mod random;

use linear::Vector4F;
use settings::Settings;
use settings::Scene;
use settings::Color;
use settings::LightType;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use stopwatch::StopWatch;
use random::Random;

const HALF_SECOND: u64 = 500000000;

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

    //Pre-calculate values for multi sampling
    let samples = settings.output.samples as f64;
    let samples2 = samples * samples;
    let sample_width = img_pix_inc_h / samples;
    let sample_offset = (img_pix_inc_h / 2.0) - (sample_width / 2.0);

    let mut stop_watch = StopWatch::new();
    stop_watch.start();

    let mut last_time = time::precise_time_ns();
    let mut lines_done = 0;
    let mut py = img_plane_b;

    let numcpus = num_cpus::get();
    println!("Number of CPUs: {}", numcpus);

    let mut num_threads = 0;
    let mut iy = 0;    
    let arc_settings = Arc::new(settings);
    let arc_cam_pos = Arc::new(cam_pos);
    let mut lines = Vec::with_capacity(img_h as usize);
    for _i in 0..img_h {
        lines.push(Vec::new());
    }

    let (tx, rx) = mpsc::channel();

    while iy < img_h {
        while num_threads < numcpus && iy < img_h {
            let larc_settings = arc_settings.clone();
            let larc_cam_pos = arc_cam_pos.clone();
            let ltx = mpsc::Sender::clone(&tx);
            let liy = iy;

            thread::spawn(move || {
                let mut random = Random::new(31 + iy);
                let mut px = img_plane_l;
                let mut colors = Vec::with_capacity(img_w as usize);

                for _ix in 0..img_w {
                    //Create sample grid of samples * samples sub-pixels
                    let sub_pix_l = px - sample_offset;
                    let sub_pix_b = py - sample_offset;

                    let mut pix_color = Color::black();

                    let steps = larc_settings.output.samples;
                    let mut spy = sub_pix_b;
                    for _spy in 0..steps  {
                        let mut spx = sub_pix_l;
                        for _spx in 0..steps {
                            let pixel = Vector4F {
                                x: spx,
                                y: spy,
                                z: img_plane_dist,
                                w: 0.0,
                            };

                            let ray_dir = &pixel - &larc_cam_pos;
                            let pc = trace(&larc_cam_pos, &ray_dir, &larc_settings.scene, &mut random, 0);

                            pix_color.r += pc.r;
                            pix_color.g += pc.g;
                            pix_color.b += pc.b;

                            spx += sample_width;
                        }
                        spy += sample_width;
                    }

                    pix_color.r = pix_color.r / samples2;
                    pix_color.g = pix_color.g / samples2;
                    pix_color.b = pix_color.b / samples2;
                    colors.push(pix_color);

                    px += img_pix_inc_h;
                }
                    
                ltx.send((liy, colors)).unwrap();
            });
            
            num_threads += 1;
            py += img_pix_inc_v;
            iy += 1;
        }

        //Read back results from threads
        let mut rxv = rx.try_recv();
        while rxv.is_ok() {
            let result = rxv.unwrap();
            let line = result.0 as usize;
            lines[line] = result.1;
            num_threads -= 1;
            lines_done += 1;
            rxv = rx.try_recv();
        }

        let this_time = time::precise_time_ns();
        let diff = this_time - last_time;
        if diff > HALF_SECOND {
            let percent = (lines_done as f64 / img_h as f64) * 100.0;
            println!("{}%", percent.round());
            last_time = this_time;
        }
    }

    //Read all the rest (blocking)
    while num_threads > 0 {
        let rxv = rx.recv();
        let result = rxv.unwrap();
        let line = result.0;
        let colors = result.1;
        lines[line as usize] = colors;
        num_threads -= 1;
    }

    stop_watch.stop();
    println!("Render time: {}ms", stop_watch.get_millis());

    stop_watch.start();
    let mut pixels = Vec::with_capacity(((img_w * img_h) * 3) as usize);
    let mut rand = Random::new(97);
    for line in &lines {
        for col in line {
            pixels.push(convert(col.b, &mut rand));
            pixels.push(convert(col.g, &mut rand));
            pixels.push(convert(col.r, &mut rand));
        }
    }    
    stop_watch.stop();
    println!("Convert time: {}ms", stop_watch.get_millis());

    stop_watch.start();
    tga::write_tga(
        arc_settings.output.filename.as_str(),
        img_w as u16,
        img_h as u16,
        pixels.as_slice(),
    );
    stop_watch.stop();
    println!("Write time: {}ms", stop_watch.get_millis());
}

fn trace(ray_org: &Vector4F, ray_dir: &Vector4F, scene: &Scene, random: &mut Random, depth: u32) -> Color {
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
        let vdir = (ray_org - &inter.pos).normalize();

        let mut material = None;
        for mat in &scene.materials {
            if mat.id == sp.material {
                material = Some(mat);
                break;
            }
        }

        if material.is_some() {
            let mat = material.unwrap();
            let mut lcolor = Color::black();

            for light in &scene.lights {
                let mut light_intens = 0.0;
                let ldir = (&light.position - &inter.pos).normalize();

                if let LightType::Point = light.ltype {
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

                    light_intens = if is_in_shadow {0.0} else {1.0};
                }
                else if let LightType::Sphere = light.ltype {
                    let mut v = 0.0;

                    for _sample in 0..light.samples {
                        let rand_pos = random.random_point_on_sphere(&light.position, light.radius);
                        let sample_dir = &rand_pos - &inter.pos;
                        let mut is_in_shadow = false;
                        for l in 0..spheres.len() {
                            if l != closest_index {
                                let ssp = &spheres[l];
                                if linear::ray_intersects_sphere(&inter.pos, &sample_dir, &ssp.center, ssp.radius) {
                                    is_in_shadow = true;
                                    break;
                                }
                            }
                        }

                        if !is_in_shadow {
                            v = v + 1.0;
                        }
                    }

                    light_intens = v / (light.samples as f64);
                }

                //Realistic inverse-square light attenuation
                let ldist = (&light.position - &inter.pos).len();
                let ratio = light.radius / ldist;
                light_intens = (ratio * ratio) * light_intens * light.intensity;

                let diffuse = shade::shade_oren_nayar(&ldir, &inter.normal, &vdir, mat.roughness, 0.01);
                let specular = shade::shade_cook_torrance(&ldir, &vdir, &inter.normal, mat.roughness, 0.01);
                let shading = diffuse + specular;

                let light_total = shading * light_intens;
                lcolor.r = lcolor.r + (light.color.r * light_total);
                lcolor.g = lcolor.g + (light.color.g * light_total);
                lcolor.b = lcolor.b + (light.color.b * light_total);
            }

            //Reflection
            let mut refl_color = Color::black();
            if mat.reflect > 0.0 {
                let vr = Vector4F::reflect(&vdir.invert(), &inter.normal).normalize();
                let rc = trace(&inter.pos, &vr, scene, random, depth + 1);

                //Reflection fresnel (Schlick)
                let vdotn = Vector4F::dot(&vdir, &inter.normal);
                let f = (1.0 - vdotn).powf(mat.ior);

                refl_color = Color {
                    r: rc.r * f * mat.reflect,
                    g: rc.g * f * mat.reflect,
                    b: rc.b * f * mat.reflect,
                };
            }

            //Refraction
            //TODO: Does not work!
            /*
            let mut refr_color = Color::black();
            if mat.refract > 0.0 {
                let vr = Vector4F::refract(&vdir, &inter.normal, mat.ior).normalize();
                let rc = trace(&inter.pos, &vr, scene, random, depth + 1);

                refr_color = Color {
                    r: rc.r * mat.refract,
                    g: rc.g * mat.refract,
                    b: rc.b * mat.refract,
                };
            }
            */

            result.r = (mat.color.r * lcolor.r) + refl_color.r;
            result.g = (mat.color.g * lcolor.g) + refl_color.g;
            result.b = (mat.color.b * lcolor.b) + refl_color.b;
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

fn convert(v: f64, rand: &mut Random) -> u8 {
    let mut result = v;

    //Add some slight random noise to reduce banding
    let r = rand.random_f() * 2.0 - 1.0;
    result = result + (r * (1.0 / 512.0));
    
    if result < 0.0 {
        result = 0.0;
    }
    else if result > 1.0 {
        result = 1.0;
    }
    
    result = result * 255.0;

    result.round() as u8
}
