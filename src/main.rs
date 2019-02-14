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
    let samples2 = samples * samples;
    let sample_width = img_pix_inc_h / samples;
    let sample_offset = (img_pix_inc_h / 2.0) - (sample_width / 2.0);

    let arc_settings = Arc::new(settings);
    let arc_cam_pos = Arc::new(cam_pos);

    let numcpus = num_cpus::get();
    println!("Number of CPUs: {}", numcpus);

    let mut lines: Vec<Vec<Color>> = Vec::with_capacity(img_h as usize);
    for _i in 0..img_h {
        let mut line = Vec::with_capacity(img_w as usize);
        for _j in 0..img_w {
            line.push(Color::black());
        }
        lines.push(line);
    }

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

            let orig = &mut lines[line];
            let new = &result.1;
            for l in 0..new.len() {
                let oc = &mut orig[l];
                let nc = &new[l];

                oc.r = oc.r + nc.r;
                oc.g = oc.g + nc.g;
                oc.b = oc.b + nc.b;
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
        
        let orig = &mut lines[line];
        let new = &result.1;
        for l in 0..new.len() {
            let oc = &mut orig[l];
            let nc = &new[l];

            oc.r = oc.r + nc.r;
            oc.g = oc.g + nc.g;
            oc.b = oc.b + nc.b;
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

    println!("=========================");
    total_watch.stop();
    println!("TOTAL: {}ms", total_watch.get_millis());
}

fn trace(ray_org: &Vector4F, ray_dir: &Vector4F, scene: &Scene, random: &mut Random, depth: u32) -> Color {
    let mut result = Color::black();

    if depth > scene.max_depth {
        return result;
    }

    let objects = scene.objects();

    let mut closest = None;
    let mut closest_object = None;
    let mut min_t = std::f64::MAX;

    for obj in &objects {
        let intersection = obj.intersect(ray_org, ray_dir, min_t);

        if intersection.is_some() {
            let inter = intersection.unwrap();

            if inter.ray_t < min_t {
                min_t = inter.ray_t;
                closest = Some(inter);
                closest_object = Some(obj);
            }
        }
    }

    if closest.is_some() {
        let inter = closest.unwrap();
        let object = closest_object.unwrap();
        let vdir = (ray_org - &inter.pos).normalize();

        if inter.ray_t == 0.0 {
            dbg!(inter.ray_t);
        }

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
            let mut light_verts = Vec::with_capacity(scene.path_samples as usize);

            for light in &scene.lights {
                let mut light_intens = 0.0;
                let ldir = (&light.position - &inter.pos).normalize();

                if let LightType::Point = light.ltype {
                    let mut is_in_shadow = false;

                    for obj in &objects {
                        if obj.intersect(&inter.pos, &ldir, std::f64::MAX).is_some() {
                            is_in_shadow = true;
                            break;
                        }
                    }

                    light_intens = if is_in_shadow {0.0} else {1.0};

                    for _ps in 0..scene.path_samples {
                        let rand_dir = random.random_direction();
                        for obj in &objects {
                            let s_inter = obj.intersect(&light.position, &rand_dir, std::f64::MAX);
                            if s_inter.is_some() {
                                light_verts.push(s_inter.unwrap().pos);
                            }
                        }
                    }
                }
                else if let LightType::Sphere = light.ltype {
                    let mut v = 0.0;

                    for _sample in 0..light.samples {
                        let rand_pos = random.random_point_on_sphere(&light.position, light.radius);
                        let sample_dir = &rand_pos - &inter.pos;
                        let mut is_in_shadow = false;

                        for obj in &objects {
                            if obj.intersect(&inter.pos, &sample_dir, std::f64::MAX).is_some() {
                                is_in_shadow = true;
                                break;
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

                lcolor.r += light.color.r * light_total;
                lcolor.g += light.color.g * light_total;
                lcolor.b += light.color.b * light_total;
            }

            if light_verts.len() > 0 {
                let mut path_color = Color::black();

                for lv in &light_verts {
                    let path_dir = lv - &inter.pos;
                    let pc = trace(&inter.pos, &path_dir, scene, random, depth + 1);

                    let diffuse = shade::shade_oren_nayar(&path_dir, &inter.normal, &vdir, mat.roughness, 0.1);
                    let specular = shade::shade_cook_torrance(&path_dir, &vdir, &inter.normal, mat.roughness, 0.1);
                    let shading = diffuse + specular;
                    
                    path_color.r += pc.r * shading;
                    path_color.g += pc.g * shading;
                    path_color.b += pc.b * shading;
                }

                let ps = 1.0 / (light_verts.len() as f64);
                path_color.r *= ps;
                path_color.g *= ps;
                path_color.b *= ps;
                
                lcolor.r += path_color.r;
                lcolor.g += path_color.r;
                lcolor.b += path_color.r;
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
