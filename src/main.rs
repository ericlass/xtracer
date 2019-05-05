extern crate time;
extern crate rand;
extern crate num_cpus;

mod json;
mod linear;
mod obj;
mod octree;
mod random;
mod settings;
mod shade;
mod stopwatch;
mod tga;

use linear::Vector4F;
use linear::Intersection;
use settings::Settings;
use settings::Scene;
use settings::Color;
use settings::LightType;
use settings::Intersectable;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use stopwatch::StopWatch;
use random::Random;

const HALF_SECOND: u64 = 500000000;

fn main() {
    let ro = Vector4F::new(1.01, 0.0, -2.0);
    let rd = Vector4F::new(0.0, 0.0, 1.0);

    let min = Vector4F::new(-1.0, -1.0, -1.0);
    let max = Vector4F::new(1.0, 1.0, 1.0);

    let int = linear::ray_intersects_aabb(&ro, &rd, &min, &max);
    println!("Intersects: {}", int);

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
    let samplesi = settings.output.samples;
    let samples = samplesi as f64;
    let samples2 = (samples * samples) as f32;
    let sample_width = img_pix_inc_h / samples;
    let sample_offset = (img_pix_inc_h / 2.0) - (sample_width / 2.0);

    let arc_settings = Arc::new(settings);
    let arc_cam_pos = Arc::new(cam_pos);

    let numcpus = num_cpus::get();
    println!("Number of CPUs: {}", numcpus);

    let num_values = img_h * img_w * 3;
    let mut final_buffer = vec![0.0f32; num_values as usize];

    let mut total_watch = StopWatch::new();
    total_watch.start();

    let mut stop_watch = StopWatch::new();
    stop_watch.start();

    let mut last_time = time::precise_time_ns();
    let mut lines_done = 0;
    let mut py = img_plane_b;

    let mut num_threads = 0;
    let mut iy = 0;    
    
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

                let num_values = (img_w * 3) as usize;
                let mut colors = Vec::with_capacity(num_values);

                for _ix in 0..img_w {
                    //Create sample grid of samples * samples sub-pixels
                    let sub_pix_l = px - sample_offset;
                    let sub_pix_b = py - sample_offset;

                    let mut pcr = 0.0;
                    let mut pcg = 0.0;
                    let mut pcb = 0.0;

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

                            pcr += pc.r;
                            pcg += pc.g;
                            pcb += pc.b;

                            spx += sample_width;
                        }
                        spy += sample_width;
                    }

                    colors.push(pcb / samples2);
                    colors.push(pcg / samples2);
                    colors.push(pcr / samples2);

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

            let stride = img_w as usize * 3;
            let start = line * stride;
            let end = start + stride;

            let new = &result.1;

            let mut nl = 0;
            for l in start..end {
                final_buffer[l] += new[nl];
                nl += 1;
            }

            num_threads -= 1;
            lines_done += 1;
            rxv = rx.try_recv();
        }

        let this_time = time::precise_time_ns();
        let diff = this_time - last_time;
        if diff > HALF_SECOND {
            let mut percent = (lines_done as f64 / img_h as f64) * 100.0;
            percent = (percent * 100.0).round() / 100.0;
            println!("{} %", percent);
            last_time = this_time;
        }
    }

    //Read all the rest (blocking)
    while num_threads > 0 {
        let rxv = rx.recv();
        let result = rxv.unwrap();
        let line = result.0 as usize;

        let stride = img_w as usize * 3;
        let start = line * stride;
        let end = start + stride;

        let new = &result.1;

        let mut nl = 0;
        for l in start..end {
            final_buffer[l] += new[nl];
            nl += 1;
        }

        num_threads -= 1;
    }

    stop_watch.stop();
    let render_millis = stop_watch.get_millis();
    println!("Render time: {}ms", render_millis);

    println!("=========================");

    stop_watch.start();
    let mut pixels = Vec::with_capacity(((img_w * img_h) * 3) as usize);
    let mut rand = Random::new(97);
    for line in &final_buffer {
        pixels.push(convert(*line, &mut rand));
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

    println!("=========================");
    total_watch.stop();
    println!("TOTAL: {}ms", total_watch.get_millis());

    let spp = (samplesi * samplesi) * (arc_settings.scene.path_samples.pow(arc_settings.scene.max_depth));
    println!("Samples Per Pixel: {}", spp);
}

