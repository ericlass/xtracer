extern crate time;

mod linear;
mod tga;

use linear::Vector4F;

fn main() {
    let cam_pos = Vector4F {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0
    };

    let img_w = 1920;
    let img_h = 1080;

    let img_plane_dist = 1.0;

    let img_ratio = img_w as f64 / img_h as f64;
    let img_plane_w = img_plane_dist / 2.0;
    let img_plane_h = img_plane_w / img_ratio;

    let img_plane_l = cam_pos.x - (img_plane_w / 2.0);
    let img_plane_b = cam_pos.y - (img_plane_h / 2.0);

    let img_pix_inc_h = img_plane_w / img_w as f64;
    let img_pix_inc_v = img_plane_h / img_h as f64;

    let sp_c = Vector4F {
        x: 0.6,
        y: 0.0,
        z: 5.0,
        w: 1.0
    };

    let sp_r = 0.5;

    let sp_c2 = Vector4F {
        x: -0.1,
        y: 0.0,
        z: 8.0,
        w: 1.0
    };

    let sp_r2 = 0.5;

    let mut pixels: Vec<u8> = Vec::with_capacity((img_w * img_h) * 3);

    let start = time::precise_time_ns();
    for iy in 0..img_h {
        for ix in 0..img_w {
            let px = Vector4F {
                x: img_plane_l + (ix as f64 * img_pix_inc_h),
                y: img_plane_b + (iy as f64 * img_pix_inc_v),
                z: img_plane_dist,
                w: 0.0
            };

            let ray_dir = &px - &cam_pos;
            let intersects = linear::intersect_ray_sphere(&cam_pos, &ray_dir, &sp_c, sp_r, 1000.0);
            let intersects2 = linear::intersect_ray_sphere(&cam_pos, &ray_dir, &sp_c2, sp_r2, 1000.0);

            let mut closest: Option<linear::Intersection> = None;
            if intersects.is_some() {
                if intersects2.is_some() {
                    let i1 = intersects.unwrap();
                    let i2 = intersects2.unwrap();
                    if i1.ray_t < i2.ray_t {
                        closest = Some(i1);
                    }                    
                    else {
                        closest = Some(i2);
                    }
                }
                else {
                    closest = intersects;
                }
            }
            else {
                if intersects2.is_some() {
                    closest = intersects2;
                }
            }
            
            match closest {
                Some(inter) => {
                    let normal = inter.normal;
                    pixels.push(convert(normal.z));
                    pixels.push(convert(normal.y));
                    pixels.push(convert(normal.x));
                },
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

    tga::write_tga("render.tga", img_w as u16, img_h as u16, pixels.as_slice());
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