//Checks if the given ray (ray_org -> ray_dir) intersects any of the objects in the given vec and returns the closest point of intersection and the corresponding object.
fn intersect<'a>(ray_org: &Vector4F, ray_dir: &Vector4F, objects: &'a Vec<&Intersectable>) -> (Option<Intersection>, Option<&'a Intersectable>) {
    let mut closest = None;
    let mut closest_object = None;
    let mut min_t = std::f64::MAX;

    for obj in objects {
        let intersection = obj.intersect(ray_org, ray_dir, min_t);

        if intersection.is_some() {
            let inter = intersection.unwrap();

            if inter.ray_t < min_t {
                min_t = inter.ray_t;
                closest = Some(inter);
                closest_object = Some(*obj);
            }
        }
    }

    (closest, closest_object)
}

//Checks if the given ray (ray_org -> ray_dir) intersects any of the objects in the given vec.
fn intersect_any(ray_org: &Vector4F, ray_dir: &Vector4F, objects: &Vec<&Intersectable>) -> bool {
    for obj in objects {
        if obj.intersect(ray_org, ray_dir, std::f64::MAX).is_some() {
            return true;
        }
    }

    false
}

//Traces the given ray (ray_org -> ray_dir) from the camera into the scene, shading and recursivly path tracing accordingly. Returns the color of the pixel.
fn trace(ray_org: &Vector4F, ray_dir: &Vector4F, scene: &Scene, random: &mut Random, depth: u32) -> Color {
    let mut result = Color::black();

    if depth > scene.max_depth {
        return result;
    }

    let objects = scene.objects();

    let inter = intersect(ray_org, ray_dir, &objects);
    let closest = inter.0;
    let closest_object = inter.1;

    if closest.is_some() {
        let inter = closest.unwrap();
        let object = closest_object.unwrap();
        let vdir = (ray_org - &inter.pos).normalize();

        let mat_name = object.material();
        let mut material = None;
        for mat in &scene.materials {
            if mat.id == mat_name {
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
                    light_intens = if intersect_any(&inter.pos, &ldir, &objects) {0.0} else {1.0};
                }
                else if let LightType::Sphere = light.ltype {
                    let mut v = 0.0;

                    for _sample in 0..light.samples {
                        let rand_pos = random.random_point_on_sphere(&light.position, light.radius);
                        let sample_dir = &rand_pos - &inter.pos;

                        if !intersect_any(&inter.pos, &sample_dir, &objects) {
                            v = v + 1.0;
                        }
                    }

                    light_intens = v / (light.samples as f64);
                }

                //Realistic inverse-square light attenuation
                let ldist = (&light.position - &inter.pos).len();
                let ratio = light.radius / ldist;
                light_intens = (ratio * ratio) * light_intens * light.intensity;
                
                /*let diffuse = shade::shade_oren_nayar(&ldir, &inter.normal, &vdir, mat.roughness, 0.01);
                let specular = shade::shade_cook_torrance(&ldir, &vdir, &inter.normal, mat.roughness, 0.01);
                let shading = diffuse + specular;*/

                let shading = shade::shade_lambert(&ldir, &inter.normal);

                let light_total = shading * light_intens;

                lcolor.r += light.color.r * light_total as f32;
                lcolor.g += light.color.g * light_total as f32;
                lcolor.b += light.color.b * light_total as f32;
            }

            if scene.path_samples > 0 {
                let mut path_color = Color::black();

                for _ps in 0..scene.path_samples {
                    let path_dir = random.random_point_on_hemisphere(&inter.normal);
                    let pc = trace(&inter.pos, &path_dir, scene, random, depth + 1);

                    /*let diffuse = shade::shade_oren_nayar(&path_dir, &inter.normal, &vdir, mat.roughness, 0.1);
                    let specular = shade::shade_cook_torrance(&path_dir, &vdir, &inter.normal, mat.roughness, 0.1);
                    let shading = diffuse + specular;*/

                    let shading = shade::shade_lambert(&path_dir, &inter.normal);

                    path_color.r += pc.r * shading as f32;
                    path_color.g += pc.g * shading as f32;
                    path_color.b += pc.b * shading as f32;
                }

                let ps = 1.0 / (scene.path_samples as f32);
                
                path_color.r *= ps;
                path_color.g *= ps;
                path_color.b *= ps;

                lcolor.r += path_color.r;
                lcolor.g += path_color.g;
                lcolor.b += path_color.b;
            }

            //Enabling this only shows GI
            /*if depth == 0 {
                result.r = path_color.r;
                result.g = path_color.g;
                result.b = path_color.b;
            }
            else {*/
                result.r = mat.color.r * lcolor.r;
                result.g = mat.color.g * lcolor.g;
                result.b = mat.color.b * lcolor.b;
            //}
        } else {
            //If no material could be found, color is black
            println!("Material not found: {}", mat_name);

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

fn convert(v: f32, rand: &mut Random) -> u8 {
    let mut result = v;

    //Add some slight random noise to reduce banding
    let r = (rand.random_f() * 2.0 - 1.0) as f32;
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
